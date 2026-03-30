// Advanced Scarab Configuration Example
// Demonstrates custom plugins, keybindings, and dynamic configuration
//
// Usage: Copy to ~/.config/scarab/config.fsx
// Note: This is a FUTURE feature - Fusabi config DSL not yet implemented

open Scarab.Config
open Scarab.Themes
open Scarab.Plugins
open Scarab.Keybindings
open System
open System.IO

// =============================================================================
// Custom Theme Definition
// =============================================================================

let myCustomTheme =
    customTheme "tokyo-night" {
        background = rgb 0x1a 0x1b 0x26
        foreground = rgb 0xa9 0xb1 0xd6
        cursor = rgb 0xc0 0xca 0xf5

        // Normal colors
        black = rgb 0x32 0x34 0x4a
        red = rgb 0xf7 0x76 0x8e
        green = rgb 0x9e 0xce 0x6a
        yellow = rgb 0xe0 0xaf 0x68
        blue = rgb 0x7a 0xa2 0xf7
        magenta = rgb 0xbb 0x9a 0xf7
        cyan = rgb 0x7d 0xcf 0xff
        white = rgb 0xa9 0xb1 0xd6

        // Bright colors
        brightBlack = rgb 0x44 0x46 0x68
        brightRed = rgb 0xff 0x7a 0x93
        brightGreen = rgb 0xb9 0xf2 0x7c
        brightYellow = rgb 0xff 0x9e 0x64
        brightBlue = rgb 0x7d 0xa6 0xff
        brightMagenta = rgb 0xc0 0x9a 0xff
        brightCyan = rgb 0xb4 0xf9 0xf8
        brightWhite = rgb 0xc0 0xca 0xf5
    }

// =============================================================================
// Custom Plugins
// =============================================================================

// Plugin: Auto-save terminal output to daily log file
let autoSavePlugin =
    let logsDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "terminal-logs")
    let todayLog = Path.Combine(logsDir, DateTime.Now.ToString("yyyy-MM-dd") + ".log")

    // Ensure log directory exists
    if not (Directory.Exists logsDir) then
        Directory.CreateDirectory logsDir |> ignore

    {
        name = "auto-save-output"
        version = "1.0.0"

        onLoad = fun ctx ->
            ctx.log Info (sprintf "Auto-save plugin: Logging to %s" todayLog)

        onOutput = fun line ctx ->
            // Append each output line to today's log file
            File.AppendAllText(todayLog, line + "\n")
            Continue  // Don't interfere with normal output

        onInput = fun _ _ -> Continue
        onResize = fun _ _ _ -> ()
        onUnload = fun () -> ()
    }

// Plugin: Command timing - shows execution time of commands
let commandTimerPlugin =
    let mutable commandStart = DateTime.Now
    let mutable inCommand = false

    {
        name = "command-timer"
        version = "1.0.0"

        onLoad = fun ctx ->
            ctx.log Info "Command timer plugin loaded"

        onInput = fun input ctx ->
            // Check if Enter key was pressed (start timing)
            if Array.contains 13uy input then  // ASCII 13 = Enter
                commandStart <- DateTime.Now
                inCommand <- true
            Continue

        onOutput = fun line ctx ->
            // Check for prompt marker (end timing)
            if inCommand && (line.Contains("$") || line.Contains(">")) then
                let elapsed = DateTime.Now - commandStart
                if elapsed.TotalMilliseconds > 100.0 then
                    ctx.log Info (sprintf "Command took %.2fs" elapsed.TotalSeconds)
                inCommand <- false
            Continue

        onResize = fun _ _ _ -> ()
        onUnload = fun () -> ()
    }

// Plugin: Workspace-specific configuration
let workspacePlugin =
    let homeDir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)

    {
        name = "workspace-config"
        version = "1.0.0"

        onLoad = fun ctx ->
            let cwd = ctx.getWorkingDirectory()

            // Apply workspace-specific settings
            if cwd.StartsWith(Path.Combine(homeDir, "projects/rust")) then
                ctx.setEnv "RUST_BACKTRACE" "1"
                ctx.log Info "Rust workspace detected: Set RUST_BACKTRACE=1"
            elif cwd.StartsWith(Path.Combine(homeDir, "projects/python")) then
                ctx.setEnv "PYTHONUNBUFFERED" "1"
                ctx.log Info "Python workspace detected: Set PYTHONUNBUFFERED=1"

        onOutput = fun _ _ -> Continue
        onInput = fun _ _ -> Continue
        onResize = fun _ _ _ -> ()
        onUnload = fun () -> ()
    }

// =============================================================================
// Custom Keybindings
// =============================================================================

// Quick-clear: Ctrl+Shift+K - Clear screen and show last command
let quickClear ctx =
    ctx.sendInput "\x0c"  // Ctrl+L (clear screen)
    ctx.sendInput "!!\n"  // Re-run last command

// Smart paste: Ctrl+Shift+V - Remove newlines from clipboard before pasting
let smartPaste ctx =
    let clipboard = ctx.getClipboard()
    let sanitized = clipboard.Replace("\n", " ").Replace("\r", "")
    ctx.sendInput sanitized

// Project jump: Ctrl+Shift+P - Quick jump to project directory
let projectJump ctx =
    ctx.showQuickPicker [
        ("Scarab", "cd ~/projects/scarab")
        ("Fusabi", "cd ~/projects/fusabi")
        ("Dotfiles", "cd ~/dotfiles")
    ] (fun (name, cmd) ->
        ctx.sendInput (cmd + "\n")
    )

// =============================================================================
// Main Configuration
// =============================================================================

let config =
    ScarabConfig.create()
        // Terminal with larger dimensions for ultrawide monitor
        |> withTerminal {
            shell = "/usr/bin/fish"
            columns = 180u16
            rows = 50u16
            scrollback = 50000u
            altScreen = true
            scrollMultiplier = 3.0f
            autoScroll = true
        }

        // Font optimized for code
        |> withFont {
            family = "Fira Code"
            size = 13.0f
            lineHeight = 1.3f
            fallback = [
                "JetBrains Mono"
                "Hack"
                "Cascadia Code"
                "monospace"
            ]
        }

        // Custom theme
        |> withTheme myCustomTheme

        // Load built-in and custom plugins
        |> withPlugins [
            // Built-in plugins
            gitPrompt()
            urlHighlighter()

            // Custom plugins
            autoSavePlugin
            commandTimerPlugin
            workspacePlugin
        ]

        // Custom keybindings
        |> withKeybinding (bind [Ctrl; Shift] (Char 'K') (Custom quickClear))
        |> withKeybinding (bind [Ctrl; Shift] (Char 'V') (Custom smartPaste))
        |> withKeybinding (bind [Ctrl; Shift] (Char 'P') (Custom projectJump))

        // Override default keybindings
        |> withKeybinding (bind [Ctrl; Shift] (Char 'C') CopySelection)
        |> withKeybinding (bind [Ctrl; Shift] (Char 'N') NewTab)
        |> withKeybinding (bind [Ctrl; Shift] (Char 'W') CloseTab)

        // UI settings
        |> withUi {
            linkHints = true
            commandPalette = true
            animations = true
            smoothScroll = true
            showTabs = true
            tabPosition = Top
            cursorStyle = Block
            cursorBlink = true
            cursorBlinkInterval = 500u
        }

        // Session persistence
        |> withSessions {
            restoreOnStartup = true
            autoSaveInterval = 300u  // Save every 5 minutes
            saveScrollback = true
            workingDirectory = Some "/home/user/projects"
        }

// Export configuration
Scarab.export config

// Log configuration summary
printfn "Scarab Configuration Loaded:"
printfn "  Theme: Tokyo Night (custom)"
printfn "  Shell: fish"
printfn "  Dimensions: 180x50"
printfn "  Plugins: %d active" 6
printfn "  Custom Keybindings: 3"
