#!/usr/bin/env nu

# Validate raibid-cli project structure and conventions

def main [] {
    print "=== Validating raibid-cli project structure ==="

    let checks = [
        (check_file "Cargo.toml" "Workspace manifest"),
        (check_file "justfile" "Build automation"),
        (check_file "README.md" "Project documentation"),
        (check_directory "src" "Source code"),
        (check_directory "crates/raibid-core" "Core library"),
        (check_directory "docs" "Documentation"),
        (check_directory "scripts" "Automation scripts"),
        (check_rust_files),
        (check_docs_structure),
    ]

    let passed = ($checks | where status == "✅" | length)
    let failed = ($checks | where status == "❌" | length)

    print "\n=== Validation Summary ==="
    $checks | table

    print $"\nPassed: ($passed), Failed: ($failed)"

    if $failed > 0 {
        exit 1
    }
}

def check_file [path: string, description: string] {
    if ($path | path exists) {
        {status: "✅", check: $description, path: $path}
    } else {
        {status: "❌", check: $description, path: $path}
    }
}

def check_directory [path: string, description: string] {
    if ($path | path exists) and (ls $path | length) > 0 {
        {status: "✅", check: $description, path: $path}
    } else {
        {status: "❌", check: $description, path: $path}
    }
}

def check_rust_files [] {
    let rust_files = (ls **/*.rs | length)
    if $rust_files > 0 {
        {status: "✅", check: $"Rust source files \(($rust_files)\)", path: "**/*.rs"}
    } else {
        {status: "❌", check: "Rust source files", path: "**/*.rs"}
    }
}

def check_docs_structure [] {
    let required_docs = [
        "docs/architecture.md",
        "docs/roadmap.md",
        "docs/research-git-sync.md",
        "docs/research-raibid-labs-org.md",
    ]

    let missing = ($required_docs | where { |doc| not ($doc | path exists) })

    if ($missing | length) == 0 {
        {status: "✅", check: "Required documentation files", path: "docs/"}
    } else {
        {status: "❌", check: $"Missing docs: ($missing | str join ', ')", path: "docs/"}
    }
}
