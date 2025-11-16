#!/usr/bin/env nu

# MOP Backup Script
# Automated backup of configurations and dashboards
#
# Usage: ./backup.nu --env <environment> [options]
#
# Features:
# - Export Grafana dashboards
# - Export Grafana datasources
# - Backup Tanka configurations
# - Create timestamped archives
# - Upload to cloud storage (S3/GCS)
# - Verify backup integrity

def main [
    --env: string               # Environment to backup (dev/staging/prod)
    --output: string = "backups" # Output directory for backups
    --upload: string            # Cloud storage URL (s3://bucket or gs://bucket)
    --retention: int = 30       # Retention period in days
    --grafana-url: string = "http://localhost:3000" # Grafana URL
    --grafana-token: string     # Grafana API token (or use GRAFANA_TOKEN env var)
] {
    print $"💾 (ansi green_bold)MOP Backup Starting(ansi reset)"
    print $"   Environment: (ansi yellow)($env)(ansi reset)"
    print ""

    try {
        # Create backup directory
        let timestamp = (date now | format date '%Y%m%d-%H%M%S')
        let backup_dir = $"($output)/mop-($env)-($timestamp)"
        mkdir $backup_dir

        print $"📁 Backup directory: ($backup_dir)"
        print ""

        # Get Grafana token
        let token = if $grafana_token != null {
            $grafana_token
        } else {
            $env.GRAFANA_TOKEN? | default ""
        }

        # Run backup tasks
        backup-grafana-dashboards $backup_dir $grafana_url $token
        backup-grafana-datasources $backup_dir $grafana_url $token
        backup-tanka-configs $backup_dir $env
        backup-kubernetes-resources $backup_dir $env

        # Create archive
        let archive_path = create-archive $backup_dir

        # Upload to cloud storage if requested
        if $upload != null {
            upload-backup $archive_path $upload
        }

        # Cleanup old backups
        cleanup-old-backups $output $retention

        # Verify backup
        verify-backup $archive_path

        print ""
        print $"✅ (ansi green_bold)Backup completed successfully!(ansi reset)"
        print $"   Archive: ($archive_path)"
        print $"   Size: (get-file-size $archive_path)"

    } catch { |err|
        print $"❌ (ansi red_bold)Backup failed:(ansi reset) ($err.msg)"
        exit 1
    }
}

# Backup Grafana dashboards
def backup-grafana-dashboards [backup_dir: string, grafana_url: string, token: string] {
    print $"📊 (ansi cyan)Backing up Grafana dashboards...(ansi reset)"

    let dashboard_dir = $"($backup_dir)/grafana/dashboards"
    mkdir $dashboard_dir

    if $token == "" {
        print $"   ⚠️  No Grafana token provided, skipping dashboard backup"
        return
    }

    # Get list of all dashboards
    let search_url = $"($grafana_url)/api/search?type=dash-db"
    let headers = [Authorization $"Bearer ($token)"]

    let search_result = (http get -H $headers $search_url 2>&1 | complete)

    if $search_result.exit_code != 0 {
        print $"   ⚠️  Failed to fetch dashboard list: ($search_result.stderr)"
        return
    }

    let dashboards = ($search_result.stdout | from json)
    let total = ($dashboards | length)

    print $"   Found ($total) dashboards"

    # Export each dashboard
    mut exported = 0
    for dashboard in $dashboards {
        let uid = $dashboard.uid
        let title = $dashboard.title
        let slug = ($title | str replace -a " " "-" | str downcase)

        print $"   Exporting: ($title)"

        let dashboard_url = $"($grafana_url)/api/dashboards/uid/($uid)"
        let dashboard_result = (http get -H $headers $dashboard_url 2>&1 | complete)

        if $dashboard_result.exit_code == 0 {
            let dashboard_data = ($dashboard_result.stdout | from json)
            $dashboard_data | to json -i 2 | save -f $"($dashboard_dir)/($slug).json"
            $exported = $exported + 1
        } else {
            print $"   ⚠️  Failed to export: ($title)"
        }
    }

    print $"   ✓ Exported ($exported)/($total) dashboards"
    print ""
}

# Backup Grafana datasources
def backup-grafana-datasources [backup_dir: string, grafana_url: string, token: string] {
    print $"🔌 (ansi cyan)Backing up Grafana datasources...(ansi reset)"

    let datasource_dir = $"($backup_dir)/grafana/datasources"
    mkdir $datasource_dir

    if $token == "" {
        print $"   ⚠️  No Grafana token provided, skipping datasource backup"
        return
    }

    let datasources_url = $"($grafana_url)/api/datasources"
    let headers = [Authorization $"Bearer ($token)"]

    let result = (http get -H $headers $datasources_url 2>&1 | complete)

    if $result.exit_code != 0 {
        print $"   ⚠️  Failed to fetch datasources: ($result.stderr)"
        return
    }

    let datasources = ($result.stdout | from json)
    let total = ($datasources | length)

    print $"   Found ($total) datasources"

    # Export each datasource
    for ds in $datasources {
        let name = ($ds.name | str replace -a " " "-" | str downcase)
        print $"   Exporting: ($ds.name) \(($ds.type)\)"

        # Remove sensitive fields
        let safe_ds = ($ds | reject secureJsonData | reject password)
        $safe_ds | to json -i 2 | save -f $"($datasource_dir)/($name).json"
    }

    print $"   ✓ Exported ($total) datasources"
    print ""
}

# Backup Tanka configurations
def backup-tanka-configs [backup_dir: string, env: string] {
    print $"⚙️  (ansi cyan)Backing up Tanka configurations...(ansi reset)"

    let tanka_dir = $"($backup_dir)/tanka"
    mkdir $tanka_dir

    # Copy environment files
    if ("environments" | path exists) {
        print $"   Copying environments directory..."
        cp -r environments $tanka_dir
    }

    # Copy lib files
    if ("lib" | path exists) {
        print $"   Copying lib directory..."
        cp -r lib $tanka_dir
    }

    # Copy jsonnetfile
    if ("jsonnetfile.json" | path exists) {
        print $"   Copying jsonnetfile.json..."
        cp jsonnetfile.json $tanka_dir
    }

    # Copy tanka config
    if ("tanka.yml" | path exists) {
        print $"   Copying tanka.yml..."
        cp tanka.yml $tanka_dir
    }

    # Export current rendered config
    print $"   Exporting rendered configuration..."
    let rendered_dir = $"($tanka_dir)/rendered"
    mkdir $rendered_dir

    let env_dir = $"environments/($env)"
    if ($env_dir | path exists) {
        let rendered = (tk show $env_dir 2>&1 | complete)
        if $rendered.exit_code == 0 {
            $rendered.stdout | save -f $"($rendered_dir)/($env).yaml"
            print $"   ✓ Rendered configuration saved"
        }
    }

    print $"   ✓ Tanka configurations backed up"
    print ""
}

# Backup Kubernetes resources
def backup-kubernetes-resources [backup_dir: string, env: string] {
    print $"☸️  (ansi cyan)Backing up Kubernetes resources...(ansi reset)"

    let k8s_dir = $"($backup_dir)/kubernetes"
    mkdir $k8s_dir

    let namespace = $"mop-($env)"

    # Resource types to backup
    let resources = [
        "configmaps"
        "secrets"
        "services"
        "deployments"
        "statefulsets"
        "persistentvolumeclaims"
        "ingresses"
    ]

    for resource in $resources {
        print $"   Exporting ($resource)..."

        let output = (kubectl get $resource -n $namespace -o yaml 2>&1 | complete)

        if $output.exit_code == 0 and ($output.stdout | str length) > 0 {
            $output.stdout | save -f $"($k8s_dir)/($resource).yaml"
        } else {
            print $"   ⚠️  No ($resource) found or export failed"
        }
    }

    print $"   ✓ Kubernetes resources backed up"
    print ""
}

# Create compressed archive
def create-archive [backup_dir: string] {
    print $"📦 (ansi cyan)Creating archive...(ansi reset)"

    let archive_name = ($backup_dir | path basename)
    let archive_path = $"($backup_dir).tar.gz"

    print $"   Compressing: ($archive_name)"

    let parent_dir = ($backup_dir | path dirname)

    # Create tar.gz archive
    cd $parent_dir
    tar -czf $archive_path $archive_name

    # Remove uncompressed directory
    rm -rf $backup_dir

    print $"   ✓ Archive created: ($archive_path)"
    print ""

    $archive_path
}

# Upload backup to cloud storage
def upload-backup [archive_path: string, storage_url: string] {
    print $"☁️  (ansi cyan)Uploading to cloud storage...(ansi reset)"

    let filename = ($archive_path | path basename)

    if ($storage_url | str starts-with "s3://") {
        # AWS S3 upload
        print $"   Uploading to S3: ($storage_url)"

        let upload = (aws s3 cp $archive_path $"($storage_url)/($filename)" 2>&1 | complete)

        if $upload.exit_code == 0 {
            print $"   ✓ Uploaded to S3"
        } else {
            print $"   ⚠️  S3 upload failed: ($upload.stderr)"
        }

    } else if ($storage_url | str starts-with "gs://") {
        # Google Cloud Storage upload
        print $"   Uploading to GCS: ($storage_url)"

        let upload = (gsutil cp $archive_path $"($storage_url)/($filename)" 2>&1 | complete)

        if $upload.exit_code == 0 {
            print $"   ✓ Uploaded to GCS"
        } else {
            print $"   ⚠️  GCS upload failed: ($upload.stderr)"
        }

    } else {
        print $"   ⚠️  Unknown storage type: ($storage_url)"
        print $"   Supported: s3:// or gs://"
    }

    print ""
}

# Cleanup old backups
def cleanup-old-backups [backup_dir: string, retention_days: int] {
    print $"🧹 (ansi cyan)Cleaning up old backups...(ansi reset)"

    if not ($backup_dir | path exists) {
        print $"   Backup directory does not exist"
        return
    }

    let cutoff_date = ((date now) - ($retention_days * 24 * 60 * 60 * 1_000_000_000))

    let old_backups = (ls $backup_dir | where type == "file" and name =~ "\.tar\.gz$" | where modified < $cutoff_date)

    let count = ($old_backups | length)

    if $count > 0 {
        print $"   Found ($count) backups older than ($retention_days) days"

        for backup in $old_backups {
            print $"   Removing: ($backup.name)"
            rm $backup.name
        }

        print $"   ✓ Removed ($count) old backups"
    } else {
        print $"   No old backups to remove"
    }

    print ""
}

# Verify backup integrity
def verify-backup [archive_path: string] {
    print $"✅ (ansi cyan)Verifying backup integrity...(ansi reset)"

    # Test tar archive
    let test_result = (tar -tzf $archive_path 2>&1 | complete)

    if $test_result.exit_code == 0 {
        let file_count = ($test_result.stdout | lines | length)
        print $"   ✓ Archive is valid ($file_count files)"
    } else {
        print $"   ❌ Archive verification failed: ($test_result.stderr)"
        error make {msg: "Backup verification failed"}
    }

    print ""
}

# Get human-readable file size
def get-file-size [path: string] {
    let size = (ls $path | get size | first)

    if $size > 1_000_000_000 {
        $"(($size / 1_000_000_000) | math round -p 2) GB"
    } else if $size > 1_000_000 {
        $"(($size / 1_000_000) | math round -p 2) MB"
    } else if $size > 1_000 {
        $"(($size / 1_000) | math round -p 2) KB"
    } else {
        $"($size) bytes"
    }
}
