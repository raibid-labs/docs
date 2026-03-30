(*
 * session-manager.fsx
 *
 * Name: Session Manager Plugin
 * Version: 1.0.0
 * Description: Advanced session management with quick switching and layouts
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates:
 * - Complex state management
 * - Integration with daemon session system
 * - Advanced command palette usage
 * - Persistent state across sessions
 * - Client attach/detach hooks
 *)

open Scarab.PluginApi
open System
open System.Collections.Generic
open System.IO

// Plugin metadata
let metadata = {
    Name = "session-manager"
    Version = "1.0.0"
    Description = "Advanced session management and quick switching"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Session metadata we track
type SessionMetadata = {
    Id: string
    Name: string
    CreatedAt: DateTime
    LastAccessed: DateTime
    Tags: string list
    Color: uint32
}

// Session state
let sessions = Dictionary<string, SessionMetadata>()
let mutable currentSessionId = ""
let mutable sessionStateFile = ""

// Session colors for visual differentiation
let sessionColors = [|
    0x5E81ACFFu  // Blue
    0xBF616AFFu  // Red
    0xA3BE8CFFu  // Green
    0xEBCB8BFFu  // Yellow
    0xB48EADFFu  // Purple
    0x88C0D0FFu  // Cyan
    0xD08770FFu  // Orange
|]

let mutable nextColorIndex = 0

// Get next color for session
let getNextColor () =
    let color = sessionColors.[nextColorIndex]
    nextColorIndex <- (nextColorIndex + 1) % sessionColors.Length
    color

// Load session state from disk
let loadSessionState (ctx: PluginContext) =
    try
        if File.Exists(sessionStateFile) then
            let json = File.ReadAllText(sessionStateFile)
            // In real implementation, deserialize JSON
            ctx.Log(LogLevel.Debug, "Loaded session state from disk")
        else
            ctx.Log(LogLevel.Debug, "No saved session state found")
    with ex ->
        ctx.Log(LogLevel.Warn, sprintf "Failed to load session state: %s" ex.Message)

// Save session state to disk
let saveSessionState (ctx: PluginContext) =
    try
        // In real implementation, serialize sessions dictionary to JSON
        // File.WriteAllText(sessionStateFile, json)
        ctx.Log(LogLevel.Debug, "Saved session state to disk")
    with ex ->
        ctx.Log(LogLevel.Warn, sprintf "Failed to save session state: %s" ex.Message)

// Add or update session
let updateSession (id: string) (name: string) (ctx: PluginContext) =
    let now = DateTime.Now

    if sessions.ContainsKey(id) then
        let existing = sessions.[id]
        sessions.[id] <- { existing with LastAccessed = now }
    else
        sessions.[id] <- {
            Id = id
            Name = name
            CreatedAt = now
            LastAccessed = now
            Tags = []
            Color = getNextColor()
        }

    saveSessionState ctx

// Draw session indicator
let drawSessionIndicator (ctx: PluginContext) =
    if currentSessionId <> "" && sessions.ContainsKey(currentSessionId) then
        let session = sessions.[currentSessionId]
        let sessionText = sprintf " [%s] " session.Name

        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 30000UL  // Fixed ID for session indicator
            X = 0us
            Y = 0us
            Text = sessionText
            Style = {
                Fg = 0xFFFFFFFFu
                Bg = session.Color
                ZIndex = 100.0f
            }
        })

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Session manager plugin loaded")

        // Get state file path from config or use default
        let homeDir = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
        sessionStateFile <- Path.Combine(homeDir, ".config", "scarab", "sessions.json")

        // Load saved state
        loadSessionState ctx

        // Draw initial indicator
        drawSessionIndicator ctx

        return Ok ()
    }

// on_attach hook - track when client connects
let on_attach (clientId: uint64) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, sprintf "Client %d attached" clientId)

        // In a real implementation, we'd query daemon for current session
        // For now, just update the indicator
        drawSessionIndicator ctx

        return Ok ()
    }

// on_detach hook - track when client disconnects
let on_detach (clientId: uint64) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, sprintf "Client %d detached" clientId)
        return Ok ()
    }

// Get session list formatted for command palette
let getSessionCommands () : ModalItem list =
    let sortedSessions =
        sessions.Values
        |> Seq.sortByDescending (fun s -> s.LastAccessed)
        |> Seq.toList

    let items =
        sortedSessions
        |> List.map (fun s ->
            let timeAgo =
                let diff = DateTime.Now - s.LastAccessed
                if diff.TotalMinutes < 1.0 then "just now"
                elif diff.TotalHours < 1.0 then sprintf "%.0f min ago" diff.TotalMinutes
                elif diff.TotalDays < 1.0 then sprintf "%.0f hr ago" diff.TotalHours
                else sprintf "%.0f days ago" diff.TotalDays

            {
                Id = sprintf "session.switch.%s" s.Id
                Label = s.Name
                Description = Some (sprintf "Last accessed: %s" timeAgo)
            }
        )

    // Add "new session" option at the top
    {
        Id = "session.new"
        Label = "+ New Session"
        Description = Some "Create a new terminal session"
    } :: items

// Show session list
let showSessionList (ctx: PluginContext) =
    let items = getSessionCommands()
    ctx.QueueCommand(RemoteCommand.ShowModal {
        Title = "Sessions"
        Items = items
    })

// Provide commands for the command palette
let getCommands () : ModalItem list =
    [
        { Id = "session.list"; Label = "Show Sessions"; Description = Some "List all terminal sessions" }
        { Id = "session.new"; Label = "New Session"; Description = Some "Create a new session" }
        { Id = "session.rename"; Label = "Rename Session"; Description = Some "Rename current session" }
        { Id = "session.delete"; Label = "Delete Session"; Description = Some "Delete a session" }
        { Id = "session.tag"; Label = "Tag Session"; Description = Some "Add tags to session" }
    ]

// Handle remote command from command palette
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "session.list" ->
            ctx.Log(LogLevel.Info, "Showing session list")
            showSessionList ctx

        | "session.new" ->
            ctx.Log(LogLevel.Info, "Creating new session")
            // In real implementation, send SessionCreate message to daemon
            ctx.Notify("New session created (demo)")
            let newId = Guid.NewGuid().ToString()
            let newName = sprintf "Session %d" (sessions.Count + 1)
            updateSession newId newName ctx
            currentSessionId <- newId
            drawSessionIndicator ctx

        | "session.rename" ->
            ctx.Log(LogLevel.Info, "Renaming session")
            // In real implementation, show input modal
            ctx.Notify("Session rename (demo)")

        | "session.delete" ->
            ctx.Log(LogLevel.Info, "Deleting session")
            showSessionList ctx
            // Modal items with ID prefix "session.delete." would trigger deletion

        | "session.tag" ->
            ctx.Log(LogLevel.Info, "Tagging session")
            ctx.Notify("Session tagging (demo)")

        // Handle session switching
        | id when id.StartsWith("session.switch.") ->
            let sessionId = id.Substring("session.switch.".Length)
            if sessions.ContainsKey(sessionId) then
                ctx.Log(LogLevel.Info, sprintf "Switching to session: %s" sessionId)
                currentSessionId <- sessionId
                updateSession sessionId sessions.[sessionId].Name ctx
                drawSessionIndicator ctx
                ctx.Notify(sprintf "Switched to: %s" sessions.[sessionId].Name)
            else
                ctx.Log(LogLevel.Warn, sprintf "Unknown session: %s" sessionId)

        | _ ->
            ctx.Log(LogLevel.Warn, sprintf "Unknown command: %s" id)

        return Ok ()
    }

// on_resize hook - reposition session indicator
let on_resize (cols: uint16) (rows: uint16) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Debug, sprintf "Terminal resized to %dx%d" cols rows)
        drawSessionIndicator ctx
        return Ok ()
    }

// on_input hook - keyboard shortcuts for session switching
let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Alt+S (Esc + 's') - Show session list
        if input.Length = 2 && input.[0] = 0x1Buy && input.[1] = byte 's' then
            ctx.Log(LogLevel.Debug, "Session list shortcut triggered")
            showSessionList ctx
            return Ok (Action.Modify [||])  // Consume key

        // Alt+N (Esc + 'n') - New session
        elif input.Length = 2 && input.[0] = 0x1Buy && input.[1] = byte 'n' then
            ctx.Log(LogLevel.Debug, "New session shortcut triggered")
            // Trigger new session via remote command
            do! on_remote_command "session.new" ctx |> Async.Ignore
            return Ok (Action.Modify [||])

        // Alt+1..9 - Quick switch to session 1-9
        elif input.Length = 2 && input.[0] = 0x1Buy &&
             input.[1] >= byte '1' && input.[1] <= byte '9' then
            let index = int input.[1] - int '1'
            let sortedSessions =
                sessions.Values
                |> Seq.sortByDescending (fun s -> s.LastAccessed)
                |> Seq.toList

            if index < sortedSessions.Length then
                let targetSession = sortedSessions.[index]
                ctx.Log(LogLevel.Info, sprintf "Quick switching to session %d: %s" (index + 1) targetSession.Name)
                currentSessionId <- targetSession.Id
                updateSession targetSession.Id targetSession.Name ctx
                drawSessionIndicator ctx
                return Ok (Action.Modify [||])
            else
                return Ok Action.Continue
        else
            return Ok Action.Continue
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Save state before unloading
        saveSessionState ctx

        // Clear UI
        ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some 30000UL })
        ctx.QueueCommand(RemoteCommand.HideModal)

        ctx.Log(LogLevel.Info, "Session manager plugin unloaded")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None
    OnInput = Some on_input
    OnResize = Some on_resize
    OnAttach = Some on_attach
    OnDetach = Some on_detach
    OnPreCommand = None
    OnPostCommand = None
    OnRemoteCommand = Some on_remote_command
    GetCommands = getCommands
}
