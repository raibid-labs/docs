// Scarab Configuration (Fusabi DSL)
// This file configures your Scarab terminal using F# syntax

module ScarabConfig

open Scarab.Config

// ============================================================================
// Terminal Settings
// ============================================================================

[<TerminalConfig>]
let terminal = {
    DefaultShell = env "SHELL" |> Option.defaultValue "/bin/zsh"
    ScrollbackLines = 10_000
    AltScreen = true
    ScrollMultiplier = 3.0
    AutoScroll = true
    Columns = 80
    Rows = 24
}

// ============================================================================
// Font Configuration
// ============================================================================

[<FontConfig>]
let font = {
    Family = "JetBrains Mono"
    Size = 14.0
    LineHeight = 1.2
    Fallback = [
        "Fira Code"
        "DejaVu Sans Mono"
        "Menlo"
        "Courier New"
    ]
    BoldIsBright = true
    UseThinStrokes = false
}

// ============================================================================
// Colors & Theme
// ============================================================================

[<ColorConfig>]
let colors = {
    // Use built-in theme
    Theme = Some "dracula"

    // Or define custom colors (overrides theme)
    Foreground = None
    Background = None
    Cursor = None
    SelectionBackground = None
    SelectionForeground = None

    // ANSI color palette (16 colors)
    Palette = {
        // Normal colors
        Black = "#21222c"
        Red = "#ff5555"
        Green = "#50fa7b"
        Yellow = "#f1fa8c"
        Blue = "#bd93f9"
        Magenta = "#ff79c6"
        Cyan = "#8be9fd"
        White = "#f8f8f2"

        // Bright colors
        BrightBlack = "#6272a4"
        BrightRed = "#ff6e6e"
        BrightGreen = "#69ff94"
        BrightYellow = "#ffffa5"
        BrightBlue = "#d6acff"
        BrightMagenta = "#ff92df"
        BrightCyan = "#a4ffff"
        BrightWhite = "#ffffff"
    }

    // Transparency
    Opacity = 1.0
    DimOpacity = 0.7
}

// ============================================================================
// Keybindings
// ============================================================================

[<KeyBindings>]
let keybindings = {
    LeaderKey = "Space"
    CopyMode = "Ctrl+Shift+C"
    Paste = "Ctrl+Shift+V"
    Search = "Ctrl+Shift+F"
    CommandPalette = "Ctrl+Shift+P"
    NewWindow = "Ctrl+Shift+N"
    CloseWindow = "Ctrl+Shift+W"
    NextTab = "Ctrl+Tab"
    PrevTab = "Ctrl+Shift+Tab"

    // Custom keybindings
    Custom = Map.ofList [
        ("OpenConfig", "Ctrl+,")
        ("ReloadConfig", "Ctrl+Shift+R")
        ("ToggleFullscreen", "F11")
        ("IncreaseFontSize", "Ctrl+=")
        ("DecreaseFontSize", "Ctrl+-")
        ("ResetFontSize", "Ctrl+0")
    ]
}

// ============================================================================
// UI Configuration
// ============================================================================

[<UiConfig>]
let ui = {
    LinkHints = true
    CommandPalette = true
    Animations = true
    SmoothScroll = true
    ShowTabs = true
    TabPosition = TabPosition.Top
    CursorStyle = CursorStyle.Block
    CursorBlink = true
    CursorBlinkInterval = 750
}

// ============================================================================
// Plugin Configuration
// ============================================================================

[<PluginConfig>]
let plugins = {
    // Enabled plugins (loaded on startup)
    Enabled = [
        "url-detector"
        "git-status"
        "command-suggestions"
        "session-manager"
    ]

    // Per-plugin configuration
    Config = Map.ofList [
        ("url-detector", {|
            IgnoreLocalhost = true
            AutoOpen = false
            NotifyOnDetect = true
        |})

        ("git-status", {|
            ShowBranch = true
            ShowDirty = true
            RefreshInterval = 1000
        |})

        ("command-suggestions", {|
            MaxSuggestions = 5
            MinScore = 0.7
            HistorySize = 1000
        |})
    ]
}

// ============================================================================
// Session Management
// ============================================================================

[<SessionConfig>]
let sessions = {
    RestoreOnStartup = false
    AutoSaveInterval = 300  // 5 minutes
    SaveScrollback = true
    WorkingDirectory = None  // Use current directory
}

// ============================================================================
// Advanced: Dynamic Configuration
// ============================================================================

// Example: Theme based on time of day
let dynamicTheme () =
    let hour = System.DateTime.Now.Hour
    if hour >= 6 && hour < 18 then
        "solarized-light"
    else
        "dracula"

// Example: Font size based on screen resolution
let dynamicFontSize () =
    let screenWidth = 1920  // TODO: Get from display info
    if screenWidth >= 3840 then
        16.0  // 4K display
    elif screenWidth >= 2560 then
        14.0  // 2K display
    else
        12.0  // 1080p or lower

// Apply dynamic config (uncomment to enable)
// colors.Theme <- Some (dynamicTheme())
// font.Size <- dynamicFontSize()

// ============================================================================
// Custom Hooks (Advanced)
// ============================================================================

// Hook: Executed on config load
[<OnConfigLoad>]
let onLoad () =
    printfn "🪲 Scarab config loaded!"
    printfn "Theme: %A" colors.Theme
    printfn "Font: %s @ %.1fpt" font.Family font.Size

// Hook: Executed on config reload (Ctrl+Shift+R)
[<OnConfigReload>]
let onReload () =
    printfn "🔄 Config reloaded!"

// Hook: Validation before applying config
[<OnConfigValidate>]
let validate () =
    if font.Size < 6.0 || font.Size > 72.0 then
        Error "Font size must be between 6.0 and 72.0"
    elif terminal.ScrollbackLines < 100 then
        Error "Scrollback must be at least 100 lines"
    else
        Ok ()

// Export configuration
[<ExportConfig>]
let config = {
    Terminal = terminal
    Font = font
    Colors = colors
    KeyBindings = keybindings
    Ui = ui
    Plugins = plugins
    Sessions = sessions
}
