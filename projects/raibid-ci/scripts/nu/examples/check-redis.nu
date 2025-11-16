#!/usr/bin/env nu
# Raibid-CI Redis Health Check Example
# Demonstrates using redis module to check Redis health

# Source environment and config
source ../env.nu
source ../config.nu

# Import redis module
use ../modules/redis.nu *

def main [] {
    print "=== Raibid-CI Redis Health Check ==="
    print ""

    let redis_url = if "RAIBID_REDIS_URL" in $env {
        $env.RAIBID_REDIS_URL
    } else {
        "redis://localhost:6379"
    }

    print $"Redis URL: ($redis_url)"
    print ""

    # Check connection
    if not (redis-check-connection $redis_url) {
        log-error "Cannot connect to Redis. Is Redis running?"
        exit 1
    }

    print ""

    # Get memory usage
    log-info "Memory Usage:"
    let memory = (redis-memory)
    print $"  Used memory: ($memory)"

    print ""

    # Get server info
    log-info "Server Information:"
    redis-info $redis_url "server" | lines | each {|line|
        if ($line | str contains ":") and not ($line | str starts-with "#") {
            print $"  ($line)"
        }
    }

    print ""

    # Check for job stream
    let stream_name = if "RAIBID_REDIS_STREAM" in $env {
        $env.RAIBID_REDIS_STREAM
    } else {
        "raibid:jobs"
    }

    log-info $"Job Stream: ($stream_name)"

    try {
        let stream_len = (redis-stream-len $stream_name $redis_url)
        print $"  Stream length: ($stream_len) entries"

        if $stream_len > 0 {
            log-info "Stream has pending jobs"
        } else {
            log-info "Stream is empty"
        }
    } catch {
        log-warning $"Stream ($stream_name) does not exist yet"
    }

    print ""

    # List keys
    log-info "Keys in database:"
    let keys = (redis-keys "*" $redis_url)

    if ($keys | length) > 0 {
        $keys | each {|key| print $"  ($key)"}
    } else {
        print "  (no keys)"
    }

    print ""
    log-success "Redis health check complete!"
}

main
