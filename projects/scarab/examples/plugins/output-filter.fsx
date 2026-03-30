(*
 * output-filter.fsx
 *
 * Name: Output Filter Plugin
 * Version: 1.0.0
 * Description: Detects patterns in terminal output and highlights them
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates:
 * - The on_output hook for intercepting terminal output
 * - Pattern matching with regex
 * - Drawing overlays on the terminal
 * - Stateful plugin design
 *)

open Scarab.PluginApi
open System.Text.RegularExpressions

// Plugin metadata
let metadata = {
    Name = "output-filter"
    Version = "1.0.0"
    Description = "Highlights errors and warnings in terminal output"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Regex patterns to detect
let errorPattern = Regex(@"\b(error|ERROR|Error)\b", RegexOptions.Compiled)
let warningPattern = Regex(@"\b(warning|WARNING|Warning)\b", RegexOptions.Compiled)
let successPattern = Regex(@"\b(success|SUCCESS|Success|passed|PASSED)\b", RegexOptions.Compiled)

// Mutable state to track overlay IDs
let mutable nextOverlayId = 0UL

// Generate unique overlay ID
let getNextOverlayId () =
    let id = nextOverlayId
    nextOverlayId <- nextOverlayId + 1UL
    id

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Output filter plugin loaded")

        // Check configuration for custom patterns
        match ctx.Config.GetOpt<bool>("enabled") with
        | Some false ->
            ctx.Log(LogLevel.Info, "Output filtering is disabled by config")
        | _ ->
            ctx.Log(LogLevel.Info, "Output filtering is enabled")

        return Ok ()
    }

// on_output hook - called for each line of output before it's displayed
let on_output (line: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Check if filtering is enabled
        match ctx.Config.GetOpt<bool>("enabled") with
        | Some false -> return Ok Action.Continue
        | _ -> ()

        // Get current cursor position to know where to draw overlays
        let (cursorX, cursorY) = ctx.GetCursor()

        // Check for errors
        if errorPattern.IsMatch(line) then
            ctx.Log(LogLevel.Debug, sprintf "Found error pattern in: %s" line)

            // Draw a red indicator overlay
            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = getNextOverlayId()
                X = 0us
                Y = cursorY
                Text = "[ERROR]"
                Style = {
                    Fg = 0xFFFFFFFFu  // White text
                    Bg = 0xFF0000FFu  // Red background
                    ZIndex = 50.0f
                }
            })

        // Check for warnings
        if warningPattern.IsMatch(line) then
            ctx.Log(LogLevel.Debug, sprintf "Found warning pattern in: %s" line)

            // Draw a yellow indicator overlay
            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = getNextOverlayId()
                X = 0us
                Y = cursorY
                Text = "[WARN]"
                Style = {
                    Fg = 0x000000FFu  // Black text
                    Bg = 0xFFFF00FFu  // Yellow background
                    ZIndex = 50.0f
                }
            })

        // Check for success
        if successPattern.IsMatch(line) then
            ctx.Log(LogLevel.Debug, sprintf "Found success pattern in: %s" line)

            // Draw a green indicator overlay
            ctx.QueueCommand(RemoteCommand.DrawOverlay {
                Id = getNextOverlayId()
                X = 0us
                Y = cursorY
                Text = "[OK]"
                Style = {
                    Fg = 0xFFFFFFFFu  // White text
                    Bg = 0x00FF00FFu  // Green background
                    ZIndex = 50.0f
                }
            })

        // Always continue processing (don't block output)
        return Ok Action.Continue
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Clear all overlays when unloading
        ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = None })
        ctx.Log(LogLevel.Info, "Output filter plugin unloaded")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = Some on_output
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = None
    GetCommands = fun () -> []
}
