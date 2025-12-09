#!/usr/bin/env nu
# Rust Documentation Generator for Documentation Hub
#
# Purpose: Generate unified rustdoc JSON from all Rust projects in the org
# Usage: nu scripts/build-rustdoc.nu [--output path] [--verbose]

# Configuration
let config = {
  projects_dir: "content/projects",
  output_file: "public/rustdoc.json",
  output_format: "json",
}

# Parse command line arguments
def parse_args [] {
  let args = $env.ARGS.positional
  let flags = $env.ARGS.named
  
  {
    output: (($flags | get -o output) | get 0? or $config.output_file),
    verbose: ($flags | get -o verbose | is-not-empty),
    help: ($flags | get -o help | is-not-empty),
  }
}

# Main function
def main [] {
  let opts = parse_args
  
  # Show help
  if $opts.help {
    print "Rust Documentation Generator
    
Usage: nu scripts/build-rustdoc.nu [OPTIONS]

Options:
  --output <path>     Output file path (default: public/rustdoc.json)
  --verbose          Show detailed progress
  --help             Show this help message"
    exit 0
  }
  
  # Create output directory
  mkdir ($opts.output | path dirname)
  
  if $opts.verbose {
    print $"📚 Starting Rust documentation build..."
    print $"📁 Projects directory: ($config.projects_dir)"
    print $"📝 Output file: ($opts.output)"
  }
  
  # Find all Rust projects (with Cargo.toml)
  let projects = (
    glob $"($config.projects_dir)/*/Cargo.toml"
    | each { |cargo_file|
      {
        path: ($cargo_file | path dirname),
        name: ($cargo_file | path dirname | path basename),
      }
    }
  )
  
  if ($projects | is-empty) {
    if $opts.verbose {
      print "⚠️  No Rust projects found with Cargo.toml"
    }
    # Create empty rustdoc structure
    let empty_doc = {
      crate_version: null,
      includes: [],
      index: {},
      paths: {},
      root: null,
    }
    $empty_doc | to json | save --force $opts.output
    exit 0
  }
  
  if $opts.verbose {
    print $"🔍 Found ($projects | length) Rust project(s)"
  }
  
  # Generate rustdoc JSON for each project
  let all_docs = []
  mut merged_index = {}
  mut merged_paths = {}
  mut all_crates = []
  
  for project in $projects {
    if $opts.verbose {
      print $"  📦 Building docs for: ($project.name)"
    }
    
    try {
      # Generate rustdoc JSON with --output-format json
      # This requires Rust with cargo installed
      cd $project.path

      # Build rustdoc JSON output
      # Note: --output-format json is available in recent Rust versions
      let result = (do { cargo doc --output-format json --no-deps } | complete)

      if ($result.exit_code != 0) {
        if $opts.verbose {
          print $"    ❌ Failed to generate docs for ($project.name): ($result.stderr | str trim)"
        }
        continue
      }

      # Look for target/doc/*.json files
      let json_files = try { glob "target/doc/*.json" } catch { [] }
      
      if ($json_files | is-empty) {
        if $opts.verbose {
          print $"    ℹ️  No JSON output found (may not be in this Rust version)"
        }
        continue
      }
      
      # Read and merge documentation
      for json_file in $json_files {
        try {
          let doc_data = (open $json_file)
          
          # Merge crate information
          $all_crates = ($all_crates | append $project.name)
          
          # Merge index and paths
          if ($doc_data | get -o index | is-not-empty) {
            $merged_index = ($merged_index | merge ($doc_data.index))
          }
          if ($doc_data | get -o paths | is-not-empty) {
            $merged_paths = ($merged_paths | merge ($doc_data.paths))
          }
          
          if $opts.verbose {
            print $"    ✅ Merged documentation from ($json_file)"
          }
        } catch { |err|
          if $opts.verbose {
            print $"    ⚠️  Error reading ($json_file): ($err.msg)"
          }
        }
      }
      
      cd -
    } catch { |err|
      if $opts.verbose {
        print $"    ❌ Error building docs for ($project.name): ($err.msg)"
      }
      cd -
    }
  }
  
  # Build final rustdoc JSON structure
  let rustdoc_output = {
    crate_version: "0.1.0",
    includes: [],
    index: $merged_index,
    paths: $merged_paths,
    root: null,
    format_version: 28,  # Current rustdoc JSON format version
  }
  
  # Save the merged documentation
  $rustdoc_output | to json | save --force $opts.output
  
  if $opts.verbose {
    let output_size = if ($opts.output | path exists) {
      let file_size = (ls $opts.output | get 0?.size? | default 0)
      $"($file_size) bytes"
    } else {
      "0 bytes"
    }
    let stats = {
      total_items: ($merged_index | columns | length),
      crates: ($all_crates | length),
      output_size: $output_size,
    }
    print ""
    print $"✅ Documentation generation completed!"
    print $"📊 Statistics:"
    print $"  - Total items indexed: ($stats.total_items)"
    print $"  - Crates processed: ($stats.crates)"
    print $"  - Output file: ($opts.output)"
    print $"  - File size: ($stats.output_size)"
  }
}

main
