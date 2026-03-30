// @name theme
// @version 0.1.0
// @description Theme customization plugin
// @author Scarab Team
// @api-version 0.1.0
// @min-scarab-version 0.1.0

// Theme Configuration Example
// Demonstrates how to customize colors and appearance

type Theme = {
    name: string
    background: string
    foreground: string
    cursor: string
    selection: string
    colors: string list
}

let dracula = {
    name = "Dracula"
    background = "#282a36"
    foreground = "#f8f8f2"
    cursor = "#f8f8f2"
    selection = "#44475a"
    colors = [
        "#21222c"  // black
        "#ff5555"  // red
        "#50fa7b"  // green
        "#f1fa8c"  // yellow
        "#bd93f9"  // blue
        "#ff79c6"  // magenta
        "#8be9fd"  // cyan
        "#f8f8f2"  // white
    ]
}

let nord = {
    name = "Nord"
    background = "#2e3440"
    foreground = "#d8dee9"
    cursor = "#d8dee9"
    selection = "#4c566a"
    colors = [
        "#3b4252"  // black
        "#bf616a"  // red
        "#a3be8c"  // green
        "#ebcb8b"  // yellow
        "#81a1c1"  // blue
        "#b48ead"  // magenta
        "#88c0d0"  // cyan
        "#e5e9f0"  // white
    ]
}

let current_theme = dracula

let apply_theme theme =
    printfn "Applying theme: %s" theme.name
    // Set terminal colors
    printfn "Background: %s" theme.background
    printfn "Foreground: %s" theme.foreground
