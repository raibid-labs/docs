// @name keybindings
// @version 0.1.0
// @description Custom keybindings plugin
// @author Scarab Team
// @api-version 0.1.0
// @min-scarab-version 0.1.0

// Custom Keybindings Example
// Demonstrates how to intercept and handle keyboard input

type KeyModifier =
    | None
    | Ctrl
    | Alt
    | Shift
    | CtrlShift
    | AltShift

type KeyBinding = {
    key: string
    modifier: KeyModifier
    action: string
}

// Define custom keybindings
let bindings = [
    { key = "t"; modifier = Ctrl; action = "new_tab" }
    { key = "w"; modifier = Ctrl; action = "close_tab" }
    { key = "n"; modifier = CtrlShift; action = "new_window" }
    { key = "f"; modifier = Ctrl; action = "search" }
    { key = "p"; modifier = CtrlShift; action = "command_palette" }
    { key = "k"; modifier = Ctrl; action = "clear_screen" }
]

// Handle key input
let handle_key key modifier =
    let binding = bindings
                  |> List.tryFind (fun b -> b.key = key && b.modifier = modifier)

    match binding with
    | Some b ->
        printfn "Executing action: %s" b.action
        true
    | None ->
        false

// Parse key sequence from bytes
let parse_key_sequence (bytes: byte[]) =
    // Detect Ctrl+T (0x14)
    if bytes.Length = 1 && bytes.[0] = 0x14uy then
        Some ("t", Ctrl)
    // Detect Ctrl+W (0x17)
    elif bytes.Length = 1 && bytes.[0] = 0x17uy then
        Some ("w", Ctrl)
    // Detect Ctrl+K (0x0B)
    elif bytes.Length = 1 && bytes.[0] = 0x0Buy then
        Some ("k", Ctrl)
    else
        None

// Input handler hook
let on_input (input: byte[]) =
    match parse_key_sequence input with
    | Some (key, modifier) ->
        if handle_key key modifier then
            printfn "Key handled by plugin"
        else
            printfn "Key not bound"
    | None ->
        printfn "Unknown key sequence"
