module clipboard_history

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "clipboard-history"
    Version = "0.1.0"
    Description = "Tracks clipboard history and provides quick access"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "📋"
    Color = Some "#607D8B"
    Catchphrase = Some "Never lose what you copied!"
}

// Maximum history items to store
let maxHistorySize = 50

// Get clipboard history from storage
let getHistory (ctx: PluginContext) : string list =
    match ctx.GetData "clipboard_history" with
    | Some historyStr ->
        historyStr.Split('\n')
        |> Array.toList
        |> List.filter (fun s -> not (String.IsNullOrWhiteSpace(s)))
    | None -> []

// Save clipboard history to storage
let saveHistory (ctx: PluginContext) (history: string list) =
    let historyStr = String.concat "\n" (List.take (min maxHistorySize (List.length history)) history)
    ctx.SetData "clipboard_history" historyStr

// Add item to history
let addToHistory (ctx: PluginContext) (item: string) =
    let history = getHistory ctx
    let newHistory = item :: (List.filter ((<>) item) history)
    saveHistory ctx newHistory
    List.length newHistory

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Clipboard History plugin loaded!"

        // Initialize empty history if needed
        if ctx.GetData "clipboard_history" = None then
            ctx.SetData "clipboard_history" ""

        return Ok ()
    }
