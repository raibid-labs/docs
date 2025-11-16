#!/usr/bin/env nu
# Raibid-CI Nushell Setup Validation
# Validates that Nushell environment is properly configured
#
# Usage: nu scripts/nu/validate-setup.nu

def main [] {
    print "=== Raibid-CI Nushell Setup Validation ==="
    print ""

    mut all_passed = true

    # Check Nushell version
    print "1. Checking Nushell version..."
    let version_ok = check-nushell-version
    if $version_ok {
        print $"   (ansi green)✓(ansi reset) Version check passed"
    } else {
        print $"   (ansi red)✗(ansi reset) Version check failed"
        $all_passed = false
    }
    print ""

    # Check file structure
    print "2. Checking file structure..."
    let files_ok = check-file-structure
    if $files_ok {
        print $"   (ansi green)✓(ansi reset) File structure check passed"
    } else {
        print $"   (ansi red)✗(ansi reset) File structure check failed"
        $all_passed = false
    }
    print ""

    # Check modules syntax
    print "3. Checking modules..."
    let modules_ok = check-modules
    if $modules_ok {
        print $"   (ansi green)✓(ansi reset) Module check passed"
    } else {
        print $"   (ansi red)✗(ansi reset) Module check failed"
        $all_passed = false
    }
    print ""

    # Check examples syntax
    print "4. Checking example scripts..."
    let examples_ok = check-examples
    if $examples_ok {
        print $"   (ansi green)✓(ansi reset) Examples check passed"
    } else {
        print $"   (ansi red)✗(ansi reset) Examples check failed"
        $all_passed = false
    }
    print ""

    # Summary
    print "=== Validation Summary ==="
    if $all_passed {
        print $"(ansi green)✓ All checks passed!(ansi reset)"
        print ""
        print "Nushell environment is properly configured."
        print "You can now use Nushell scripts and modules."
        print ""
        print "To load the environment, run:"
        print "  source scripts/nu/env.nu"
        print "  source scripts/nu/config.nu"
        print ""
        print "Then try running:"
        print "  nu scripts/nu/examples/dev-workflow.nu --status"
        exit 0
    } else {
        print $"(ansi red)✗ Some checks failed.(ansi reset)"
        print ""
        print "Please fix the issues above and run validation again."
        exit 1
    }
}

def check-nushell-version [] {
    let nu_version = (version | get version)
    let parts = ($nu_version | split row '.')
    let major = ($parts | first | into int)
    let minor = ($parts | get 1 | into int)

    print $"   Nushell version: ($nu_version)"

    if $major > 0 or ($major == 0 and $minor >= 96) {
        true
    } else {
        print $"   (ansi red)Error:(ansi reset) Nushell 0.96 or later is required"
        false
    }
}

def check-file-structure [] {
    let project_root = ($env.FILE_PWD | path dirname | path dirname)

    let required_files = [
        "scripts/nu/config.nu"
        "scripts/nu/env.nu"
        "scripts/nu/README.md"
        "scripts/nu/modules/kubectl.nu"
        "scripts/nu/modules/redis.nu"
        "scripts/nu/modules/gitea.nu"
        "scripts/nu/examples/check-cluster.nu"
        "scripts/nu/examples/check-redis.nu"
        "scripts/nu/examples/check-gitea.nu"
        "scripts/nu/examples/dev-workflow.nu"
        "docs/guides/nushell.md"
    ]

    let results = ($required_files | each {|file|
        let file_path = ($project_root | path join $file)
        if ($file_path | path exists) {
            print $"   ✓ ($file)"
            true
        } else {
            print $"   (ansi red)✗(ansi reset) ($file) - missing"
            false
        }
    })

    $results | all {|r| $r == true}
}

def check-modules [] {
    let project_root = ($env.FILE_PWD | path dirname | path dirname)

    let modules = ["kubectl" "redis" "gitea"]

    let results = ($modules | each {|module|
        let module_path = ($project_root | path join "scripts" "nu" "modules" $"($module).nu")

        # Use --ide-check for syntax validation without execution
        let result = (nu --ide-check 0 $module_path | complete)
        if $result.exit_code == 0 {
            print $"   ✓ ($module).nu syntax valid"
            true
        } else {
            print $"   (ansi red)✗(ansi reset) ($module).nu - syntax error"
            if ($result.stderr | str length) > 0 {
                print $"      ($result.stderr)"
            }
            false
        }
    })

    $results | all {|r| $r == true}
}

def check-examples [] {
    let project_root = ($env.FILE_PWD | path dirname | path dirname)

    let examples = ["check-cluster.nu" "check-redis.nu" "check-gitea.nu" "dev-workflow.nu"]

    let results = ($examples | each {|example|
        let example_path = ($project_root | path join "scripts" "nu" "examples" $example)

        # Check if executable
        let perms = (ls -l $example_path | get 0.mode)
        if not ($perms | str contains "x") {
            print $"   (ansi yellow)⚠(ansi reset) ($example) - not executable (run: chmod +x)"
        }

        # Use --ide-check for syntax validation without execution
        let result = (nu --ide-check 0 $example_path | complete)
        if $result.exit_code == 0 {
            print $"   ✓ ($example) syntax valid"
            true
        } else {
            print $"   (ansi red)✗(ansi reset) ($example) - syntax error"
            if ($result.stderr | str length) > 0 {
                print $"      ($result.stderr)"
            }
            false
        }
    })

    $results | all {|r| $r == true}
}

main
