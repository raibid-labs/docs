(*
 * custom-keybind.fsx
 *
 * Name: Custom Keybind Plugin
 * Version: 1.0.0
 * Description: Intercepts key combinations and triggers custom actions
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates:
 * - The on_input hook for intercepting keyboard input
 * - Detecting key combinations
 * - Modifying or consuming input
 * - Queuing remote UI commands (modals, overlays)
 * - Registering custom commands in the command palette
 *)

open Scarab.PluginApi

// Plugin metadata
let metadata = {
    Name = "custom-keybind"
    Version = "1.0.0"
    Description = "Custom keybindings for common actions"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Mutable state to track active mode
let mutable commandMode = false

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Custom keybind plugin loaded")
        ctx.Log(LogLevel.Info, "Press Ctrl+K to enter command mode")
        ctx.Log(LogLevel.Info, "Press Ctrl+H for help")
        return Ok ()
    }

// Helper function to check if input matches a key combo
let matchesKey (input: byte[]) (expected: byte[]) : bool =
    if input.Length <> expected.Length then false
    else
        Array.forall2 (fun a b -> a = b) input expected

// Show help modal
let showHelp (ctx: PluginContext) =
    let items = [
        { Id = "help-1"; Label = "Ctrl+K"; Description = Some "Enter command mode" }
        { Id = "help-2"; Label = "Ctrl+H"; Description = Some "Show this help" }
        { Id = "help-3"; Label = "Ctrl+G"; Description = Some "Show terminal stats" }
        { Id = "help-4"; Label = "Ctrl+T"; Description = Some "Timestamp current line" }
    ]
    ctx.QueueCommand(RemoteCommand.ShowModal {
        Title = "Custom Keybindings Help"
        Items = items
    })

// Show terminal statistics
let showStats (ctx: PluginContext) =
    let (cols, rows) = ctx.GetSize()
    let (cursorX, cursorY) = ctx.GetCursor()

    let statsText = sprintf "Size: %dx%d | Cursor: (%d,%d)" cols rows cursorX cursorY

    // Draw overlay with stats
    ctx.QueueCommand(RemoteCommand.DrawOverlay {
        Id = 9999UL
        X = 0us
        Y = 0us
        Text = statsText
        Style = {
            Fg = 0xFFFFFFFFu  // White
            Bg = 0x0000FFFFu  // Blue
            ZIndex = 200.0f
        }
    })

    ctx.Log(LogLevel.Info, statsText)

// Insert timestamp at current position
let insertTimestamp (ctx: PluginContext) : byte[] =
    let timestamp = System.DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss")
    ctx.Log(LogLevel.Info, sprintf "Inserting timestamp: %s" timestamp)
    System.Text.Encoding.UTF8.GetBytes(sprintf "# %s " timestamp)

// on_input hook - intercepts keyboard input
let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Ctrl+K (0x0B) - Enter command mode
        if matchesKey input [|0x0Buy|] then
            commandMode <- true
            ctx.Log(LogLevel.Info, "Command mode activated")
            ctx.Notify("Command Mode: Press a key for action (Esc to cancel)")

            // Show available commands
            let items = [
                { Id = "cmd-help"; Label = "h - Help"; Description = Some "Show keybinding help" }
                { Id = "cmd-stats"; Label = "s - Stats"; Description = Some "Show terminal stats" }
                { Id = "cmd-time"; Label = "t - Time"; Description = Some "Insert timestamp" }
                { Id = "cmd-clear"; Label = "c - Clear"; Description = Some "Clear overlays" }
            ]
            ctx.QueueCommand(RemoteCommand.ShowModal {
                Title = "Command Mode"
                Items = items
            })

            // Consume the key
            return Ok (Action.Modify [||])

        // Ctrl+H (0x08) - Show help
        if matchesKey input [|0x08uy|] then
            ctx.Log(LogLevel.Info, "Showing help")
            showHelp ctx
            return Ok (Action.Modify [||])  // Consume the key

        // Ctrl+G (0x07) - Show stats
        if matchesKey input [|0x07uy|] then
            ctx.Log(LogLevel.Info, "Showing stats")
            showStats ctx
            return Ok (Action.Modify [||])  // Consume the key

        // Ctrl+T (0x14) - Insert timestamp
        if matchesKey input [|0x14uy|] then
            let timestamp = insertTimestamp ctx
            return Ok (Action.Modify timestamp)  // Replace input with timestamp

        // Handle command mode input
        if commandMode then
            // Escape (0x1B) - Cancel command mode
            if matchesKey input [|0x1Buy|] then
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                ctx.Log(LogLevel.Info, "Command mode cancelled")
                return Ok (Action.Modify [||])

            // Match command mode keys
            match input with
            | [|0x68uy|] ->  // 'h' - Help
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                showHelp ctx
                return Ok (Action.Modify [||])

            | [|0x73uy|] ->  // 's' - Stats
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                showStats ctx
                return Ok (Action.Modify [||])

            | [|0x74uy|] ->  // 't' - Timestamp
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                let timestamp = insertTimestamp ctx
                return Ok (Action.Modify timestamp)

            | [|0x63uy|] ->  // 'c' - Clear overlays
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = None })
                ctx.Log(LogLevel.Info, "Cleared all overlays")
                return Ok (Action.Modify [||])

            | _ ->
                // Unknown key in command mode, cancel
                commandMode <- false
                ctx.QueueCommand(RemoteCommand.HideModal)
                ctx.Log(LogLevel.Warn, "Unknown command key")
                return Ok (Action.Modify [||])
        else
            // Not in command mode, pass through
            return Ok Action.Continue
    }

// Provide commands for the command palette
let getCommands () : ModalItem list =
    [
        { Id = "keybind.help"; Label = "Show Keybindings Help"; Description = Some "Display all custom keybindings" }
        { Id = "keybind.stats"; Label = "Show Terminal Stats"; Description = Some "Display terminal size and cursor info" }
        { Id = "keybind.timestamp"; Label = "Insert Timestamp"; Description = Some "Insert current timestamp at cursor" }
        { Id = "keybind.clear"; Label = "Clear Overlays"; Description = Some "Clear all overlay indicators" }
    ]

// Handle remote command from command palette
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "keybind.help" ->
            showHelp ctx
        | "keybind.stats" ->
            showStats ctx
        | "keybind.timestamp" ->
            let timestamp = insertTimestamp ctx
            // Note: We can't directly inject input here, but we log it
            ctx.Log(LogLevel.Info, "Timestamp command triggered from palette")
        | "keybind.clear" ->
            ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = None })
            ctx.Log(LogLevel.Info, "Cleared all overlays")
        | _ ->
            ctx.Log(LogLevel.Warn, sprintf "Unknown command: %s" id)

        return Ok ()
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = None })
        ctx.QueueCommand(RemoteCommand.HideModal)
        ctx.Log(LogLevel.Info, "Custom keybind plugin unloaded")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None
    OnInput = Some on_input
    OnResize = None
    OnAttach = None
    OnDetach = None
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = Some on_remote_command
    GetCommands = getCommands
}
