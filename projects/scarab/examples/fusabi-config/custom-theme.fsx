// Custom Theme Example
// Demonstrates creating and using custom color schemes
//
// Usage: Copy to ~/.config/scarab/themes/my-theme.fsx
// Note: This is a FUTURE feature - Fusabi config DSL not yet implemented

open Scarab.Config
open Scarab.Themes

// =============================================================================
// Helper Functions for Theme Creation
// =============================================================================

// RGB color constructor
let rgb r g b = Color.fromRgb r g b

// Hex color constructor (e.g., "#1e1e2e")
let hex (s: string) =
    let r = System.Convert.ToInt32(s.Substring(1, 2), 16)
    let g = System.Convert.ToInt32(s.Substring(3, 2), 16)
    let b = System.Convert.ToInt32(s.Substring(5, 2), 16)
    rgb r g b

// =============================================================================
// Example 1: Catppuccin Mocha Theme
// =============================================================================

let catppuccinMocha =
    customTheme "catppuccin-mocha" {
        background = hex "#1e1e2e"
        foreground = hex "#cdd6f4"
        cursor = hex "#f5e0dc"

        // Base colors
        black = hex "#45475a"
        red = hex "#f38ba8"
        green = hex "#a6e3a1"
        yellow = hex "#f9e2af"
        blue = hex "#89b4fa"
        magenta = hex "#f5c2e7"
        cyan = hex "#94e2d5"
        white = hex "#bac2de"

        // Bright variants
        brightBlack = hex "#585b70"
        brightRed = hex "#f38ba8"
        brightGreen = hex "#a6e3a1"
        brightYellow = hex "#f9e2af"
        brightBlue = hex "#89b4fa"
        brightMagenta = hex "#f5c2e7"
        brightCyan = hex "#94e2d5"
        brightWhite = hex "#a6adc8"
    }

// =============================================================================
// Example 2: Solarized Dark Theme
// =============================================================================

let solarizedDark =
    customTheme "solarized-dark" {
        background = hex "#002b36"
        foreground = hex "#839496"
        cursor = hex "#93a1a1"

        black = hex "#073642"
        red = hex "#dc322f"
        green = hex "#859900"
        yellow = hex "#b58900"
        blue = hex "#268bd2"
        magenta = hex "#d33682"
        cyan = hex "#2aa198"
        white = hex "#eee8d5"

        brightBlack = hex "#002b36"
        brightRed = hex "#cb4b16"
        brightGreen = hex "#586e75"
        brightYellow = hex "#657b83"
        brightBlue = hex "#839496"
        brightMagenta = hex "#6c71c4"
        brightCyan = hex "#93a1a1"
        brightWhite = hex "#fdf6e3"
    }

// =============================================================================
// Example 3: One Dark Theme
// =============================================================================

let oneDark =
    customTheme "one-dark" {
        background = hex "#282c34"
        foreground = hex "#abb2bf"
        cursor = hex "#528bff"

        black = hex "#1e2127"
        red = hex "#e06c75"
        green = hex "#98c379"
        yellow = hex "#d19a66"
        blue = hex "#61afef"
        magenta = hex "#c678dd"
        cyan = hex "#56b6c2"
        white = hex "#abb2bf"

        brightBlack = hex "#5c6370"
        brightRed = hex "#e06c75"
        brightGreen = hex "#98c379"
        brightYellow = hex "#d19a66"
        brightBlue = hex "#61afef"
        brightMagenta = hex "#c678dd"
        brightCyan = hex "#56b6c2"
        brightWhite = hex "#ffffff"
    }

// =============================================================================
// Example 4: Dynamic Theme Based on Time of Day
// =============================================================================

let timeBasedTheme () =
    let hour = System.DateTime.Now.Hour

    // Use light theme during day (6am-6pm), dark theme at night
    if hour >= 6 && hour < 18 then
        // Light theme - Solarized Light
        customTheme "solarized-light" {
            background = hex "#fdf6e3"
            foreground = hex "#657b83"
            cursor = hex "#586e75"

            black = hex "#073642"
            red = hex "#dc322f"
            green = hex "#859900"
            yellow = hex "#b58900"
            blue = hex "#268bd2"
            magenta = hex "#d33682"
            cyan = hex "#2aa198"
            white = hex "#eee8d5"

            brightBlack = hex "#002b36"
            brightRed = hex "#cb4b16"
            brightGreen = hex "#586e75"
            brightYellow = hex "#657b83"
            brightBlue = hex "#839496"
            brightMagenta = hex "#6c71c4"
            brightCyan = hex "#93a1a1"
            brightWhite = hex "#fdf6e3"
        }
    else
        // Dark theme - One Dark
        oneDark

// =============================================================================
// Example 5: Theme with Custom ANSI Color Mappings
// =============================================================================

let customAnsiTheme =
    customTheme "custom-ansi" {
        background = hex "#0d1117"
        foreground = hex "#c9d1d9"
        cursor = hex "#58a6ff"

        // Custom ANSI color palette inspired by GitHub Dark
        black = hex "#484f58"
        red = hex "#ff7b72"
        green = hex "#3fb950"
        yellow = hex "#d29922"
        blue = hex "#58a6ff"
        magenta = hex "#bc8cff"
        cyan = hex "#39c5cf"
        white = hex "#b1bac4"

        brightBlack = hex "#6e7681"
        brightRed = hex "#ffa198"
        brightGreen = hex "#56d364"
        brightYellow = hex "#e3b341"
        brightBlue = hex "#79c0ff"
        brightMagenta = hex "#d2a8ff"
        brightCyan = hex "#56d4dd"
        brightWhite = hex "#f0f6fc"
    }

// =============================================================================
// Using Custom Themes in Config
// =============================================================================

// Example configuration using custom theme
let config =
    ScarabConfig.create()
        |> withFont {
            family = "JetBrains Mono"
            size = 14.0f
            lineHeight = 1.2f
            fallback = ["monospace"]
        }
        // Choose one of the custom themes:
        |> withTheme catppuccinMocha
        // |> withTheme solarizedDark
        // |> withTheme oneDark
        // |> withTheme (timeBasedTheme())
        // |> withTheme customAnsiTheme

Scarab.export config

// =============================================================================
// Theme Switching Function (for keybinding)
// =============================================================================

// This could be bound to a hotkey to cycle themes
let cycleTheme ctx =
    let themes = [
        ("Catppuccin Mocha", catppuccinMocha)
        ("Solarized Dark", solarizedDark)
        ("One Dark", oneDark)
        ("GitHub Dark", customAnsiTheme)
    ]

    ctx.showQuickPicker themes (fun (name, theme) ->
        ctx.setTheme theme
        ctx.log Info (sprintf "Switched to %s theme" name)
    )

// You could add this to your main config:
// |> withKeybinding (bind [Ctrl; Shift] (Char 'T') (Custom cycleTheme))
