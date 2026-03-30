module auto_cd

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "auto-cd"
    Version = "0.1.0"
    Description = "Smart directory change suggestions based on history"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📁"
    Color = Some "#03A9F4"
    Catchphrase = Some "Navigate smarter, not harder!"
}

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Auto-CD plugin loaded!"
        return Ok ()
    }

[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Track cd commands
        if command.StartsWith("cd ") then
            let dir = command.Substring(3).Trim()
            ctx.Log Debug (sprintf "Directory change: %s" dir)

        return Continue
    }
