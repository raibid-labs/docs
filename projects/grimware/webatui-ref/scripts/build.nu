#!/usr/bin/env nu

# build.nu - Comprehensive build automation for webatui project
#
# Usage:
#   nu scripts/build.nu [OPTIONS]
#
# Options:
#   --release         Build in release mode
#   --wasm            Build for WASM target
#   --debug           Build in debug mode (default)
#   --target TARGET   Specify target triple
#   --features FEATS  Enable specific features
#   --optimize        Optimize WASM output with wasm-opt
#   --pack            Use wasm-pack for building
#   --help            Show this help message

def main [
    --release         # Build in release mode
    --wasm            # Build for WASM target
    --debug           # Build in debug mode (default)
    --target: string  # Specify target triple
    --features: string # Enable specific features
    --optimize        # Optimize WASM with wasm-opt
    --pack            # Use wasm-pack for building
    --help            # Show help message
] {
    if $help {
        show_help
        return
    }

    print_header "🔨 Build Automation"

    # Determine build profile
    let profile = if $release { "release" } else { "debug" }
    print_info $"Build profile: ($profile)"

    # Build for WASM
    if $wasm {
        build_wasm $profile $optimize $pack
        return
    }

    # Build for specified target or native
    if $target != null {
        build_target $target $profile $features
    } else {
        build_native $profile $features
    }
}

# Build native target
def build_native [profile: string, features: string] {
    print_step "Building native target"

    let mut cmd = ["cargo", "build"]

    if $profile == "release" {
        $cmd = ($cmd | append "--release")
    }

    if $features != null {
        $cmd = ($cmd | append ["--features", $features])
    }

    print_command $cmd

    let result = (run-external $cmd.0 ...$cmd.1..)

    if $result == null or ($result | is-empty) {
        print_success "Native build completed!"
    } else {
        print_error "Build failed!"
        exit 1
    }
}

# Build for specific target
def build_target [target: string, profile: string, features: string] {
    print_step $"Building for target: ($target)"

    let mut cmd = ["cargo", "build", "--target", $target]

    if $profile == "release" {
        $cmd = ($cmd | append "--release")
    }

    if $features != null {
        $cmd = ($cmd | append ["--features", $features])
    }

    print_command $cmd

    let result = (run-external $cmd.0 ...$cmd.1..)

    if $result == null or ($result | is-empty) {
        print_success $"Build completed for ($target)!"
    } else {
        print_error "Build failed!"
        exit 1
    }
}

# Build WASM target
def build_wasm [profile: string, optimize: bool, pack: bool] {
    print_step "Building WASM target"

    if $pack {
        build_with_wasm_pack $profile
    } else {
        build_wasm_cargo $profile
    }

    if $optimize {
        optimize_wasm $profile
    }
}

# Build with cargo for WASM
def build_wasm_cargo [profile: string] {
    print_info "Using cargo build for WASM target"

    let mut cmd = ["cargo", "build", "--target", "wasm32-unknown-unknown"]

    if $profile == "release" {
        $cmd = ($cmd | append "--release")
    }

    print_command $cmd

    let result = (run-external $cmd.0 ...$cmd.1..)

    if $result == null or ($result | is-empty) {
        print_success "WASM build completed!"

        let wasm_path = if $profile == "release" {
            "target/wasm32-unknown-unknown/release"
        } else {
            "target/wasm32-unknown-unknown/debug"
        }

        print_info $"WASM artifacts in: ($wasm_path)"
    } else {
        print_error "WASM build failed!"
        exit 1
    }
}

# Build with wasm-pack
def build_with_wasm_pack [profile: string] {
    print_info "Using wasm-pack for building"

    # Check if wasm-pack is installed
    let wasm_pack_installed = (which wasm-pack | length) > 0

    if not $wasm_pack_installed {
        print_error "wasm-pack is not installed!"
        print_info "Install with: cargo install wasm-pack"
        exit 1
    }

    let mut cmd = ["wasm-pack", "build", "--target", "web", "--out-dir", "pkg"]

    if $profile == "release" {
        $cmd = ($cmd | append "--release")
    } else {
        $cmd = ($cmd | append "--dev")
    }

    print_command $cmd

    let result = (run-external $cmd.0 ...$cmd.1..)

    if $result == null or ($result | is-empty) {
        print_success "wasm-pack build completed!"
        print_info "WASM package in: pkg/"
    } else {
        print_error "wasm-pack build failed!"
        exit 1
    }
}

# Optimize WASM with wasm-opt
def optimize_wasm [profile: string] {
    print_step "Optimizing WASM bundle"

    # Check if wasm-opt is installed
    let wasm_opt_installed = (which wasm-opt | length) > 0

    if not $wasm_opt_installed {
        print_warning "wasm-opt is not installed! Skipping optimization."
        print_info "Install with: cargo install wasm-opt"
        return
    }

    # Find WASM files to optimize
    let wasm_files = (ls pkg/**/*_bg.wasm | get name)

    if ($wasm_files | length) == 0 {
        print_warning "No WASM files found to optimize!"
        return
    }

    for file in $wasm_files {
        let output = ($file | str replace "_bg.wasm" "_optimized_bg.wasm")

        print_info $"Optimizing: ($file)"

        let cmd = ["wasm-opt", "-Oz", "-o", $output, $file]
        print_command $cmd

        run-external $cmd.0 ...$cmd.1..

        # Show size comparison
        let original_size = (ls $file | get size | first)
        let optimized_size = (ls $output | get size | first)

        print_info $"Original size: ($original_size)"
        print_info $"Optimized size: ($optimized_size)"

        # Replace original with optimized
        mv -f $output $file

        print_success $"Optimized: ($file)"
    }
}

# Show help message
def show_help [] {
    print "
🔨 build.nu - Build automation for webatui

Usage:
    nu scripts/build.nu [OPTIONS]

Options:
    --release         Build in release mode (optimized)
    --debug           Build in debug mode (default)
    --wasm            Build for WASM target
    --target TARGET   Build for specific target triple
    --features FEATS  Enable specific features (comma-separated)
    --optimize        Optimize WASM output with wasm-opt
    --pack            Use wasm-pack for building WASM
    --help            Show this help message

Examples:
    # Build in debug mode
    nu scripts/build.nu

    # Build in release mode
    nu scripts/build.nu --release

    # Build WASM with wasm-pack
    nu scripts/build.nu --wasm --pack --release

    # Build WASM and optimize
    nu scripts/build.nu --wasm --release --optimize

    # Build for specific target
    nu scripts/build.nu --target x86_64-pc-windows-gnu --release
    "
}

# === Utility Functions ===

def print_header [msg: string] {
    print $"(ansi green_bold)($msg)(ansi reset)"
    print $"(ansi green){'=' * 50}(ansi reset)"
}

def print_step [msg: string] {
    print $"(ansi cyan_bold)▶ ($msg)(ansi reset)"
}

def print_info [msg: string] {
    print $"(ansi blue)ℹ ($msg)(ansi reset)"
}

def print_success [msg: string] {
    print $"(ansi green)✓ ($msg)(ansi reset)"
}

def print_error [msg: string] {
    print $"(ansi red)✗ ($msg)(ansi reset)"
}

def print_warning [msg: string] {
    print $"(ansi yellow)⚠ ($msg)(ansi reset)"
}

def print_command [cmd: list] {
    print $"(ansi purple)$ ($cmd | str join ' ')(ansi reset)"
}
