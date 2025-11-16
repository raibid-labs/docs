#!/usr/bin/env nu

# serve.nu - HTTP server for serving WASM builds locally
#
# Usage:
#   nu scripts/serve.nu [OPTIONS]
#
# Options:
#   --port PORT       Port to serve on (default: 8080)
#   --dir DIR         Directory to serve (default: pkg)
#   --host HOST       Host to bind to (default: 0.0.0.0)
#   --open            Open browser after starting
#   --cors            Enable CORS headers
#   --spa             Enable SPA mode (fallback to index.html)
#   --help            Show this help message

def main [
    --port: int = 8080    # Port to serve on
    --dir: string = "pkg" # Directory to serve
    --host: string = "0.0.0.0" # Host to bind to
    --open                # Open browser after starting
    --cors                # Enable CORS headers
    --spa                 # Enable SPA mode
    --help                # Show help message
] {
    if $help {
        show_help
        return
    }

    print_header "🌐 HTTP Server for WASM"

    # Check if directory exists
    if not ($dir | path exists) {
        print_error $"Directory not found: ($dir)"
        print_info "Build your project first with: just build-wasm"
        exit 1
    }

    # Determine which server to use
    let server_type = detect_server

    match $server_type {
        "python3" => { serve_python3 $port $dir $host $open $cors $spa }
        "python" => { serve_python $port $dir $host $open $cors $spa }
        "basic-http-server" => { serve_basic_http $port $dir $host $open $cors }
        "http-server" => { serve_http_server $port $dir $host $open $cors $spa }
        _ => {
            print_error "No suitable HTTP server found!"
            print_info "Install one of the following:"
            print_info "  - Python 3: built-in http.server"
            print_info "  - basic-http-server: cargo install basic-http-server"
            print_info "  - http-server: npm install -g http-server"
            exit 1
        }
    }
}

# Detect available HTTP server
def detect_server [] -> string {
    if (which python3 | length) > 0 {
        return "python3"
    } else if (which python | length) > 0 {
        return "python"
    } else if (which basic-http-server | length) > 0 {
        return "basic-http-server"
    } else if (which http-server | length) > 0 {
        return "http-server"
    } else {
        return "none"
    }
}

# Serve with Python 3
def serve_python3 [port: int, dir: string, host: string, open_browser: bool, cors: bool, spa: bool] {
    print_info $"Using Python 3 http.server"
    print_step $"Serving ($dir) on http://($host):($port)"

    if $open_browser {
        # Open browser in background
        let url = $"http://localhost:($port)"
        print_info $"Opening browser: ($url)"

        if $nu.os-info.name == "macos" {
            run-external "open" $url
        } else if $nu.os-info.name == "linux" {
            run-external "xdg-open" $url
        } else if $nu.os-info.name == "windows" {
            run-external "start" $url
        }
    }

    print_success "Server started! Press Ctrl+C to stop."
    print_info "Serving files from: $dir"

    cd $dir

    if $cors {
        # Create a simple CORS-enabled Python server
        let script = "
from http.server import HTTPServer, SimpleHTTPRequestHandler
import sys

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', '*')
        self.send_header('Cache-Control', 'no-store, no-cache, must-revalidate')
        return super().end_headers()

httpd = HTTPServer(('" + $host + "', " + ($port | into string) + "), CORSRequestHandler)
httpd.serve_forever()
"
        print_warning "CORS enabled - suitable for development only!"
        run-external "python3" "-c" $script
    } else {
        run-external "python3" "-m" "http.server" ($port | into string) "--bind" $host
    }
}

# Serve with Python 2
def serve_python [port: int, dir: string, host: string, open_browser: bool, cors: bool, spa: bool] {
    print_info "Using Python 2 SimpleHTTPServer"
    print_step $"Serving ($dir) on http://($host):($port)"

    if $open_browser {
        let url = $"http://localhost:($port)"
        print_info $"Opening browser: ($url)"

        if $nu.os-info.name == "macos" {
            run-external "open" $url
        } else if $nu.os-info.name == "linux" {
            run-external "xdg-open" $url
        }
    }

    print_success "Server started! Press Ctrl+C to stop."

    cd $dir
    run-external "python" "-m" "SimpleHTTPServer" ($port | into string)
}

# Serve with basic-http-server
def serve_basic_http [port: int, dir: string, host: string, open_browser: bool, cors: bool] {
    print_info "Using basic-http-server"
    print_step $"Serving ($dir) on http://($host):($port)"

    let mut cmd = ["basic-http-server", $dir, "-a", $"($host):($port)"]

    if $cors {
        $cmd = ($cmd | append "--cors")
        print_warning "CORS enabled - suitable for development only!"
    }

    if $open_browser {
        let url = $"http://localhost:($port)"
        print_info $"Opening browser: ($url)"

        if $nu.os-info.name == "macos" {
            run-external "open" $url
        } else if $nu.os-info.name == "linux" {
            run-external "xdg-open" $url
        }
    }

    print_success "Server started! Press Ctrl+C to stop."
    print_command $cmd

    run-external $cmd.0 ...$cmd.1..
}

# Serve with http-server (Node.js)
def serve_http_server [port: int, dir: string, host: string, open_browser: bool, cors: bool, spa: bool] {
    print_info "Using http-server (Node.js)"
    print_step $"Serving ($dir) on http://($host):($port)"

    let mut cmd = ["http-server", $dir, "-p", ($port | into string), "-a", $host]

    if $cors {
        $cmd = ($cmd | append "--cors")
        print_warning "CORS enabled - suitable for development only!"
    }

    if $spa {
        $cmd = ($cmd | append "--proxy" $"http://($host):($port)?")
        print_info "SPA mode enabled - fallback to index.html"
    }

    if $open_browser {
        $cmd = ($cmd | append "-o")
    }

    # Disable caching
    $cmd = ($cmd | append ["-c-1"])

    print_success "Server started! Press Ctrl+C to stop."
    print_command $cmd

    run-external $cmd.0 ...$cmd.1..
}

# Show help message
def show_help [] {
    print "
🌐 serve.nu - HTTP server for WASM builds

Usage:
    nu scripts/serve.nu [OPTIONS]

Options:
    --port PORT       Port to serve on (default: 8080)
    --dir DIR         Directory to serve (default: pkg)
    --host HOST       Host to bind to (default: 0.0.0.0)
    --open            Open browser after starting
    --cors            Enable CORS headers
    --spa             Enable SPA mode (fallback to index.html)
    --help            Show this help message

Examples:
    # Serve default directory on port 8080
    nu scripts/serve.nu

    # Serve on custom port
    nu scripts/serve.nu --port 3000

    # Serve specific directory
    nu scripts/serve.nu --dir www

    # Open browser automatically
    nu scripts/serve.nu --open

    # Enable CORS for cross-origin requests
    nu scripts/serve.nu --cors

    # Enable SPA mode
    nu scripts/serve.nu --spa --open

Detected servers:
"
    let server = detect_server
    if $server != "none" {
        print $"    ✓ ($server) (will be used)"
    } else {
        print "    ✗ No HTTP server detected"
        print ""
        print "Install one of:"
        print "    - Python 3 (built-in)"
        print "    - cargo install basic-http-server"
        print "    - npm install -g http-server"
    }
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
