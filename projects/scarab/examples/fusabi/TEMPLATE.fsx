// @name plugin-template
// @version 0.1.0
// @description Template for creating new Fusabi plugins
// @author Your Name
// @homepage https://github.com/yourusername/scarab-plugin-name
// @license MIT
// @api-version 0.1.0
// @min-scarab-version 0.1.0

// =============================================================================
// Plugin Metadata
// =============================================================================
// This section defines metadata about your plugin that Scarab uses for
// discovery, compatibility checking, and display in the UI.

/// Plugin metadata record
let metadata = {
    name = "plugin-template"
    version = "0.1.0"
    description = "Template for creating new Fusabi plugins"
    author = "Your Name"
    homepage = Some "https://github.com/yourusername/scarab-plugin-name"
    api_version = "0.1.0"
    min_scarab_version = "0.1.0"
}

// =============================================================================
// Plugin Configuration
// =============================================================================
// Define any configuration options your plugin needs

type PluginConfig = {
    enabled: bool
    debug_mode: bool
    // Add your custom config fields here
}

let default_config = {
    enabled = true
    debug_mode = false
}

// Mutable state for your plugin (if needed)
let mutable config = default_config

// =============================================================================
// Lifecycle Hooks
// =============================================================================

/// Called when the plugin is loaded
/// Use this for initialization, registering commands, etc.
let on_load (ctx: PluginContext) =
    printfn "[%s] Plugin loaded successfully!" metadata.name

    // Register any commands your plugin provides
    // ctx.register_command "my-command" handle_my_command

    Ok ()

/// Called when the plugin is being unloaded
/// Clean up resources here
let on_unload () =
    printfn "[%s] Plugin unloading..." metadata.name
    Ok ()

// =============================================================================
// Event Hooks
// =============================================================================

/// Called before terminal output is displayed
/// Return Action.Continue to pass through, Action.Block to suppress, or Action.Modified to alter
let on_output (line: string) (ctx: PluginContext) =
    // Example: Log all output lines in debug mode
    if config.debug_mode then
        printfn "[%s] Output: %s" metadata.name line

    // Example: Detect error patterns
    // if line.Contains("ERROR") || line.Contains("error:") then
    //     ctx.notify "Error detected in output"

    Action.Continue

/// Called after user input is received
/// Return Action.Continue to pass through, Action.Block to suppress, or Action.Modified to alter
let on_input (input: byte[]) (ctx: PluginContext) =
    // Example: Log input in debug mode
    if config.debug_mode then
        let input_str = System.Text.Encoding.UTF8.GetString(input)
        printfn "[%s] Input: %s" metadata.name input_str

    // Example: Intercept specific key combinations
    // match input with
    // | [|27uy; 91uy; 65uy|] -> // Up arrow
    //     ctx.notify "Up arrow pressed"
    //     Action.Continue
    // | _ -> Action.Continue

    Action.Continue

/// Called before a command is executed
let on_pre_command (command: string) (ctx: PluginContext) =
    // Example: Log commands in debug mode
    if config.debug_mode then
        printfn "[%s] Executing command: %s" metadata.name command

    // Example: Warn about dangerous commands
    // if command.StartsWith("rm -rf") then
    //     ctx.warn "Dangerous command detected!"

    Action.Continue

/// Called after a command completes
let on_post_command (command: string) (exit_code: int) (ctx: PluginContext) =
    // Example: Log command completion
    if config.debug_mode then
        printfn "[%s] Command completed: %s (exit code: %d)" metadata.name command exit_code

    // Example: Notify on command failure
    // if exit_code <> 0 then
    //     ctx.notify (sprintf "Command failed with exit code %d" exit_code)

    Ok ()

/// Called when terminal is resized
let on_resize (cols: uint16) (rows: uint16) (ctx: PluginContext) =
    if config.debug_mode then
        printfn "[%s] Terminal resized: %dx%d" metadata.name cols rows

    Ok ()

/// Called when a client attaches to the session
let on_attach (client_id: uint64) (ctx: PluginContext) =
    printfn "[%s] Client attached: %d" metadata.name client_id
    Ok ()

/// Called when a client detaches from the session
let on_detach (client_id: uint64) (ctx: PluginContext) =
    printfn "[%s] Client detached: %d" metadata.name client_id
    Ok ()

/// Called when a remote command is triggered by the client
let on_remote_command (id: string) (ctx: PluginContext) =
    printfn "[%s] Remote command triggered: %s" metadata.name id

    // Handle your custom remote commands here
    // match id with
    // | "toggle-feature" -> toggle_feature ctx
    // | "show-status" -> show_status ctx
    // | _ -> Ok ()

    Ok ()

// =============================================================================
// Helper Functions
// =============================================================================

/// Example helper function
let log_message (level: string) (message: string) =
    printfn "[%s] [%s] %s" metadata.name level message

/// Example feature toggle
let toggle_feature () =
    config <- { config with debug_mode = not config.debug_mode }
    log_message "INFO" (sprintf "Debug mode: %b" config.debug_mode)

// =============================================================================
// Commands
// =============================================================================

/// Example command that can be invoked from Scarab
let commands = [
    // Define modal commands that show up in the command palette
    {
        id = "template.toggle-debug"
        label = "Template: Toggle Debug Mode"
        description = Some "Enable or disable debug logging"
        category = "Plugin"
        handler = fun () -> toggle_feature ()
    }
]

/// Export commands for registration
let get_commands () = commands

// =============================================================================
// Plugin Export
// =============================================================================

/// Main plugin export
/// This is what Scarab loads
let plugin = {
    metadata = metadata
    on_load = on_load
    on_unload = on_unload
    on_output = on_output
    on_input = on_input
    on_pre_command = on_pre_command
    on_post_command = on_post_command
    on_resize = on_resize
    on_attach = on_attach
    on_detach = on_detach
    on_remote_command = on_remote_command
    get_commands = get_commands
}
