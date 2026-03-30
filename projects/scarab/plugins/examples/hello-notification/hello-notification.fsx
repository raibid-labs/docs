module hello_notification

open Scarab.PluginApi

[<Plugin>]
let metadata = {
    Name = "hello-notification"
    Version = "0.1.0"
    Description = "Shows a welcome notification when Scarab starts"
    Author = "Scarab Team"
    Homepage = Some "https://github.com/raibid-labs/scarab"
    ApiVersion = "0.1.0"
    MinScarabVersion = "0.1.0"
    Emoji = Some "👋"
    Color = Some "#4CAF50"
    Catchphrase = Some "Welcome to your terminal!"
}

// List of greeting messages
let greetings = [
    "Welcome back!"
    "Ready to get things done?"
    "Let's make something awesome!"
    "Your terminal, supercharged!"
    "Happy coding!"
    "Time to be productive!"
]

// Get random greeting
let getRandomGreeting () =
    let random = System.Random()
    let index = random.Next(List.length greetings)
    List.item index greetings

// Get time-based greeting
let getTimeBasedGreeting () =
    let hour = System.DateTime.Now.Hour
    match hour with
    | h when h < 5 -> "Burning the midnight oil?"
    | h when h < 12 -> "Good morning!"
    | h when h < 17 -> "Good afternoon!"
    | h when h < 21 -> "Good evening!"
    | _ -> "Working late tonight?"

[<OnLoad>]
let onLoad (ctx: PluginContext) =
    async {
        // Log plugin load
        ctx.Log Info "Hello Notification plugin loaded!"

        // Get username from environment
        let username =
            match ctx.GetEnv "USER" with
            | Some user -> user
            | None -> "friend"

        // Combine time-based and random greetings
        let greeting = getTimeBasedGreeting ()
        let message = getRandomGreeting ()

        // Show personalized notification
        ctx.NotifySuccess
            (sprintf "%s %s" greeting username)
            message

        return Ok ()
    }
