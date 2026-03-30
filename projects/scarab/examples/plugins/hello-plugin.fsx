(*
 * hello-plugin.fsx
 *
 * Name: Hello World Plugin
 * Version: 1.0.0
 * Description: A simple example plugin that logs a message when loaded
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This is the simplest possible Scarab plugin. It demonstrates:
 * - Plugin metadata declaration
 * - The on_load hook
 * - Logging via the context
 *)

// Import the Scarab Plugin API module
open Scarab.PluginApi

// Plugin metadata - required for all plugins
let metadata = {
    Name = "hello-plugin"
    Version = "1.0.0"
    Description = "Simple hello world plugin"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// on_load hook - called when the plugin is first loaded
// This is where you initialize plugin state, load config, etc.
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        // Log a simple message
        ctx.Log(LogLevel.Info, "Hello from the hello-plugin!")
        ctx.Log(LogLevel.Info, "Plugin loaded successfully")

        // You can access terminal state
        let (cols, rows) = ctx.GetSize()
        ctx.Log(LogLevel.Debug, sprintf "Terminal size: %dx%d" cols rows)

        // You can read environment variables
        match ctx.GetEnv("USER") with
        | Some user -> ctx.Log(LogLevel.Info, sprintf "Running for user: %s" user)
        | None -> ()

        // Return success
        return Ok ()
    }

// on_unload hook - called when the plugin is being unloaded
// Clean up resources here
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Goodbye from hello-plugin!")
        return Ok ()
    }

// Export the plugin definition
// This makes the plugin discoverable by the Scarab daemon
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None      // Not handling output
    OnInput = None       // Not handling input
    OnResize = None      // Not handling resize
    OnAttach = None      // Not handling client attach
    OnDetach = None      // Not handling client detach
    OnPreCommand = None  // Not handling pre-command
    OnPostCommand = None // Not handling post-command
    OnRemoteCommand = None // Not handling remote commands
    GetCommands = fun () -> [] // No commands to register
}
