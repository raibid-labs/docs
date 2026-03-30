(*
 * logging-demo.fsx
 *
 * Name: Logging and Notification Demo Plugin
 * Version: 1.0.0
 * Description: Demonstrates the plugin logging and notification system
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates the complete logging and notification API:
 * - Using ctx.Log() for different log levels
 * - Using ctx.Notify() for user-facing notifications
 * - Convenience notification methods (NotifyInfo, NotifySuccess, etc.)
 * - Integration with output and remote command hooks
 *)

// Import the Scarab Plugin API module
open Scarab.PluginApi

// Plugin metadata
let metadata = {
    Name = "logging-demo"
    Version = "1.0.0"
    Description = "Demo plugin showing logging and notifications"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📊"
    Color = Some "#4A90E2"
    Catchphrase = Some "Watch me log everything!"
}

// on_load hook - demonstrate different logging levels
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Log at different levels to demonstrate routing
        ctx.Log(LogLevel.Info, "Logging Demo Plugin initialized")
        ctx.Log(LogLevel.Debug, "This is a debug message - only visible with debug logging")

        // Send a success notification to the user
        ctx.NotifySuccess("Plugin Loaded", "Logging Demo Plugin is ready to demonstrate features")

        // Store some plugin-specific data
        ctx.SetData("load_time", System.DateTime.Now.ToString())
        ctx.SetData("notification_count", "0")

        return Ok ()
    }

// on_output hook - detect patterns in terminal output and notify user
let on_output (data: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Log every 10th output line (to avoid spam)
        let count =
            match ctx.GetData("output_count") with
            | Some c -> int c
            | None -> 0

        ctx.SetData("output_count", string (count + 1))

        if count % 10 = 0 then
            ctx.Log(LogLevel.Debug, sprintf "Processed %d lines of output" count)

        // Detect error patterns and notify
        if data.Contains("error") || data.Contains("ERROR") then
            ctx.Log(LogLevel.Warn, "Detected error in terminal output")
            ctx.NotifyWarning("Terminal Error Detected", sprintf "Line: %s" (data.Substring(0, min 50 data.Length)))

        // Detect success patterns and celebrate
        if data.Contains("success") || data.Contains("SUCCESS") || data.Contains("✓") then
            ctx.Log(LogLevel.Info, "Detected success message")
            ctx.NotifySuccess("Operation Succeeded", "Detected successful operation in terminal")

        // Detect build completion
        if data.Contains("Finished") || data.Contains("Build succeeded") then
            ctx.NotifyInfo("Build Complete", "Your build has finished running")

        // Continue processing (don't modify output)
        return Ok Action.Continue
    }

// on_resize hook - notify user about terminal resize
let on_resize (cols: int) (rows: int) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, sprintf "Terminal resized to %dx%d" cols rows)

        // Only show notification for significant size changes (to avoid spam)
        if cols < 80 || rows < 24 then
            ctx.NotifyWarning("Small Terminal", sprintf "Terminal is %dx%d - some features may not display correctly" cols rows)

        return Ok ()
    }

// on_attach hook - welcome message when client connects
let on_attach (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Client connected to session")

        // Show welcome notification
        let loadTime = ctx.GetData("load_time") |> Option.defaultValue "unknown"
        ctx.NotifyInfo("Welcome Back", sprintf "Logging Demo Plugin loaded at %s" loadTime)

        return Ok ()
    }

// on_remote_command hook - handle user-triggered commands
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "logging-demo.test-notifications" ->
            ctx.Log(LogLevel.Info, "Testing all notification levels")

            // Demonstrate all notification levels
            ctx.NotifyInfo("Info Notification", "This is an informational message")
            ctx.NotifySuccess("Success Notification", "This indicates a successful operation")
            ctx.NotifyWarning("Warning Notification", "This is a warning message")
            ctx.NotifyError("Error Notification", "This indicates an error occurred")

            // Increment notification counter
            let count =
                match ctx.GetData("notification_count") with
                | Some c -> int c + 4
                | None -> 4
            ctx.SetData("notification_count", string count)

            return Ok ()

        | "logging-demo.test-logging" ->
            ctx.Log(LogLevel.Info, "Testing all log levels")

            // Demonstrate all log levels
            ctx.Log(LogLevel.Error, "This is an ERROR level log message")
            ctx.Log(LogLevel.Warn, "This is a WARN level log message")
            ctx.Log(LogLevel.Info, "This is an INFO level log message")
            ctx.Log(LogLevel.Debug, "This is a DEBUG level log message")

            ctx.NotifySuccess("Logging Test", "Check the daemon logs to see all levels")
            return Ok ()

        | "logging-demo.stats" ->
            let outputCount = ctx.GetData("output_count") |> Option.defaultValue "0"
            let notifyCount = ctx.GetData("notification_count") |> Option.defaultValue "0"
            let loadTime = ctx.GetData("load_time") |> Option.defaultValue "unknown"

            ctx.Log(LogLevel.Info, sprintf "Plugin stats: %s lines processed, %s notifications sent" outputCount notifyCount)
            ctx.NotifyInfo("Plugin Statistics", sprintf "Loaded: %s\nLines: %s\nNotifications: %s" loadTime outputCount notifyCount)
            return Ok ()

        | _ ->
            return Ok ()
    }

// Define remote commands that appear in the command palette
let get_commands () : ModalItem list =
    [
        {
            Id = "logging-demo.test-notifications"
            Label = "📊 Test All Notifications"
            Description = Some "Trigger all notification levels (Info, Success, Warning, Error)"
        }
        {
            Id = "logging-demo.test-logging"
            Label = "📋 Test All Log Levels"
            Description = Some "Send messages at all log levels to the daemon"
        }
        {
            Id = "logging-demo.stats"
            Label = "📈 Show Plugin Stats"
            Description = Some "Display statistics about this plugin's activity"
        }
    ]

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        let outputCount = ctx.GetData("output_count") |> Option.defaultValue "0"
        let notifyCount = ctx.GetData("notification_count") |> Option.defaultValue "0"

        ctx.Log(LogLevel.Info, sprintf "Logging Demo Plugin unloading - processed %s lines, sent %s notifications" outputCount notifyCount)
        ctx.NotifyInfo("Plugin Unloaded", "Logging Demo Plugin is shutting down")

        return Ok ()
    }

// Export the plugin definition
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = Some on_output
    OnInput = None
    OnResize = Some on_resize
    OnAttach = Some on_attach
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = Some on_remote_command
    GetCommands = get_commands
}
