#!/bin/bash
# record-demos.sh - Record all demo GIFs for Scarab Terminal
#
# Usage: ./scripts/record-demos.sh
#
# Prerequisites:
#   - asciinema (pip install asciinema)
#   - agg (cargo install agg)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEMO_DIR="$PROJECT_ROOT/docs/assets/demos"
CAST_DIR="$DEMO_DIR/casts"

# Create directories
mkdir -p "$DEMO_DIR"
mkdir -p "$CAST_DIR"

echo "Recording Scarab Terminal Demos"
echo "================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to record and convert a demo
record_demo() {
    local name="$1"
    local script="$2"

    echo -e "${BLUE}Recording: $name${NC}"

    # Create cast file
    cat > "$CAST_DIR/$name.sh" << EOF
$script
EOF

    chmod +x "$CAST_DIR/$name.sh"

    # Record
    asciinema rec "$CAST_DIR/$name.cast" -c "$CAST_DIR/$name.sh" --overwrite

    # Convert to GIF
    agg "$CAST_DIR/$name.cast" "$DEMO_DIR/$name.gif" \
        --theme dracula \
        --font-family "JetBrains Mono" \
        --font-size 14 \
        --cols 100 \
        --rows 30

    echo -e "${GREEN}✓ Created: $DEMO_DIR/$name.gif${NC}"
    echo ""
}

# Demo 1: Link Hints
record_demo "link-hints-demo" '
clear
sleep 1
echo "Scarab Terminal - Link Hints Demo"
echo "=================================="
sleep 1
echo ""
echo "URLs are automatically detected:"
sleep 1
echo "  • GitHub: https://github.com/raibid-labs/scarab"
sleep 1
echo "  • Docs: https://scarab.dev/docs"
sleep 1
echo "  • Issues: https://github.com/raibid-labs/scarab/issues"
sleep 1
echo ""
echo "Press Ctrl+Shift+O to activate link hints..."
sleep 2
echo ""
echo "[Link hints would appear here: a, b, c]"
sleep 2
echo ""
echo "Press a letter to open the corresponding link"
sleep 2
clear
echo "✓ Link opened in browser!"
sleep 2
'

# Demo 2: Command Palette
record_demo "command-palette" '
clear
sleep 1
echo "Scarab Terminal - Command Palette"
echo "=================================="
sleep 1
echo ""
echo "Press Ctrl+Shift+P to open command palette..."
sleep 2
clear
cat << "PALETTE"
┌─────────────────────────────────────────┐
│ Command Palette                Ctrl+P   │
├─────────────────────────────────────────┤
│ > sess_                                 │
│                                         │
│   session new     Create new session    │
│   session list    List all sessions     │
│   session switch  Switch to session     │
│   session rename  Rename session        │
│   session delete  Delete session        │
│                                         │
└─────────────────────────────────────────┘
PALETTE
sleep 3
echo ""
echo "Use fuzzy search to find commands quickly!"
sleep 2
'

# Demo 3: Plugin Install
record_demo "plugin-install" '
clear
sleep 1
echo "Scarab Terminal - Plugin Installation"
echo "====================================="
sleep 1
echo ""
echo "Step 1: Create plugin directory"
echo "$ mkdir -p ~/.config/scarab/plugins"
sleep 2
echo ""
echo "Step 2: Create hello-world.fsx"
sleep 1
cat << "PLUGIN"

// hello-world.fsx
open Scarab.PluginApi

let metadata = {
    Name = "hello-world"
    Version = "1.0.0"
    Description = "My first plugin"
    Author = "Your Name"
}

let on_load ctx = async {
    ctx.Log(Info, "Hello from plugin!")
    return Ok ()
}

Plugin.Register {
    Metadata = metadata
    OnLoad = Some on_load
}
PLUGIN
sleep 4
echo ""
echo "Step 3: Restart daemon"
echo "$ scarab-daemon &"
sleep 2
echo ""
echo "[INFO] Loading plugins..."
sleep 1
echo "[INFO] Plugin hello-world loaded"
sleep 1
echo "[INFO] Hello from plugin!"
sleep 2
clear
echo "✓ Plugin installed successfully!"
sleep 2
'

# Demo 4: Theme Switch
record_demo "theme-switch" '
clear
sleep 1
echo "Scarab Terminal - Theme Switching"
echo "================================="
sleep 1
echo ""
echo "Current theme: Dracula (dark)"
sleep 2
echo ""
echo "Press Ctrl+Shift+P > theme monokai"
sleep 2
clear
echo "Theme changed to: Monokai"
sleep 1
echo ""
echo "Available themes:"
echo "  • dracula"
echo "  • monokai"
echo "  • solarized-dark"
echo "  • nord"
echo "  • gruvbox-dark"
echo "  • one-dark"
sleep 3
echo ""
echo "Themes switch instantly - no restart!"
sleep 2
'

# Demo 5: Split Panes (placeholder - feature upcoming)
record_demo "split-panes" '
clear
sleep 1
echo "Scarab Terminal - Split Panes"
echo "============================="
sleep 1
echo ""
echo "This feature is coming soon!"
sleep 1
echo ""
echo "Planned features:"
echo "  • Horizontal/vertical splits"
echo "  • Vim-style navigation"
echo "  • Resize with keyboard"
echo "  • Multiple terminals in one window"
sleep 3
echo ""
echo "Follow development on GitHub:"
echo "github.com/raibid-labs/scarab"
sleep 2
'

echo ""
echo -e "${GREEN}All demos recorded!${NC}"
echo ""
echo "Demos saved to: $DEMO_DIR"
echo ""
echo "Generated files:"
ls -lh "$DEMO_DIR"/*.gif
echo ""
echo "To optimize GIFs (reduce file size):"
echo "  gifsicle -O3 --colors 256 input.gif -o output.gif"
