module quick_notes

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "quick-notes"
    Version = "0.1.0"
    Description = "Quick scratchpad overlay for taking notes"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📝"
    Color = Some "#FFC107"
    Catchphrase = Some "Jot it down, find it fast!"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Quick Notes plugin loaded!"

        // Initialize notes storage
        if ctx.GetData "notes" = None then
            ctx.SetData "notes" ""

        return Ok ()
    }
