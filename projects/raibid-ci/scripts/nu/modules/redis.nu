# Raibid-CI Redis Utility Module
# Helper functions for interacting with Redis and Redis Streams
#
# Usage:
#   use redis.nu *
#   redis-check

# Check if redis-cli is available
export def redis-check [] {
    if (which redis-cli | is-empty) {
        error make {msg: "redis-cli not found. Please install Redis."}
    }
}

# Check Redis connectivity
export def redis-check-connection [
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check

    print "Checking Redis connection..."

    try {
        let result = (redis-cli -u $url PING | complete)
        if $result.exit_code == 0 and ($result.stdout | str trim) == "PONG" {
            print $"(ansi green)✓(ansi reset) Connected to Redis"
            return true
        } else {
            print $"(ansi red)✗(ansi reset) Cannot connect to Redis"
            return false
        }
    } catch {
        print $"(ansi red)✗(ansi reset) Error connecting to Redis"
        return false
    }
}

# Get Redis info
export def redis-info [
    url: string = "redis://localhost:6379"  # Redis URL
    section: string = "all"  # Info section (server, clients, memory, etc.)
] {
    redis-check

    if $section == "all" {
        redis-cli -u $url INFO
    } else {
        redis-cli -u $url INFO $section
    }
}

# Get Redis memory usage
export def redis-memory [] {
    redis-check

    redis-cli INFO memory |
        lines |
        where {|line| $line =~ "used_memory_human"} |
        each {|line| $line | str replace "used_memory_human:" "" | str trim}
}

# Ping Redis
export def redis-ping [
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url PING
}

# Get all keys matching pattern
export def redis-keys [
    pattern: string = "*"  # Key pattern
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url KEYS $pattern | lines
}

# Get value of a key
export def redis-get [
    key: string  # Key to retrieve
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url GET $key
}

# Set a key-value pair
export def redis-set [
    key: string   # Key to set
    value: string  # Value to set
    url: string = "redis://localhost:6379"  # Redis URL
    --expire (-e): int  # Expiration in seconds
] {
    redis-check

    if $expire != null {
        redis-cli -u $url SET $key $value EX ($expire | into string)
    } else {
        redis-cli -u $url SET $key $value
    }
}

# Delete a key
export def redis-del [
    key: string  # Key to delete
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url DEL $key
}

# Add entry to Redis Stream
export def redis-stream-add [
    stream: string  # Stream name
    data: record    # Data to add (key-value pairs)
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check

    # Build XADD command arguments
    mut args = ["XADD", $stream, "*"]

    # Add key-value pairs
    for kv in ($data | transpose key value) {
        $args = ($args | append [$kv.key, ($kv.value | into string)])
    }

    redis-cli -u $url ...$args
}

# Read from Redis Stream
export def redis-stream-read [
    stream: string  # Stream name
    url: string = "redis://localhost:6379"  # Redis URL
    --count (-c): int = 10  # Number of entries to read
    --block (-b): int  # Block for N milliseconds
] {
    redis-check

    mut args = ["XREAD", "COUNT", ($count | into string)]

    if $block != null {
        $args = ($args | append ["BLOCK", ($block | into string)])
    }

    $args = ($args | append ["STREAMS", $stream, "0"])

    redis-cli -u $url ...$args
}

# Get stream length
export def redis-stream-len [
    stream: string  # Stream name
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url XLEN $stream | into int
}

# Get stream info
export def redis-stream-info [
    stream: string  # Stream name
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url XINFO STREAM $stream
}

# Create consumer group
export def redis-stream-create-group [
    stream: string  # Stream name
    group: string   # Consumer group name
    url: string = "redis://localhost:6379"  # Redis URL
    --from-start    # Start from beginning of stream
] {
    redis-check

    let start_id = if $from_start { "0" } else { "$" }

    try {
        redis-cli -u $url XGROUP CREATE $stream $group $start_id
        print $"(ansi green)✓(ansi reset) Consumer group created: ($group)"
    } catch {
        print $"(ansi yellow)⚠(ansi reset) Consumer group may already exist: ($group)"
    }
}

# Read from consumer group
export def redis-stream-read-group [
    stream: string   # Stream name
    group: string    # Consumer group name
    consumer: string # Consumer name
    url: string = "redis://localhost:6379"  # Redis URL
    --count (-c): int = 10  # Number of entries to read
    --block (-b): int  # Block for N milliseconds
] {
    redis-check

    mut args = ["XREADGROUP", "GROUP", $group, $consumer, "COUNT", ($count | into string)]

    if $block != null {
        $args = ($args | append ["BLOCK", ($block | into string)])
    }

    $args = ($args | append ["STREAMS", $stream, ">"])

    redis-cli -u $url ...$args
}

# Acknowledge stream message
export def redis-stream-ack [
    stream: string  # Stream name
    group: string   # Consumer group name
    id: string      # Message ID
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url XACK $stream $group $id
}

# Monitor pending messages in consumer group
export def redis-stream-pending [
    stream: string  # Stream name
    group: string   # Consumer group name
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    redis-cli -u $url XPENDING $stream $group
}

# Flush all data (DANGER!)
export def redis-flush-all [
    url: string = "redis://localhost:6379"  # Redis URL
    --confirm  # Confirmation flag
] {
    if not $confirm {
        print $"(ansi red)⚠ WARNING:(ansi reset) This will delete ALL data in Redis!"
        print "Use --confirm flag if you're sure."
        return
    }

    redis-check
    print "Flushing all Redis data..."
    redis-cli -u $url FLUSHALL
    print $"(ansi green)✓(ansi reset) Redis flushed"
}

# Monitor Redis commands (real-time)
export def redis-monitor [
    url: string = "redis://localhost:6379"  # Redis URL
] {
    redis-check
    print "Monitoring Redis commands (Ctrl+C to stop)..."
    redis-cli -u $url MONITOR
}

# Get Redis statistics
export def redis-stats [] {
    redis-check

    print "\nRedis Statistics:"
    print "================\n"

    # Get basic stats
    let info = (redis-cli INFO stats | lines)

    for line in $info {
        if ($line | str contains ":") and not ($line | str starts-with "#") {
            print $line
        }
    }
}
