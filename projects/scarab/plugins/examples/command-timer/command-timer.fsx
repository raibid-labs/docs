module command_timer

open Scarab.PluginApi
open System

[<Plugin>]
let metadata = {
    Name = "command-timer"
    Version = "0.1.0"
    Description = "Times command execution and alerts on slow commands"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "⏱️"
    Color = Some "#FF9800"
    Catchphrase = Some "Every millisecond counts!"
}

// Configuration thresholds (in milliseconds)
let warningThreshold = 5000    // 5 seconds
let slowThreshold = 10000      // 10 seconds

// Format duration nicely
let formatDuration (ms: int) : string =
    if ms < 1000 then
        sprintf "%dms" ms
    elif ms < 60000 then
        sprintf "%.1fs" (float ms / 1000.0)
    else
        let minutes = ms / 60000
        let seconds = (ms % 60000) / 1000
        sprintf "%dm %ds" minutes seconds

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Command Timer plugin loaded!"
        return Ok ()
    }

[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Store start time
        let startTime = DateTime.Now.ToString("o")
        ctx.SetData "start_time" startTime
        ctx.SetData "current_command" command

        ctx.Log Debug (sprintf "Started timing: %s" command)

        return Continue
    }

[<OnPostCommand>]
let onPostCommand (ctx: PluginContext) (command: string) (exitCode: int) =
    async {
        // Retrieve start time
        match ctx.GetData "start_time" with
        | Some startTimeStr ->
            let startTime = DateTime.Parse(startTimeStr)
            let endTime = DateTime.Now
            let duration = int (endTime - startTime).TotalMilliseconds

            // Log the duration
            ctx.Log Info (sprintf "Command '%s' took %s (exit: %d)" command (formatDuration duration) exitCode)

            // Notify if slow
            if duration >= slowThreshold then
                ctx.NotifyWarning
                    "Slow Command"
                    (sprintf "'%s' took %s" command (formatDuration duration))
            elif duration >= warningThreshold then
                ctx.NotifyInfo
                    "Command Duration"
                    (sprintf "'%s' took %s" command (formatDuration duration))

            // Clear stored data
            ctx.SetData "start_time" ""
            ctx.SetData "current_command" ""

        | None ->
            ctx.Log Warn "No start time found for command"

        return ()
    }
