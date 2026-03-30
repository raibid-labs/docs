// Minimal Scarab Configuration Example
//
// Usage: Copy to ~/.config/scarab/config.fsx

// Define terminal settings
let terminal = {
    DefaultShell = "/bin/bash";
    ScrollbackLines = 5000;
    Columns = 100;
    Rows = 30
}

// Define font settings
let font = {
    Family = "Monospace";
    Size = 12.0
}

// Return the configuration record
// Only defined sections are overridden; others use defaults.
{
    terminal = terminal;
    font = font
}