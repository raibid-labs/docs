// Standard Scarab Configuration Example
//
// Usage: Copy to ~/.config/scarab/config.fsx

let terminal = {
    DefaultShell = "/bin/zsh";
    ScrollbackLines = 10000;
    AltScreen = true;
    ScrollMultiplier = 3.0;
    AutoScroll = true;
    Columns = 120;
    Rows = 40
}

let font = {
    Family = "JetBrains Mono";
    Size = 14.0;
    LineHeight = 1.2;
    Fallback = ("Fira Code", "Hack", "monospace");
    BoldIsBright = true;
    UseThinStrokes = false
}

let colors = {
    Theme = "dracula";
    Opacity = 0.95;
    DimOpacity = 0.7
}

let ui = {
    LinkHints = true;
    CommandPalette = true;
    Animations = true;
    SmoothScroll = true;
    ShowTabs = true;
    CursorBlink = true
}

// Return the full configuration object
{
    terminal = terminal;
    font = font;
    colors = colors;
    ui = ui
}