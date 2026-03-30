// @name ui-overlay
// @version 0.1.0
// @description UI overlay example plugin
// @author Scarab Team
// @api-version 0.1.0
// @min-scarab-version 0.1.0

// UI Overlay Example
// Demonstrates how to create custom UI elements in the client

type OverlayElement =
    | StatusBar of string
    | Notification of string * int
    | CommandPalette of string list
    | SearchBox of string

type Position = { x: int; y: int }
type Size = { width: int; height: int }

type Overlay = {
    element: OverlayElement
    position: Position
    size: Size
    visible: bool
}

// Status bar overlay
let status_bar = {
    element = StatusBar "Scarab Terminal | Ctrl+Shift+P for commands"
    position = { x = 0; y = 0 }
    size = { width = 100; height = 1 }
    visible = true
}

// Command palette
let commands = [
    "File: New Tab"
    "File: Close Tab"
    "Edit: Copy"
    "Edit: Paste"
    "View: Toggle Sidebar"
    "Terminal: Clear"
    "Terminal: Split Horizontal"
    "Terminal: Split Vertical"
]

let command_palette = {
    element = CommandPalette commands
    position = { x = 20; y = 10 }
    size = { width = 60; height = 15 }
    visible = false
}

// Toggle command palette visibility
let mutable palette_visible = false

let toggle_command_palette () =
    palette_visible <- not palette_visible
    printfn "Command palette: %s" (if palette_visible then "visible" else "hidden")

// Show notification
let show_notification message duration =
    let notification = {
        element = Notification (message, duration)
        position = { x = 5; y = 2 }
        size = { width = 50; height = 3 }
        visible = true
    }
    printfn "Showing notification: %s" message

// Handle remote commands from client
let on_remote_command (command_id: string) =
    match command_id with
    | "toggle_palette" ->
        toggle_command_palette ()
    | "show_help" ->
        show_notification "Press Ctrl+Shift+P for commands" 3000
    | _ ->
        printfn "Unknown remote command: %s" command_id
