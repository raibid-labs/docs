#!/bin/bash
# record-videos.sh - Record video screencasts for Scarab Terminal
#
# Usage: ./scripts/record-videos.sh
#
# Prerequisites:
#   - asciinema (pip install asciinema)
#   - asciicast2mp4 (npm install -g asciicast2mp4)
#   OR
#   - agg (cargo install agg) for simpler conversion

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIDEO_DIR="$PROJECT_ROOT/docs/videos"
CAST_DIR="$VIDEO_DIR/casts"

# Create directories
mkdir -p "$VIDEO_DIR"
mkdir -p "$CAST_DIR"

echo "Recording Scarab Terminal Videos"
echo "================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check for dependencies
check_deps() {
    if ! command -v asciinema &> /dev/null; then
        echo -e "${YELLOW}Warning: asciinema not found${NC}"
        echo "Install: pip install asciinema"
        exit 1
    fi

    if ! command -v asciicast2mp4 &> /dev/null && ! command -v agg &> /dev/null; then
        echo -e "${YELLOW}Warning: No video converter found${NC}"
        echo "Install one of:"
        echo "  - npm install -g asciicast2mp4"
        echo "  - cargo install agg"
        exit 1
    fi
}

check_deps

# Function to record video
record_video() {
    local name="$1"
    local script="$2"

    echo -e "${BLUE}Recording: $name${NC}"

    # Create script file
    cat > "$CAST_DIR/$name.sh" << EOF
#!/bin/bash
$script
EOF

    chmod +x "$CAST_DIR/$name.sh"

    # Record with asciinema
    echo "Starting recording in 3 seconds..."
    sleep 3
    asciinema rec "$CAST_DIR/$name.cast" -c "$CAST_DIR/$name.sh" --overwrite

    # Convert to MP4
    if command -v asciicast2mp4 &> /dev/null; then
        echo "Converting with asciicast2mp4..."
        asciicast2mp4 -t dracula -s 2 "$CAST_DIR/$name.cast" "$VIDEO_DIR/$name.mp4"
    elif command -v agg &> /dev/null; then
        echo "Converting with agg..."
        # Note: agg creates GIF by default, but can be used for video frames
        echo "Using agg to create high-quality GIF (similar to video)..."
        agg "$CAST_DIR/$name.cast" "$VIDEO_DIR/$name.gif" \
            --theme dracula \
            --font-family "JetBrains Mono" \
            --font-size 14 \
            --cols 120 \
            --rows 30
        echo "Note: Created GIF instead of MP4 (agg limitation)"
        echo "To convert to MP4, install asciicast2mp4 or use ffmpeg"
    fi

    echo -e "${GREEN}✓ Created: $VIDEO_DIR/$name.mp4 (or .gif)${NC}"
    echo ""
}

# Video 1: Scarab in 2 Minutes
echo -e "${BLUE}=== Video 1: Scarab in 2 Minutes ===${NC}"
record_video "scarab-2min-demo" '
# Intro
clear
cat << "INTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║         SCARAB TERMINAL                        ║
║                                                ║
║    Next-Generation GPU-Accelerated Terminal    ║
║                                                ║
║    github.com/raibid-labs/scarab               ║
║                                                ║
╚════════════════════════════════════════════════╝
INTRO
sleep 4
clear

# Feature 1: GPU Rendering
sleep 1
echo "1. GPU-Accelerated Rendering"
echo "============================"
sleep 1
echo ""
echo "Built with Bevy game engine for smooth 60+ FPS"
sleep 2
echo ""
for i in {1..30}; do
    echo "Line $i: Buttery smooth scrolling with GPU acceleration ✨"
    sleep 0.05
done
sleep 2
clear

# Feature 2: Link Hints
sleep 1
echo "2. Keyboard-Driven Link Hints"
echo "=============================="
sleep 1
echo ""
echo "Open URLs without touching your mouse:"
sleep 1
echo ""
echo "  GitHub: https://github.com/raibid-labs/scarab"
sleep 1
echo "  Docs: https://scarab.dev/documentation"
sleep 1
echo "  Issues: https://github.com/raibid-labs/scarab/issues"
sleep 1
echo ""
echo "Press Ctrl+Shift+O → [a] [b] [c] → Opens in browser"
sleep 3
clear

# Feature 3: Command Palette
sleep 1
echo "3. Command Palette (Ctrl+Shift+P)"
echo "=================================="
sleep 2
cat << "PALETTE"

┌──────────────────────────────────────────────┐
│ Command Palette                     Ctrl+P   │
├──────────────────────────────────────────────┤
│ >                                            │
│                                              │
│  session new      Create new session         │
│  theme switch     Change color theme         │
│  plugin reload    Reload all plugins         │
│  config edit      Open configuration         │
│                                              │
└──────────────────────────────────────────────┘

Fuzzy search: Just type what you want!
PALETTE
sleep 4
clear

# Feature 4: F# Plugins
sleep 1
echo "4. F# Plugin System (Fusabi)"
echo "============================"
sleep 1
echo ""
echo "Write powerful plugins in F#:"
sleep 1
cat << "PLUGIN"

// git-status.fsx
open Scarab.PluginApi

let on_load ctx = async {
    let branch = getCurrentBranch()
    ctx.DrawOverlay {
        Text = sprintf "git: %s" branch
        Position = TopRight
        Style = GreenBackground
    }
    return Ok ()
}
PLUGIN
sleep 4
clear

# Feature 5: Split Architecture
sleep 1
echo "5. Split-Process Architecture"
echo "============================="
sleep 1
echo ""
echo "Daemon (server)  →  Zero-Copy IPC  →  Client (GUI)"
echo "       ↓                                    ↓"
echo "   PTY processes              Bevy rendering"
sleep 2
echo ""
echo "Benefits:"
echo "  ✓ Client crashes dont lose your work"
echo "  ✓ Multiple clients can attach"
echo "  ✓ Session persistence"
echo "  ✓ Zero-copy shared memory = blazing fast"
sleep 4
clear

# Outro
sleep 1
cat << "OUTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║    Get Started Now:                            ║
║                                                ║
║    $ git clone github.com/raibid-labs/scarab   ║
║    $ cargo build --release                     ║
║    $ ./target/release/scarab-daemon &          ║
║    $ ./target/release/scarab-client            ║
║                                                ║
║    ⭐ Star us on GitHub!                       ║
║                                                ║
╚════════════════════════════════════════════════╝
OUTRO
sleep 5
'

# Video 2: Your First Plugin
echo -e "${BLUE}=== Video 2: Your First Plugin ===${NC}"
record_video "first-plugin-tutorial" '
# Intro
clear
cat << "INTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║         YOUR FIRST SCARAB PLUGIN               ║
║                                                ║
║    Learn to create F# plugins in 5 minutes    ║
║                                                ║
╚════════════════════════════════════════════════╝
INTRO
sleep 3
clear

# Setup
sleep 1
echo "Prerequisites:"
echo "============="
sleep 1
echo "  ✓ Scarab installed and running"
echo "  ✓ Text editor (vim, emacs, VS Code, etc.)"
echo "  ✓ Basic F# knowledge (we will guide you!)"
sleep 3
clear

# Step 1
sleep 1
echo "Step 1: Create Plugin Directory"
echo "================================"
sleep 1
echo ""
echo "$ mkdir -p ~/.config/scarab/plugins"
echo "$ cd ~/.config/scarab/plugins"
sleep 2
echo ""
echo "✓ Directory created"
sleep 2
clear

# Step 2
sleep 1
echo "Step 2: Create hello-world.fsx"
echo "=============================="
sleep 1
echo ""
echo "$ vim hello-world.fsx"
sleep 1
echo ""
echo "Writing plugin code..."
sleep 1
cat << "PLUGIN"

(*
 * hello-world.fsx
 * A simple Scarab plugin
 *)

open Scarab.PluginApi

// 1. Define metadata
let metadata = {
    Name = "hello-world"
    Version = "1.0.0"
    Description = "My first Scarab plugin"
    Author = "Your Name"
    Homepage = None
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// 2. Define load handler
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Log a message
        ctx.Log(LogLevel.Info, "Hello from my plugin!")

        // Get terminal dimensions
        let (cols, rows) = ctx.GetSize()
        ctx.Log(LogLevel.Info,
            sprintf "Terminal size: %d × %d" cols rows)

        return Ok ()
    }

// 3. Register the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = Some on_load
    OnUnload = None
    OnOutput = None
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = None
    GetCommands = fun () -> []
}
PLUGIN
sleep 8
clear

# Step 3
sleep 1
echo "Step 3: Load the Plugin"
echo "======================="
sleep 1
echo ""
echo "Restart Scarab daemon to load plugins:"
sleep 1
echo ""
echo "$ pkill scarab-daemon"
sleep 1
echo "$ scarab-daemon &"
sleep 2
echo ""
echo "[2024-11-24 12:00:00] Scarab Daemon starting..."
sleep 1
echo "[2024-11-24 12:00:01] Loading plugins from ~/.config/scarab/plugins/"
sleep 1
echo "[2024-11-24 12:00:02] Found plugin: hello-world.fsx"
sleep 1
echo "[2024-11-24 12:00:03] Compiling plugin..."
sleep 1
echo "[2024-11-24 12:00:04] Plugin loaded successfully!"
sleep 1
echo "[2024-11-24 12:00:04] [hello-world] Hello from my plugin!"
sleep 1
echo "[2024-11-24 12:00:04] [hello-world] Terminal size: 120 × 30"
sleep 3
clear

# Step 4: Add functionality
sleep 1
echo "Step 4: Add More Features"
echo "========================="
sleep 1
echo ""
echo "Lets add a command that shows the current time:"
sleep 2
cat << "PLUGIN2"

// Add to hello-world.fsx:

let get_time_command (ctx: PluginContext) : Command =
    {
        Name = "show-time"
        Description = "Display current time"
        Handler = fun () -> async {
            let now = System.DateTime.Now
            let time = now.ToString("HH:mm:ss")

            ctx.Log(LogLevel.Info,
                sprintf "Current time: %s" time)

            return Ok ()
        }
    }

Plugin.Register {
    // ... same as before ...
    GetCommands = fun () -> [get_time_command ctx]
}
PLUGIN2
sleep 6
echo ""
echo "Now use: Ctrl+Shift+P → show-time"
sleep 3
clear

# Outro
sleep 1
cat << "OUTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║    Congratulations! 🎉                         ║
║                                                ║
║    You created your first Scarab plugin!       ║
║                                                ║
║    Next Steps:                                 ║
║    • Explore plugin hooks (OnOutput, etc.)     ║
║    • Check examples/plugins/ for inspiration   ║
║    • Read the API docs                         ║
║    • Share your plugin with the community!     ║
║                                                ║
║    Happy hacking!                              ║
║                                                ║
╚════════════════════════════════════════════════╝
OUTRO
sleep 5
'

# Video 3: Advanced Workflows
echo -e "${BLUE}=== Video 3: Advanced Workflows ===${NC}"
record_video "advanced-workflows" '
# Intro
clear
cat << "INTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║         ADVANCED SCARAB WORKFLOWS              ║
║                                                ║
║    Power user tips and integrations            ║
║                                                ║
╚════════════════════════════════════════════════╝
INTRO
sleep 3
clear

# Workflow 1: Git Integration
sleep 1
echo "Workflow 1: Git Integration"
echo "==========================="
sleep 1
echo ""
echo "Scarab shows real-time git status:"
sleep 2
echo ""
echo "┌─────────────────────────────────────────┐"
echo "│ user@host:~/project     git: main*  ◀── │"
echo "└─────────────────────────────────────────┘"
sleep 2
echo ""
echo "$ git status"
echo "On branch main"
echo "Changes not staged for commit:"
echo "  modified:   src/main.rs"
sleep 2
echo ""
echo "The * indicator shows uncommitted changes"
sleep 3
clear

# Workflow 2: Docker
sleep 1
echo "Workflow 2: Docker Development"
echo "=============================="
sleep 1
echo ""
echo "Manage containers from command palette:"
sleep 1
echo ""
echo "Ctrl+Shift+P → docker ps"
echo "Ctrl+Shift+P → docker logs myapp"
echo "Ctrl+Shift+P → docker exec myapp bash"
sleep 3
echo ""
echo "Stream logs with live filtering:"
echo ""
echo "$ docker logs -f myapp | grep ERROR"
echo "[ERROR] Connection timeout"
echo "[ERROR] Database unreachable"
sleep 3
clear

# Workflow 3: SSH
sleep 1
echo "Workflow 3: SSH Sessions"
echo "========================"
sleep 1
echo ""
echo "Connect to saved hosts:"
sleep 1
echo ""
echo "Ctrl+Shift+P → ssh prod-server"
sleep 2
echo ""
echo "Connecting to production.example.com..."
sleep 1
echo "user@prod-server:~$ "
sleep 2
echo ""
echo "Sessions persist even if connection drops!"
sleep 3
clear

# Workflow 4: Multi-Language
sleep 1
echo "Workflow 4: Multi-Language Development"
echo "======================================"
sleep 1
echo ""
echo "Rust project:"
echo "$ cd ~/rust-project"
echo "$ cargo watch -x check  # Auto-rebuild"
sleep 2
echo ""
echo "Python project:"
echo "$ cd ~/python-project"
echo "$ source venv/bin/activate  # Auto-detected!"
echo "(venv) $ pytest --watch"
sleep 2
echo ""
echo "Node.js project:"
echo "$ cd ~/node-project"
echo "$ npm run dev  # Live reload"
sleep 3
clear

# Outro
sleep 1
cat << "OUTRO"
╔════════════════════════════════════════════════╗
║                                                ║
║    Master Scarab Workflows                     ║
║                                                ║
║    • Customize keybindings in config.toml      ║
║    • Create project-specific .scarab.toml      ║
║    • Write custom plugins for your workflow    ║
║    • Join our community for more tips!         ║
║                                                ║
║    github.com/raibid-labs/scarab/discussions   ║
║                                                ║
╚════════════════════════════════════════════════╝
OUTRO
sleep 5
'

echo ""
echo -e "${GREEN}All videos recorded!${NC}"
echo ""
echo "Videos saved to: $VIDEO_DIR"
echo ""
echo "Next steps:"
echo "  1. Review videos for quality"
echo "  2. Optimize file sizes if needed:"
echo "     ffmpeg -i input.mp4 -vcodec libx264 -crf 28 output.mp4"
echo "  3. Upload to YouTube or GitHub Releases"
echo "  4. Update README.md with video links"
