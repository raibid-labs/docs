(*
 * notification-monitor.fsx
 *
 * Name: Notification Monitor Plugin
 * Version: 1.0.0
 * Description: Monitors long-running commands and sends notifications
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates:
 * - Tracking command execution time
 * - Using mutable state to track ongoing operations
 * - Sending notifications to users
 * - Configuration-driven behavior
 *)

open Scarab.PluginApi
open System
open System.Collections.Generic

// Plugin metadata
let metadata = {
    Name = "notification-monitor"
    Version = "1.0.0"
    Description = "Notify when long-running commands complete"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Track running commands
type CommandTracker = {
    Command: string
    StartTime: DateTime
}

let runningCommands = Dictionary<int, CommandTracker>()
let mutable nextCommandId = 0
let mutable thresholdSeconds = 5.0  // Default: notify for commands > 5 seconds

// Format duration as human-readable string
let formatDuration (duration: TimeSpan) : string =
    if duration.TotalHours >= 1.0 then
        sprintf "%.1fh" duration.TotalHours
    elif duration.TotalMinutes >= 1.0 then
        sprintf "%.1fm" duration.TotalMinutes
    else
        sprintf "%.1fs" duration.TotalSeconds

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Notification monitor plugin loaded")

        // Read threshold from config
        match ctx.Config.GetOpt<float>("threshold_seconds") with
        | Some threshold ->
            thresholdSeconds <- threshold
            ctx.Log(LogLevel.Info, sprintf "Notification threshold set to %.1f seconds" threshold)
        | None ->
            ctx.Log(LogLevel.Info, sprintf "Using default threshold of %.1f seconds" thresholdSeconds)

        return Ok ()
    }

// on_pre_command hook - start tracking
let on_pre_command (command: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        let id = nextCommandId
        nextCommandId <- nextCommandId + 1

        let tracker = {
            Command = command
            StartTime = DateTime.Now
        }

        runningCommands.[id] <- tracker
        ctx.Log(LogLevel.Debug, sprintf "Started tracking command [%d]: %s" id command)

        // Store command ID in context data for correlation
        ctx.SetData(sprintf "cmd_id_%d" id, id.ToString())

        return Ok Action.Continue
    }

// on_post_command hook - check duration and notify
let on_post_command (command: string) (exitCode: int) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Find the matching command
        let mutable foundId = -1
        for kvp in runningCommands do
            if kvp.Value.Command = command then
                foundId <- kvp.Key

        if foundId >= 0 then
            let tracker = runningCommands.[foundId]
            let duration = DateTime.Now - tracker.StartTime

            ctx.Log(LogLevel.Debug,
                sprintf "Command [%d] completed in %s: %s" foundId (formatDuration duration) command)

            // Check if we should notify
            if duration.TotalSeconds >= thresholdSeconds then
                let statusText = if exitCode = 0 then "succeeded" else sprintf "failed (exit %d)" exitCode
                let notificationText = sprintf "Command %s after %s:\n%s" statusText (formatDuration duration) command

                ctx.Notify(notificationText)
                ctx.Log(LogLevel.Info, sprintf "Sent notification for long-running command: %s" command)

                // Draw a temporary overlay
                let (fg, bg) =
                    if exitCode = 0 then
                        (0xFFFFFFFFu, 0x00AA00FFu)  // Green for success
                    else
                        (0xFFFFFFFFu, 0xFF0000FFu)  // Red for failure

                ctx.QueueCommand(RemoteCommand.DrawOverlay {
                    Id = 20000UL + uint64 foundId
                    X = 0us
                    Y = 1us
                    Text = sprintf " %s after %s " statusText (formatDuration duration)
                    Style = {
                        Fg = fg
                        Bg = bg
                        ZIndex = 150.0f
                    }
                })

            // Remove from tracking
            runningCommands.Remove(foundId) |> ignore

        return Ok ()
    }

// Provide commands for the command palette
let getCommands () : ModalItem list =
    [
        { Id = "notify.status"; Label = "Show Notification Status"; Description = Some "Show current notification settings" }
        { Id = "notify.clear"; Label = "Clear Notification Overlays"; Description = Some "Clear all notification overlays" }
    ]

// Handle remote command from command palette
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "notify.status" ->
            let runningCount = runningCommands.Count
            let statusText =
                sprintf "Notification Monitor:\n- Threshold: %.1fs\n- Running commands: %d"
                    thresholdSeconds runningCount

            ctx.Notify(statusText)
            ctx.Log(LogLevel.Info, statusText)

        | "notify.clear" ->
            // Clear notification overlays (IDs 20000+)
            for i in 0 .. 999 do
                ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some (20000UL + uint64 i) })
            ctx.Log(LogLevel.Info, "Cleared notification overlays")

        | _ ->
            ctx.Log(LogLevel.Warn, sprintf "Unknown command: %s" id)

        return Ok ()
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Clear all notification overlays
        for i in 0 .. 999 do
            ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some (20000UL + uint64 i) })

        runningCommands.Clear()
        ctx.Log(LogLevel.Info, "Notification monitor plugin unloaded")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None
    OnInput = None
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = Some on_pre_command
    OnPostCommand = Some on_post_command
    OnRemoteCommand = Some on_remote_command
    GetCommands = getCommands
}
