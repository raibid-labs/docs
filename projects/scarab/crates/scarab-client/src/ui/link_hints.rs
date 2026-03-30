// Vimium-style link hints for clickable URLs and file paths
// Detects links in terminal output and provides keyboard shortcuts

use crate::integration::SharedMemoryReader;
use crate::rendering::text::TextRenderer;
use crate::ui::grid_utils::grid_to_pixel;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use regex::Regex;
use scarab_protocol::{SharedState, GRID_HEIGHT, GRID_WIDTH};

/// Plugin for link hint functionality
pub struct LinkHintsPlugin;

impl Plugin for LinkHintsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LinkDetector>()
            .init_resource::<LinkHintsState>()
            .add_event::<LinkActivatedEvent>()
            .add_systems(
                Update,
                (
                    detect_links_system,
                    show_hints_system,
                    handle_hint_input_system,
                    activate_link_system,
                )
                    .chain(),
            );
    }
}

/// Detected link in terminal output with grid position
#[derive(Component, Clone, Debug)]
pub struct LinkHint {
    pub url: String,
    pub position: Vec2,
    pub grid_col: u16,
    pub grid_row: u16,
    pub hint_key: String,
    pub link_type: LinkType,
}

/// Type of detected link
#[derive(Clone, Debug, PartialEq)]
pub enum LinkType {
    Url,
    FilePath,
    Email,
}

/// Link detector with regex patterns
#[derive(Resource)]
pub struct LinkDetector {
    url_regex: Regex,
    filepath_regex: Regex,
    email_regex: Regex,
}

impl Default for LinkDetector {
    fn default() -> Self {
        Self {
            // Match HTTP(S) URLs
            url_regex: Regex::new(r"https?://[^\s<>{}|\^~\[\]`]+|www\.[^\s<>{}|\^~\[\]`]+")
                .unwrap(),
            // Match absolute and relative file paths
            filepath_regex: Regex::new(r"(?:~|\.{1,2}|/)?(?:[a-zA-Z0-9_\-./]+/)*[a-zA-Z0-9_\-.]+")
                .unwrap(),
            // Match email addresses
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
        }
    }
}

impl LinkDetector {
    /// Detect all links in text content with their grid positions
    pub fn detect_with_positions(&self, text: &str) -> Vec<(String, LinkType, usize, usize)> {
        let mut links = Vec::new();

        // Split text into lines and track positions
        for (row, line) in text.lines().enumerate() {
            // Detect URLs
            for m in self.url_regex.find_iter(line) {
                links.push((m.as_str().to_string(), LinkType::Url, m.start(), row));
            }

            // Detect file paths (with basic validation)
            for m in self.filepath_regex.find_iter(line) {
                let path = m.as_str();
                // Filter out very short or unlikely paths
                if path.len() > 3 && (path.contains('/') || path.contains('.')) {
                    links.push((path.to_string(), LinkType::FilePath, m.start(), row));
                }
            }

            // Detect emails
            for m in self.email_regex.find_iter(line) {
                links.push((m.as_str().to_string(), LinkType::Email, m.start(), row));
            }
        }

        links
    }

    /// Detect all links in text content (legacy API without positions)
    pub fn detect(&self, text: &str) -> Vec<(String, LinkType)> {
        self.detect_with_positions(text)
            .into_iter()
            .map(|(url, link_type, _, _)| (url, link_type))
            .collect()
    }

    /// Generate hint keys (aa, ab, ac, ..., ba, bb, ...)
    pub fn generate_hint_keys(count: usize) -> Vec<String> {
        let chars = "abcdefghijklmnopqrstuvwxyz";
        let mut keys = Vec::new();

        for i in 0..count {
            let first = i / 26;
            let second = i % 26;
            let key = if first == 0 {
                chars.chars().nth(second).unwrap().to_string()
            } else {
                format!(
                    "{}{}",
                    chars.chars().nth(first - 1).unwrap(),
                    chars.chars().nth(second).unwrap()
                )
            };
            keys.push(key);
        }

        keys
    }
}

/// State of link hints system
#[derive(Resource, Default)]
pub struct LinkHintsState {
    pub active: bool,
    pub hints: Vec<LinkHint>,
    pub current_input: String,
}

/// Event fired when a link is activated
#[derive(Event)]
pub struct LinkActivatedEvent {
    pub link: LinkHint,
}

/// Component for hint label display
#[derive(Component)]
struct HintLabel {
    hint_key: String,
}

/// Extract text from terminal grid via SharedMemoryReader
fn extract_terminal_text(state_reader: &SharedMemoryReader) -> String {
    let shared_ptr = state_reader.shmem.0.as_ptr() as *const SharedState;

    unsafe {
        let state = &*shared_ptr;
        crate::integration::extract_grid_text(state)
    }
}

/// Detect links in terminal grid with accurate pixel positioning
fn detect_links_system(
    detector: Res<LinkDetector>,
    mut state: ResMut<LinkHintsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    state_reader: Res<SharedMemoryReader>,
    renderer: Res<TextRenderer>,
) {
    // Toggle link hints with Ctrl+K
    if keyboard.just_pressed(KeyCode::ControlLeft) && keyboard.pressed(KeyCode::KeyK) {
        state.active = !state.active;

        if state.active {
            // Get actual terminal text from SharedState
            let terminal_text = extract_terminal_text(&state_reader);

            // Detect links with their grid positions
            let detected_links = detector.detect_with_positions(&terminal_text);
            let hint_keys = LinkDetector::generate_hint_keys(detected_links.len());

            // Get cell dimensions from renderer for accurate positioning
            let cell_width = renderer.cell_width;
            let cell_height = renderer.cell_height;

            state.hints = detected_links
                .into_iter()
                .zip(hint_keys)
                .map(|((url, link_type, col, row), hint_key)| {
                    // Convert grid coordinates to pixel position using grid_to_pixel
                    let position = grid_to_pixel(col as u16, row as u16, cell_width, cell_height);

                    LinkHint {
                        url,
                        position,
                        grid_col: col as u16,
                        grid_row: row as u16,
                        hint_key,
                        link_type,
                    }
                })
                .collect();

            info!("Detected {} links in terminal output", state.hints.len());
            state.current_input.clear();
        } else {
            state.hints.clear();
            state.current_input.clear();
        }
    }
}

/// Show hint labels on screen
fn show_hints_system(
    mut commands: Commands,
    state: Res<LinkHintsState>,
    existing_hints: Query<Entity, With<HintLabel>>,
) {
    // Remove existing hint labels
    for entity in existing_hints.iter() {
        commands.entity(entity).despawn();
    }

    if !state.active {
        return;
    }

    // Spawn new hint labels
    for hint in &state.hints {
        let mut matched = false;
        let mut partial_match = true;

        if !state.current_input.is_empty() {
            if hint.hint_key.starts_with(&state.current_input) {
                matched = hint.hint_key == state.current_input;
                partial_match = true;
            } else {
                partial_match = false;
            }
        }

        if !partial_match {
            continue;
        }

        let color = if matched {
            Color::srgb(0.0, 1.0, 0.0) // Green when matched
        } else {
            Color::srgb(1.0, 1.0, 0.0) // Yellow for hints
        };

        commands.spawn((
            HintLabel {
                hint_key: hint.hint_key.clone(),
            },
            Text2d::new(&hint.hint_key),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(color),
            Transform::from_translation(hint.position.extend(100.0)),
        ));
    }
}

/// Handle keyboard input for hint selection
fn handle_hint_input_system(
    mut state: ResMut<LinkHintsState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<LinkActivatedEvent>,
) {
    if !state.active {
        return;
    }

    // Handle escape to cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        state.active = false;
        state.hints.clear();
        state.current_input.clear();
        return;
    }

    // Handle letter input
    for key_code in keyboard.get_just_pressed() {
        if let Some(char) = keycode_to_char(*key_code) {
            state.current_input.push(char);

            // Check if we have a complete match
            if let Some(hint) = state
                .hints
                .iter()
                .find(|h| h.hint_key == state.current_input)
            {
                event_writer.send(LinkActivatedEvent { link: hint.clone() });
                state.active = false;
                state.hints.clear();
                state.current_input.clear();
            }
        }
    }
}

/// Activate selected link
fn activate_link_system(mut events: EventReader<LinkActivatedEvent>) {
    for event in events.read() {
        match event.link.link_type {
            LinkType::Url => {
                info!("Opening URL: {}", event.link.url);
                if let Err(e) = open_url(&event.link.url) {
                    error!("Failed to open URL: {}", e);
                }
            }
            LinkType::FilePath => {
                info!("Opening file: {}", event.link.url);
                if let Err(e) = open_file(&event.link.url) {
                    error!("Failed to open file: {}", e);
                }
            }
            LinkType::Email => {
                info!("Opening email: {}", event.link.url);
                if let Err(e) = open_email(&event.link.url) {
                    error!("Failed to open email client: {}", e);
                }
            }
        }
    }
}

/// Open a URL in the default browser using platform-specific commands
fn open_url(url: &str) -> Result<(), String> {
    info!("Attempting to open URL in browser: {}", url);

    // Ensure URL has protocol
    let full_url = if url.starts_with("www.") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&full_url)
        .spawn()
        .map_err(|e| format!("Failed to launch xdg-open: {}. Make sure xdg-utils is installed.", e));

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&full_url)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &full_url])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening URLs".to_string());

    result.map(|_| {
        info!("Successfully launched browser for URL: {}", full_url);
    })
}

/// Open a file in the default editor or application
fn open_file(path: &str) -> Result<(), String> {
    info!("Attempting to open file: {}", path);

    // Expand ~ to home directory
    let expanded_path = if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            path.replacen('~', &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };

    // Check if file exists
    if !std::path::Path::new(&expanded_path).exists() {
        return Err(format!("File does not exist: {}", expanded_path));
    }

    // Try $EDITOR first for text files, then fall back to system default
    if let Ok(editor) = std::env::var("EDITOR") {
        info!("Opening file with $EDITOR ({}): {}", editor, expanded_path);

        let result = std::process::Command::new(&editor)
            .arg(&expanded_path)
            .spawn()
            .map_err(|e| format!("Failed to launch editor '{}': {}", editor, e));

        if result.is_ok() {
            return result.map(|_| ());
        } else {
            warn!("Failed to open with $EDITOR, falling back to system default");
        }
    }

    // Fall back to platform-specific open commands
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&expanded_path)
        .spawn()
        .map_err(|e| format!("Failed to launch xdg-open: {}. Make sure xdg-utils is installed.", e));

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&expanded_path)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &expanded_path])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening files".to_string());

    result.map(|_| {
        info!("Successfully opened file: {}", expanded_path);
    })
}

/// Open email client with mailto: link
fn open_email(email: &str) -> Result<(), String> {
    info!("Attempting to open email client for: {}", email);

    // Construct mailto: URL
    let mailto_url = if email.starts_with("mailto:") {
        email.to_string()
    } else {
        format!("mailto:{}", email)
    };

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&mailto_url)
        .spawn()
        .map_err(|e| format!("Failed to launch xdg-open: {}. Make sure xdg-utils is installed.", e));

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&mailto_url)
        .spawn()
        .map_err(|e| format!("Failed to launch open command: {}", e));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", "", &mailto_url])
        .spawn()
        .map_err(|e| format!("Failed to launch cmd: {}", e));

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let result = Err("Unsupported platform for opening email client".to_string());

    result.map(|_| {
        info!("Successfully opened email client for: {}", mailto_url);
    })
}

/// Convert KeyCode to character (a-z)
fn keycode_to_char(keycode: KeyCode) -> Option<char> {
    match keycode {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        let detector = LinkDetector::default();
        let text = "Visit https://example.com or www.github.com";
        let links = detector.detect(text);

        // Note: The filepath regex also matches parts of URLs containing dots
        // so we verify that URLs are detected correctly
        let url_links: Vec<_> = links.iter().filter(|(_, t)| *t == LinkType::Url).collect();
        assert_eq!(url_links.len(), 2);
        assert_eq!(url_links[0].0, "https://example.com");
        assert_eq!(url_links[1].0, "www.github.com");
    }

    #[test]
    fn test_filepath_detection() {
        let detector = LinkDetector::default();
        let text = "Check /usr/local/bin or ./relative/path.txt";
        let links = detector.detect(text);

        assert!(links.iter().any(|(l, _)| l.contains("/usr/local/bin")));
        assert!(links.iter().any(|(l, _)| l.contains("./relative/path.txt")));
    }

    #[test]
    fn test_hint_key_generation() {
        let keys = LinkDetector::generate_hint_keys(30);

        assert_eq!(keys[0], "a");
        assert_eq!(keys[25], "z");
        assert_eq!(keys[26], "aa");
        assert_eq!(keys[27], "ab");
    }

    #[test]
    fn test_detect_with_positions() {
        let detector = LinkDetector::default();
        let text = "Line 1: https://example.com\nLine 2: test@email.com";
        let links = detector.detect_with_positions(text);

        // Should find URL on line 0 (row 0)
        let url_link = links.iter().find(|(url, _, _, _)| url.contains("example.com"));
        assert!(url_link.is_some());
        let (_, _, col, row) = url_link.unwrap();
        assert_eq!(*row, 0);
        assert_eq!(*col, 8); // Position after "Line 1: "

        // Should find email on line 1 (row 1)
        let email_link = links.iter().find(|(url, _, _, _)| url.contains("test@email.com"));
        assert!(email_link.is_some());
        let (_, _, col, row) = email_link.unwrap();
        assert_eq!(*row, 1);
        assert_eq!(*col, 8); // Position after "Line 2: "
    }
}
