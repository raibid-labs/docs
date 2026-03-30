module git_status

open Scarab.PluginApi
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "git-status"
    Version = "0.1.0"
    Description = "Shows git branch and status information in the prompt"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🌿"
    Color = Some "#F05032"
    Catchphrase = Some "Git smart, code smarter!"
}

// Git command patterns
let branchPattern = @"On branch (\S+)"
let checkoutPattern = @"Switched to (?:a new )?branch '([^']+)'"
let statusPattern = @"Your branch is (up to date|ahead|behind)"

let branchRegex = Regex(branchPattern, RegexOptions.Compiled)
let checkoutRegex = Regex(checkoutPattern, RegexOptions.Compiled)
let statusRegex = Regex(statusPattern, RegexOptions.Compiled)

// Parse branch from output
let parseBranch (line: string) : string option =
    let branchMatch = branchRegex.Match(line)
    let checkoutMatch = checkoutRegex.Match(line)

    if branchMatch.Success then
        Some branchMatch.Groups.[1].Value
    elif checkoutMatch.Success then
        Some checkoutMatch.Groups.[1].Value
    else
        None

// Parse status from output
let parseStatus (line: string) : string option =
    let statusMatch = statusRegex.Match(line)

    if statusMatch.Success then
        Some statusMatch.Groups.[1].Value
    else
        None

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "Git Status plugin loaded!"

        // Try to get initial branch from git
        ctx.SetData "git_branch" "unknown"
        ctx.SetData "git_status" "unknown"

        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (line: string) =
    async {
        // Parse branch changes
        match parseBranch line with
        | Some branch ->
            ctx.SetData "git_branch" branch
            ctx.Log Info (sprintf "Git branch: %s" branch)

            // Send to frontend to update status bar
            ctx.NotifyInfo "Git Branch" (sprintf "⎇ %s" branch)

        | None -> ()

        // Parse status changes
        match parseStatus line with
        | Some status ->
            ctx.SetData "git_status" status
            ctx.Log Debug (sprintf "Git status: %s" status)

        | None -> ()

        return Continue
    }

[<OnPreCommand>]
let onPreCommand (ctx: PluginContext) (command: string) =
    async {
        // Detect git commands
        if command.StartsWith("git ") then
            ctx.Log Debug (sprintf "Git command detected: %s" command)

        return Continue
    }
