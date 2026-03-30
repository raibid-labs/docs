(*
 * git-status.fsx
 *
 * Name: Git Status Plugin
 * Version: 1.0.0
 * Description: Shows git repository status indicators
 * Author: Scarab Examples
 * API Version: 0.1.0
 * Min Scarab Version: 0.1.0
 *
 * This plugin demonstrates:
 * - Detecting command execution (pre/post command hooks)
 * - Running external processes
 * - Drawing persistent status overlays
 * - Environment-aware plugin behavior
 *)

open Scarab.PluginApi
open System
open System.Diagnostics
open System.IO

// Plugin metadata
let metadata = {
    Name = "git-status"
    Version = "1.0.0"
    Description = "Display git repository status in terminal"
    Author = "Scarab Examples"
    Homepage = Some "https://github.com/scarab-terminal/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
}

// Mutable state to cache git status
let mutable currentBranch = ""
let mutable isDirty = false
let mutable isGitRepo = false

// Check if we're in a git repository
let checkGitRepo () : bool =
    try
        let psi = ProcessStartInfo("git", "rev-parse --git-dir")
        psi.RedirectStandardOutput <- true
        psi.RedirectStandardError <- true
        psi.UseShellExecute <- false
        psi.CreateNoWindow <- true

        use proc = Process.Start(psi)
        proc.WaitForExit()
        proc.ExitCode = 0
    with
    | _ -> false

// Get current git branch
let getGitBranch () : string option =
    try
        let psi = ProcessStartInfo("git", "rev-parse --abbrev-ref HEAD")
        psi.RedirectStandardOutput <- true
        psi.RedirectStandardError <- true
        psi.UseShellExecute <- false
        psi.CreateNoWindow <- true

        use proc = Process.Start(psi)
        let branch = proc.StandardOutput.ReadLine()
        proc.WaitForExit()

        if proc.ExitCode = 0 && not (String.IsNullOrWhiteSpace(branch)) then
            Some (branch.Trim())
        else
            None
    with
    | _ -> None

// Check if working directory is dirty
let isGitDirty () : bool =
    try
        let psi = ProcessStartInfo("git", "status --porcelain")
        psi.RedirectStandardOutput <- true
        psi.RedirectStandardError <- true
        psi.UseShellExecute <- false
        psi.CreateNoWindow <- true

        use proc = Process.Start(psi)
        let output = proc.StandardOutput.ReadToEnd()
        proc.WaitForExit()

        not (String.IsNullOrWhiteSpace(output))
    with
    | _ -> false

// Update git status cache
let updateGitStatus (ctx: PluginContext) =
    isGitRepo <- checkGitRepo()

    if isGitRepo then
        match getGitBranch() with
        | Some branch ->
            currentBranch <- branch
            isDirty <- isGitDirty()
            ctx.Log(LogLevel.Debug, sprintf "Git status: branch=%s, dirty=%b" currentBranch isDirty)
        | None ->
            currentBranch <- ""
            isDirty <- false
    else
        currentBranch <- ""
        isDirty <- false

// Draw git status overlay
let drawGitStatus (ctx: PluginContext) =
    if isGitRepo && currentBranch <> "" then
        let statusSymbol = if isDirty then "*" else ""
        let statusText = sprintf " git:%s%s " currentBranch statusSymbol

        // Get terminal size to position in top-right
        let (cols, rows) = ctx.GetSize()
        let xPos = cols - uint16 statusText.Length

        let (fg, bg) =
            if isDirty then
                (0xFFFFFFFFu, 0xFFA500FFu)  // Orange background for dirty
            else
                (0xFFFFFFFFu, 0x00AA00FFu)  // Green background for clean

        ctx.QueueCommand(RemoteCommand.DrawOverlay {
            Id = 10000UL  // Fixed ID for git status
            X = xPos
            Y = 0us
            Text = statusText
            Style = {
                Fg = fg
                Bg = bg
                ZIndex = 100.0f
            }
        })

// on_load hook
let on_load (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Info, "Git status plugin loaded")

        // Initial git status check
        updateGitStatus ctx
        drawGitStatus ctx

        return Ok ()
    }

// on_pre_command hook - detect git commands
let on_pre_command (command: string) (ctx: PluginContext) : Async<Result<Action, string>> =
    async {
        // Check if command starts with git
        let trimmedCmd = command.Trim()
        if trimmedCmd.StartsWith("git ") || trimmedCmd = "git" then
            ctx.Log(LogLevel.Debug, sprintf "Git command detected: %s" command)

        return Ok Action.Continue
    }

// on_post_command hook - update status after commands
let on_post_command (command: string) (exitCode: int) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        let trimmedCmd = command.Trim()

        // Update after git commands or cd (might change repo)
        if trimmedCmd.StartsWith("git ") || trimmedCmd = "git" || trimmedCmd.StartsWith("cd ") then
            ctx.Log(LogLevel.Debug, sprintf "Updating git status after: %s" command)

            // Small delay to ensure file system is updated
            do! Async.Sleep(100)

            updateGitStatus ctx
            drawGitStatus ctx

        return Ok ()
    }

// on_resize hook - reposition status overlay
let on_resize (cols: uint16) (rows: uint16) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.Log(LogLevel.Debug, sprintf "Terminal resized to %dx%d" cols rows)

        // Redraw git status in new position
        drawGitStatus ctx

        return Ok ()
    }

// Provide commands for the command palette
let getCommands () : ModalItem list =
    [
        { Id = "git.refresh"; Label = "Refresh Git Status"; Description = Some "Manually refresh git repository status" }
        { Id = "git.hide"; Label = "Hide Git Status"; Description = Some "Hide the git status indicator" }
        { Id = "git.show"; Label = "Show Git Status"; Description = Some "Show the git status indicator" }
    ]

// Handle remote command from command palette
let on_remote_command (id: string) (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        match id with
        | "git.refresh" ->
            ctx.Log(LogLevel.Info, "Manually refreshing git status")
            updateGitStatus ctx
            drawGitStatus ctx

        | "git.hide" ->
            ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some 10000UL })
            ctx.Log(LogLevel.Info, "Git status hidden")

        | "git.show" ->
            updateGitStatus ctx
            drawGitStatus ctx
            ctx.Log(LogLevel.Info, "Git status shown")

        | _ ->
            ctx.Log(LogLevel.Warn, sprintf "Unknown command: %s" id)

        return Ok ()
    }

// on_unload hook
let on_unload (ctx: PluginContext) : Async<Result<unit, string>> =
    async {
        ctx.QueueCommand(RemoteCommand.ClearOverlays { Id = Some 10000UL })
        ctx.Log(LogLevel.Info, "Git status plugin unloaded")
        return Ok ()
    }

// Export the plugin
Plugin.Register {
    Metadata = metadata
    OnLoad = on_load
    OnUnload = on_unload
    OnOutput = None
    OnInput = None
    OnResize = Some on_resize
    OnAttach = None
    OnDetach = None
    OnPreCommand = Some on_pre_command
    OnPostCommand = Some on_post_command
    OnRemoteCommand = Some on_remote_command
    GetCommands = getCommands
}
