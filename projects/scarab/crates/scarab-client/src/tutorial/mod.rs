//! Interactive Tutorial System for Scarab Terminal
//!
//! Provides a guided 8-step tour that launches on first run:
//! 1. Welcome - Introduce Scarab
//! 2. Navigation - Basic command usage
//! 3. Scrollback - Mouse wheel scrolling
//! 4. Link Hints - URL detection and opening
//! 5. Command Palette - Quick command access
//! 6. Plugins - Plugin system overview
//! 7. Configuration - Config file location
//! 8. Completion - Summary and next steps

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod steps;
mod ui;
mod validation;

pub use steps::TutorialSteps;
pub use ui::TutorialUI;

/// Tutorial system plugin for Bevy
pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TutorialEvent>()
            .insert_resource(TutorialSystem::new())
            .add_systems(Startup, check_first_launch)
            .add_systems(
                Update,
                (
                    update_tutorial_state,
                    render_tutorial_overlay,
                    handle_tutorial_input,
                )
                    .run_if(tutorial_active),
            );
    }
}

/// Tutorial system state
#[derive(Resource)]
pub struct TutorialSystem {
    pub current_step: usize,
    pub steps: Vec<TutorialStep>,
    pub state: TutorialState,
    pub progress_file: PathBuf,
}

/// Individual tutorial step
#[derive(Clone)]
pub struct TutorialStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instruction: String,
    pub validation: fn(&TerminalContext) -> bool,
    pub hint: Option<String>,
    pub visual_demo: Option<String>, // Path to demo GIF
}

/// Tutorial state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TutorialState {
    NotStarted,
    InProgress { step: usize },
    Completed,
    Skipped,
}

/// Tutorial events
#[derive(Event)]
pub enum TutorialEvent {
    Start,
    NextStep,
    PreviousStep,
    Skip,
    Complete,
    ValidateStep(usize),
}

/// Terminal context for validation
pub struct TerminalContext {
    pub last_command: Option<String>,
    pub scroll_position: i32,
    pub palette_opened: bool,
    pub link_hints_triggered: bool,
}

impl TutorialSystem {
    /// Create new tutorial system with all steps
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let progress_file = PathBuf::from(home)
            .join(".config/scarab/tutorial_progress.json");

        Self {
            current_step: 0,
            steps: TutorialSteps::create_all_steps(),
            state: TutorialState::NotStarted,
            progress_file,
        }
    }

    /// Check if this is the first launch
    pub fn is_first_launch(&self) -> bool {
        !self.progress_file.exists()
    }

    /// Mark tutorial as seen
    pub fn mark_tutorial_seen(&self) -> std::io::Result<()> {
        if let Some(parent) = self.progress_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let progress = TutorialProgress {
            state: self.state,
            completed_steps: (0..=self.current_step).collect(),
            last_step: self.current_step,
        };

        let json = serde_json::to_string_pretty(&progress)?;
        std::fs::write(&self.progress_file, json)?;
        Ok(())
    }

    /// Load tutorial progress
    pub fn load_progress(&mut self) -> std::io::Result<()> {
        if !self.progress_file.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(&self.progress_file)?;
        let progress: TutorialProgress = serde_json::from_str(&json)?;

        self.state = progress.state;
        if let TutorialState::InProgress { step } = progress.state {
            self.current_step = step;
        }

        Ok(())
    }

    /// Start the tutorial
    pub fn start(&mut self) {
        self.state = TutorialState::InProgress { step: 0 };
        self.current_step = 0;
    }

    /// Move to next step
    pub fn next_step(&mut self) -> bool {
        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
            self.state = TutorialState::InProgress {
                step: self.current_step,
            };
            let _ = self.mark_tutorial_seen();
            true
        } else {
            self.complete();
            false
        }
    }

    /// Move to previous step
    pub fn previous_step(&mut self) -> bool {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.state = TutorialState::InProgress {
                step: self.current_step,
            };
            let _ = self.mark_tutorial_seen();
            true
        } else {
            false
        }
    }

    /// Skip the tutorial
    pub fn skip(&mut self) {
        self.state = TutorialState::Skipped;
        let _ = self.mark_tutorial_seen();
    }

    /// Complete the tutorial
    pub fn complete(&mut self) {
        self.state = TutorialState::Completed;
        let _ = self.mark_tutorial_seen();
    }

    /// Check if current step is valid
    pub fn validate_current_step(&self, context: &TerminalContext) -> bool {
        if let Some(step) = self.steps.get(self.current_step) {
            (step.validation)(context)
        } else {
            false
        }
    }

    /// Get current step
    pub fn get_current_step(&self) -> Option<&TutorialStep> {
        self.steps.get(self.current_step)
    }

    /// Get progress percentage
    pub fn progress_percentage(&self) -> f32 {
        if self.steps.is_empty() {
            100.0
        } else {
            (self.current_step as f32 / self.steps.len() as f32) * 100.0
        }
    }
}

impl Default for TutorialSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Tutorial progress saved to disk
#[derive(Serialize, Deserialize)]
struct TutorialProgress {
    state: TutorialState,
    completed_steps: Vec<usize>,
    last_step: usize,
}

/// System to check if this is first launch
fn check_first_launch(mut tutorial: ResMut<TutorialSystem>, mut events: EventWriter<TutorialEvent>) {
    // Load existing progress
    let _ = tutorial.load_progress();

    // Check for --tutorial flag
    let args: Vec<String> = std::env::args().collect();
    let force_tutorial = args.contains(&"--tutorial".to_string());

    // Start tutorial if first launch or forced
    if force_tutorial || tutorial.is_first_launch() {
        println!("Starting interactive tutorial...");
        events.send(TutorialEvent::Start);
    }
}

/// System to update tutorial state based on events
fn update_tutorial_state(
    mut tutorial: ResMut<TutorialSystem>,
    mut events: EventReader<TutorialEvent>,
) {
    for event in events.read() {
        match event {
            TutorialEvent::Start => tutorial.start(),
            TutorialEvent::NextStep => {
                if !tutorial.next_step() {
                    println!("Tutorial completed!");
                }
            }
            TutorialEvent::PreviousStep => {
                tutorial.previous_step();
            }
            TutorialEvent::Skip => {
                tutorial.skip();
                println!("Tutorial skipped. You can replay it anytime with --tutorial flag");
            }
            TutorialEvent::Complete => {
                tutorial.complete();
                println!("Congratulations! You've completed the Scarab tutorial!");
            }
            TutorialEvent::ValidateStep(step) => {
                // Validation happens in UI rendering
                println!("Validating step {}", step);
            }
        }
    }
}

/// System to render tutorial overlay
fn render_tutorial_overlay(
    tutorial: Res<TutorialSystem>,
    // We'll use UI components from ui.rs module
) {
    if let Some(step) = tutorial.get_current_step() {
        // Tutorial UI rendering happens in ui.rs
        // This is just a marker system
        TutorialUI::render_step(&tutorial, step);
    }
}

/// System to handle tutorial input
fn handle_tutorial_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<TutorialEvent>,
) {
    // Space or Enter to advance
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        events.send(TutorialEvent::NextStep);
    }

    // Backspace to go back
    if keys.just_pressed(KeyCode::Backspace) {
        events.send(TutorialEvent::PreviousStep);
    }

    // Escape to skip
    if keys.just_pressed(KeyCode::Escape) {
        events.send(TutorialEvent::Skip);
    }
}

/// Condition to check if tutorial is active
fn tutorial_active(tutorial: Res<TutorialSystem>) -> bool {
    matches!(tutorial.state, TutorialState::InProgress { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_progression() {
        let mut tutorial = TutorialSystem::new();
        assert_eq!(tutorial.current_step, 0);

        tutorial.start();
        assert!(matches!(
            tutorial.state,
            TutorialState::InProgress { step: 0 }
        ));

        tutorial.next_step();
        assert_eq!(tutorial.current_step, 1);

        tutorial.previous_step();
        assert_eq!(tutorial.current_step, 0);
    }

    #[test]
    fn test_tutorial_skip() {
        let mut tutorial = TutorialSystem::new();
        tutorial.start();

        tutorial.skip();
        assert_eq!(tutorial.state, TutorialState::Skipped);
    }

    #[test]
    fn test_tutorial_completion() {
        let mut tutorial = TutorialSystem::new();
        tutorial.start();

        // Complete all steps
        while tutorial.next_step() {}

        assert_eq!(tutorial.state, TutorialState::Completed);
    }

    #[test]
    fn test_progress_percentage() {
        let mut tutorial = TutorialSystem::new();
        assert_eq!(tutorial.progress_percentage(), 0.0);

        tutorial.current_step = tutorial.steps.len() - 1;
        assert!(tutorial.progress_percentage() > 80.0);
    }
}
