module theme_switcher

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "theme-switcher"
    Version = "0.1.0"
    Description = "Quick theme switching with live preview"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🎨"
    Color = Some "#E91E63"
    Catchphrase = Some "Change your vibe instantly!"
}

// Available themes
let themes = [
    "dracula"
    "monokai"
    "solarized-dark"
    "solarized-light"
    "gruvbox"
    "nord"
    "tokyo-night"
    "catppuccin"
]

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Theme Switcher plugin loaded!"

        // Get current theme
        match ctx.GetData "current_theme" with
        | Some theme ->
            ctx.Log Info (sprintf "Current theme: %s" theme)
        | None ->
            ctx.SetData "current_theme" "dracula"

        return Ok ()
    }
