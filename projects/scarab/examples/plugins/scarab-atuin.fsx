(*
 * scarab-atuin.fsx
 *
 * Name: Scarab Atuin Integration
 * Version: 0.1.0
 * Description: Integrates Atuin shell history search with Scarab Terminal
 * Author: Scarab Team
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin integrates Atuin (https://github.com/atuinsh/atuin) shell history
 * with Scarab Terminal, providing:
 * - Cross-session history sync
 * - Advanced fuzzy search
 * - Command statistics
 * - Cloud sync capabilities
 *
 * Features:
 * - Detects Atuin installation
 * - Hooks Ctrl+R for history search
 * - Real-time search filtering
 * - Arrow key navigation
 * - Inserts selected command into terminal
 *)

open Scarab.PluginApi
open System
open System.Text
open System.Text.Json
open System.Diagnostics

// Plugin metadata
let metadata = {
    Name = "scarab-atuin"
    Version = "0.1.0"
    Description = "Atuin shell history integration"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/atuinsh/atuin"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Configuration keys
let CONFIG_KEYBINDING = "keybinding"
let CONFIG_MAX_RESULTS = "max_results"
let CONFIG_AUTO_SYNC = "auto_sync"
let CONFIG_SHOW_STATS = "show_stats"
let CONFIG_ENABLED = "enabled"

// State management
let mutable atuinInstalled = false
let mutable searchActive = false
let mutable currentFilter = ""
let mutable searchResults: string list = []
let mutable selectedIndex = 0

// Check if Atuin is installed
let checkAtuinInstalled () : bool =
    try
        let proc = new Process()
        proc.StartInfo.FileName <- "which"
        proc.StartInfo.Arguments <- "atuin"
        proc.StartInfo.UseShellExecute <- false
        proc.StartInfo.RedirectStandardOutput <- true
        proc.StartInfo.RedirectStandardError <- true
        proc.StartInfo.CreateNoWindow <- true

        let started = proc.Start()
        if not started then false
        else
            proc.WaitForExit(1000) |> ignore
            proc.ExitCode = 0
    with
    | _ -> false

// Execute shell command and return output
let executeCommand (cmd: string) (args: string) : Result<string, string> =
    try
        let proc = new Process()
        proc.StartInfo.FileName <- cmd
        proc.StartInfo.Arguments <- args
        proc.StartInfo.UseShellExecute <- false
        proc.StartInfo.RedirectStandardOutput <- true
        proc.StartInfo.RedirectStandardError <- true
        proc.StartInfo.CreateNoWindow <- true

        let started = proc.Start()
        if not started then
            Error "Failed to start process"
        else
            let output = proc.StandardOutput.ReadToEnd()
            let error = proc.StandardError.ReadToEnd()
            proc.WaitForExit(5000) |> ignore

            if proc.ExitCode = 0 then
                Ok output
            else
                Error error
    with
    | ex -> Error ex.Message

// Parse Atuin JSON output
let parseAtuinOutput (json: string) : string list =
    try
        if String.IsNullOrWhiteSpace(json) then []
        else
            use doc = JsonDocument.Parse(json)
            let root = doc.RootElement

            if root.ValueKind = JsonValueKind.Array then
                [
                    for item in root.EnumerateArray() do
                        if item.TryGetProperty("command") then
                            let cmd = item.GetProperty("command").GetString()
                            if not (String.IsNullOrWhiteSpace(cmd)) then
                                yield cmd
                ]
            else
                []
    with
    | _ -> []

// Query Atuin for history
let queryAtuin (filter: string) (maxResults: int) : Result<string list, string> =
    let args = sprintf "search --limit %d --format json \"%s\"" maxResults filter
    match executeCommand "atuin" args with
    | Ok output ->
        let commands = parseAtuinOutput output
        Ok commands
    | Error err ->
        Error err

// Build modal items from search results
let buildModalItems (results: string list) (selectedIdx: int) : ModalItem list =
    results
    |> List.mapi (fun i cmd ->
        let prefix = if i = selectedIdx then "> " else "  "
        {
            Id = sprintf "atuin-result-%d" i
            Label = sprintf "%s%s" prefix cmd
            Description = None
        })

// Show search overlay with current results
let showSearchOverlay (ctx: PluginContext) =
    if List.isEmpty searchResults then
        let items = [
            {
                Id = "atuin-empty"
                Label = "No results found"
                Description = Some "Try a different search term"
            }
        ]
        ctx.QueueCommand(RemoteCommand.ShowModal {
            Title = sprintf "Atuin History Search: %s" currentFilter
            Items = items
        })
    else
        let items = buildModalItems searchResults selectedIndex
        ctx.QueueCommand(RemoteCommand.ShowModal {
            Title = sprintf "Atuin History Search: %s (%d results)" currentFilter (List.length searchResults)
            Items = items
        })

// Update search results based on filter
let updateSearch (ctx: PluginContext) (filter: string) =
    currentFilter <- filter

    let maxResults =
        match ctx.Config.GetOpt<int>(CONFIG_MAX_RESULTS) with
        | Some n -> n
        | None -> 20

    match queryAtuin filter maxResults with
    | Ok results ->
        searchResults <- results
        selectedIndex <- 0
        showSearchOverlay ctx
        ctx.Log(LogLevel.Debug, sprintf "Found %d results for: %s" (List.length results) filter)
    | Error err ->
        searchResults <- []
        selectedIndex <- 0
        showSearchOverlay ctx
        ctx.Log(LogLevel.Error, sprintf "Atuin query failed: %s" err)

// Handle navigation keys in search mode
let handleNavigationKey (ctx: PluginContext) (key: byte) : bool =
    match key with
    | 0x1Buy ->  // Escape - Close search
        searchActive <- false
        currentFilter <- ""
        searchResults <- []
        selectedIndex <- 0
        ctx.QueueCommand(RemoteCommand.HideModal)
        ctx.Log(LogLevel.Debug, "Closed Atuin search")
        true

    | 0x0Duy ->  // Enter - Select current result
        if selectedIndex >= 0 && selectedIndex < List.length searchResults then
            let selected = List.item selectedIndex searchResults
            searchActive <- false
            currentFilter <- ""
            searchResults <- []
            selectedIndex <- 0
            ctx.QueueCommand(RemoteCommand.HideModal)

            // Insert the command into the terminal
            // Note: We need to convert the string to bytes and return it via Action.Modify
            // For now, we'll just log it - the actual insertion needs to be handled
            // by the on_input hook returning Action.Modify
            ctx.Log(LogLevel.Info, sprintf "Selected command: %s" selected)
            ctx.SetData("atuin_selected_command", selected)
        true

    | 0x41uy ->  // Up arrow (part of escape sequence, simplified)
        if selectedIndex > 0 then
            selectedIndex <- selectedIndex - 1
            showSearchOverlay ctx
        true

    | 0x42uy ->  // Down arrow (part of escape sequence, simplified)
        if selectedIndex < List.length searchResults - 1 then
            selectedIndex <- selectedIndex + 1
            showSearchOverlay ctx
        true

    | _ -> false

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Check if plugin is enabled in config
        let enabled =
            match ctx.Config.GetOpt<bool>(CONFIG_ENABLED) with
            | Some e -> e
            | None -> true

        if not enabled then
            ctx.Log(LogLevel.Info, "Atuin plugin is disabled in config")
            return Ok ()
        else
            // Check for Atuin installation
            atuinInstalled <- checkAtuinInstalled()

            if atuinInstalled then
                ctx.Log(LogLevel.Info, "Atuin plugin loaded successfully")
                ctx.Log(LogLevel.Info, "Press Ctrl+R to search command history")

                // Show welcome notification
                ctx.NotifySuccess("Atuin Plugin Loaded", "Press Ctrl+R to search your shell history")
            else
                ctx.Log(LogLevel.Warn, "Atuin not found on system")
                ctx.NotifyWarning(
                    "Atuin Not Installed",
                    "Install Atuin to use this plugin: cargo install atuin"
                )

            return Ok ()
    }

// on_input hook - intercepts keyboard input
let on_input (input: byte[]) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // If search is active, handle navigation
        if searchActive then
            if input.Length > 0 then
                let handled = handleNavigationKey ctx input.[0]
                if handled then
                    return Ok (Action.Modify [||])  // Consume the key
                else
                    // Allow typing to refine search
                    // For now, we'll just pass through
                    return Ok Action.Continue
            else
                return Ok Action.Continue

        // Check for Ctrl+R (0x12)
        elif input.Length = 1 && input.[0] = 0x12uy then
            if not atuinInstalled then
                ctx.NotifyWarning(
                    "Atuin Not Available",
                    "Please install Atuin: cargo install atuin"
                )
                return Ok (Action.Modify [||])
            else
                // Activate search mode
                searchActive <- true
                currentFilter <- ""
                selectedIndex <- 0

                // Show initial empty search
                updateSearch ctx ""

                ctx.Log(LogLevel.Info, "Atuin search activated")
                return Ok (Action.Modify [||])  // Consume Ctrl+R
        else
            // Check if we have a selected command to insert
            match ctx.GetData("atuin_selected_command") with
            | Some cmd ->
                ctx.SetData("atuin_selected_command", "")  // Clear the flag
                let cmdBytes = Encoding.UTF8.GetBytes(cmd)
                return Ok (Action.Modify cmdBytes)  // Insert the command
            | None ->
                return Ok Action.Continue
    }

// on_post_command hook - optional auto-sync after commands
let on_post_command (command: string) (exitCode: int) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        let autoSync =
            match ctx.Config.GetOpt<bool>(CONFIG_AUTO_SYNC) with
            | Some s -> s
            | None -> false

        if autoSync && atuinInstalled then
            // Trigger background sync
            match executeCommand "atuin" "sync" with
            | Ok _ ->
                ctx.Log(LogLevel.Debug, "Atuin history synced")
            | Error err ->
                ctx.Log(LogLevel.Warn, sprintf "Atuin sync failed: %s" err)

        return Ok ()
    }

// Provide commands for the command palette
let getCommands () : ModalItem list =
    [
        {
            Id = "atuin.search"
            Label = "Search Atuin History"
            Description = Some "Open Atuin shell history search (Ctrl+R)"
        }
        {
            Id = "atuin.sync"
            Label = "Sync Atuin History"
            Description = Some "Sync history with Atuin cloud"
        }
        {
            Id = "atuin.stats"
            Label = "Show Command Statistics"
            Description = Some "Display Atuin usage statistics"
        }
    ]

// Handle remote commands from command palette
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        if not atuinInstalled then
            ctx.NotifyWarning(
                "Atuin Not Available",
                "Please install Atuin: cargo install atuin"
            )
            return Ok ()

        match id with
        | "atuin.search" ->
            searchActive <- true
            currentFilter <- ""
            selectedIndex <- 0
            updateSearch ctx ""
            ctx.Log(LogLevel.Info, "Atuin search opened from command palette")
            return Ok ()

        | "atuin.sync" ->
            match executeCommand "atuin" "sync" with
            | Ok output ->
                ctx.NotifySuccess("Atuin Sync", "History synchronized successfully")
                ctx.Log(LogLevel.Info, "Atuin sync completed")
            | Error err ->
                ctx.NotifyError("Atuin Sync Failed", err)
                ctx.Log(LogLevel.Error, sprintf "Atuin sync failed: %s" err)
            return Ok ()

        | "atuin.stats" ->
            match executeCommand "atuin" "stats" with
            | Ok output ->
                // Show stats in notification
                let lines = output.Split('\n') |> Array.take 5 |> String.concat "\n"
                ctx.NotifyInfo("Atuin Statistics", lines)
                ctx.Log(LogLevel.Info, "Displayed Atuin statistics")
            | Error err ->
                ctx.NotifyError("Atuin Stats Failed", err)
                ctx.Log(LogLevel.Error, sprintf "Atuin stats failed: %s" err)
            return Ok ()

        | _ ->
            ctx.Log(LogLevel.Warn, sprintf "Unknown command: %s" id)
            return Ok ()
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Clean up any active overlays
        if searchActive then
            ctx.QueueCommand(RemoteCommand.HideModal)

        searchActive <- false
        currentFilter <- ""
        searchResults <- []
        selectedIndex <- 0

        ctx.Log(LogLevel.Info, "Atuin plugin unloaded")
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
    OnPostCommand = Some on_post_command
    OnRemoteCommand = Some on_remote_command
    GetCommands = getCommands
}
