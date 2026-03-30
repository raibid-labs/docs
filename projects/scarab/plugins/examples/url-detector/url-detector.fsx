module url_detector

open Scarab.PluginApi
open System.Text.RegularExpressions

[<Plugin>]
let metadata = {
    Name = "url-detector"
    Version = "0.1.0"
    Description = "Detects URLs in terminal output and highlights them"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "🔗"
    Color = Some "#2196F3"
    Catchphrase = Some "Never miss a link!"
}

// URL detection regex
let urlPattern = @"https?://[^\s<>""{}|\\^`\[\]]+"
let urlRegex = Regex(urlPattern, RegexOptions.Compiled)

// URLs to ignore (localhost, common false positives)
let ignoreUrls = Set.ofList [
    "http://localhost"
    "https://localhost"
    "http://127.0.0.1"
    "https://127.0.0.1"
]

// Check if line contains a URL
let containsUrl (text: string) : bool =
    urlRegex.IsMatch(text)

// Extract all URLs from text
let extractUrls (text: string) : string list =
    urlRegex.Matches(text)
    |> Seq.cast<Match>
    |> Seq.map (fun m -> m.Value)
    |> Seq.toList

// Check if URL should be reported
let shouldReport (url: string) : bool =
    not (Set.exists (fun ignore -> url.StartsWith(ignore)) ignoreUrls)

// Shorten URL for display
let shortenUrl (url: string) (maxLen: int) : string =
    if url.Length <= maxLen then
        url
    else
        url.Substring(0, maxLen - 3) + "..."

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        ctx.Log Info "URL Detector plugin loaded!"
        return Ok ()
    }

[<OnOutput>]
let onOutput (ctx: PluginContext) (text: string) =
    async {
        if containsUrl text then
            let urls =
                extractUrls text
                |> List.filter shouldReport

            if not (List.isEmpty urls) then
                let urlCount = List.length urls

                // Log detection
                ctx.Log Debug (sprintf "Detected %d URL(s): %s" urlCount (String.concat ", " urls))

                // Show notification
                let message =
                    match urls with
                    | [url] -> shortenUrl url 60
                    | multiple ->
                        let displayUrls = List.take (min 3 (List.length multiple)) multiple
                        String.concat "\n" (List.map (fun u -> shortenUrl u 40) displayUrls)

                ctx.NotifyInfo "URL Detected" message

        return Continue
    }
