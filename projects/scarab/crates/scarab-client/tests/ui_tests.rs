// Comprehensive tests for UI components
// Tests link hints, command palette, leader key, keybindings, animations, and visual selection

#[cfg(test)]
mod link_hints_tests {
    use scarab_client::ui::link_hints::LinkType;
    use scarab_client::ui::LinkDetector;

    #[test]
    fn test_url_detection_http() {
        let detector = LinkDetector::default();
        let text = "Check out https://example.com for more info";
        let links = detector.detect(text);

        assert!(links.len() >= 1);
        // Find the http link
        let http_link = links.iter().find(|(url, _)| url == "https://example.com");
        assert!(http_link.is_some());
        assert_eq!(http_link.unwrap().1, LinkType::Url);
    }

    #[test]
    fn test_url_detection_www() {
        let detector = LinkDetector::default();
        let text = "Visit www.github.com/user/repo";
        let links = detector.detect(text);

        assert!(links.iter().any(|(url, _)| url.contains("www.github.com")));
    }

    #[test]
    fn test_multiple_urls() {
        let detector = LinkDetector::default();
        let text = "Check https://example.com and www.test.org";
        let links = detector.detect(text);

        assert!(links.len() >= 2);
    }

    #[test]
    fn test_filepath_detection() {
        let detector = LinkDetector::default();
        let text = "Edit /usr/local/bin/script.sh or ./relative/path.txt";
        let links = detector.detect(text);

        assert!(links
            .iter()
            .any(|(path, _)| path.contains("/usr/local/bin")));
        assert!(links
            .iter()
            .any(|(path, _)| path.contains("./relative/path.txt")));
    }

    #[test]
    fn test_email_detection() {
        let detector = LinkDetector::default();
        let text = "Contact user@example.com for support";
        let links = detector.detect(text);

        assert!(links
            .iter()
            .any(|(email, t)| { email == "user@example.com" && *t == LinkType::Email }));
    }

    #[test]
    fn test_hint_key_generation() {
        let keys = LinkDetector::generate_hint_keys(30);

        assert_eq!(keys.len(), 30);
        assert_eq!(keys[0], "a");
        assert_eq!(keys[25], "z");
        assert_eq!(keys[26], "aa");
        assert_eq!(keys[27], "ab");

        // Ensure all keys are unique
        let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique_keys.len(), 30);
    }

    #[test]
    fn test_hint_key_generation_large() {
        let keys = LinkDetector::generate_hint_keys(100);
        assert_eq!(keys.len(), 100);

        // All keys should be unique
        let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique_keys.len(), 100);
    }

    #[test]
    fn test_link_detection_accuracy() {
        let detector = LinkDetector::default();

        // Test cases with expected link counts
        let test_cases = vec![
            ("No links here", 0),
            ("https://example.com", 1),
            ("Visit https://a.com and https://b.com", 2),
            ("Email user@test.com", 1),
            ("/path/to/file.txt", 1),
        ];

        for (text, expected_count) in test_cases {
            let links = detector.detect(text);
            assert!(
                links.len() >= expected_count,
                "Text '{}' should have at least {} links, got {}",
                text,
                expected_count,
                links.len()
            );
        }
    }
}

#[cfg(test)]
mod command_palette_tests {
    use scarab_client::ui::{Command, CommandRegistry};
    use std::sync::Arc;

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new(
            "test",
            "Test Command",
            "A test command",
            "Test",
            |_| {},
        ));

        assert!(registry.get("test").is_some());
        assert_eq!(registry.get("test").unwrap().name, "Test Command");
    }

    #[test]
    fn test_fuzzy_search_exact_match() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new(
            "copy",
            "Copy Selection",
            "Copy text to clipboard",
            "Edit",
            |_| {},
        ));

        let results = registry.fuzzy_search("Copy");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "copy");
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new(
            "copy",
            "Copy Selection",
            "Copy text",
            "Edit",
            |_| {},
        ));

        let results = registry.fuzzy_search("cop");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "copy");
    }

    #[test]
    fn test_fuzzy_search_ranking() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new("a", "Copy", "Copy text", "Edit", |_| {}));
        registry.register(Command::new("b", "Paste", "Paste text", "Edit", |_| {}));
        registry.register(Command::new("c", "Cut", "Cut text", "Edit", |_| {}));

        let results = registry.fuzzy_search("copy");
        assert!(results.len() > 0);
        assert_eq!(results[0].0.id, "a"); // "Copy" should rank highest
    }

    #[test]
    fn test_fuzzy_search_performance() {
        let mut registry = CommandRegistry::default();

        // Register 1000 commands
        for i in 0..1000 {
            registry.register(Command::new(
                &format!("cmd_{}", i),
                &format!("Command {}", i),
                &format!("Description {}", i),
                "Test",
                |_| {},
            ));
        }

        // Search should complete quickly
        use std::time::Instant;
        let start = Instant::now();
        let _results = registry.fuzzy_search("command");
        let duration = start.elapsed();

        // Should complete in less than 50ms
        assert!(
            duration.as_millis() < 50,
            "Fuzzy search took {}ms, expected <50ms",
            duration.as_millis()
        );
    }

    #[test]
    fn test_empty_search() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new("a", "Test A", "Test", "Cat", |_| {}));
        registry.register(Command::new("b", "Test B", "Test", "Cat", |_| {}));

        // Empty search should return all commands
        let results = registry.fuzzy_search("");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_no_matches() {
        let mut registry = CommandRegistry::default();

        registry.register(Command::new("a", "Test", "Test", "Cat", |_| {}));

        let results = registry.fuzzy_search("xyz123nonexistent");
        assert_eq!(results.len(), 0);
    }
}

#[cfg(test)]
mod keybindings_tests {
    use bevy::input::keyboard::KeyCode;
    use scarab_client::ui::{KeyBinding, KeyBindingConfig};

    #[test]
    fn test_keybinding_creation() {
        let binding = KeyBinding::new(KeyCode::KeyC).with_ctrl();

        assert_eq!(binding.key, KeyCode::KeyC);
        assert!(binding.ctrl);
        assert!(!binding.alt);
        assert!(!binding.shift);
    }

    #[test]
    fn test_keybinding_string_conversion() {
        let binding = KeyBinding::new(KeyCode::KeyC).with_ctrl().with_shift();

        let string = binding.to_string();
        assert_eq!(string, "Ctrl+Shift+KeyC");

        let parsed = KeyBinding::from_string(&string).unwrap();
        assert_eq!(parsed, binding);
    }

    #[test]
    fn test_keybinding_config() {
        let mut config = KeyBindingConfig::default();

        let binding = KeyBinding::new(KeyCode::KeyS).with_ctrl();
        config.bind(binding.clone(), "file.save");

        assert_eq!(config.get_action(&binding), Some("file.save"));
    }

    #[test]
    fn test_find_binding_by_action() {
        let mut config = KeyBindingConfig::default();

        let binding = KeyBinding::new(KeyCode::KeyO).with_ctrl();
        config.bind(binding.clone(), "file.open");

        let found = config.find_binding("file.open");
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &binding);
    }

    #[test]
    fn test_unbind() {
        let mut config = KeyBindingConfig::default();

        let binding = KeyBinding::new(KeyCode::KeyX).with_ctrl();
        config.bind(binding.clone(), "edit.cut");

        assert!(config.get_action(&binding).is_some());

        config.unbind(&binding);
        assert!(config.get_action(&binding).is_none());
    }

    #[test]
    fn test_default_bindings() {
        let config = KeyBindingConfig::default();

        // Should have some default bindings
        assert!(config.all_bindings().len() > 0);

        // Check for common bindings
        let ctrl_c = KeyBinding::new(KeyCode::KeyC).with_ctrl();
        assert!(config.get_action(&ctrl_c).is_some());
    }
}

#[cfg(test)]
mod animation_tests {
    use scarab_client::ui::FadeAnimation;

    #[test]
    fn test_fade_in_animation() {
        let mut anim = FadeAnimation::fade_in(1.0);

        assert_eq!(anim.progress(), 0.0);
        assert_eq!(anim.alpha(), 0.0);

        anim.elapsed = 0.5;
        assert!(anim.alpha() > 0.0 && anim.alpha() < 1.0);

        anim.elapsed = 1.0;
        assert_eq!(anim.alpha(), 1.0);
        assert!(anim.is_complete());
    }

    #[test]
    fn test_fade_out_animation() {
        let mut anim = FadeAnimation::fade_out(1.0);

        assert_eq!(anim.alpha(), 1.0);

        anim.elapsed = 0.5;
        assert!(anim.alpha() > 0.0 && anim.alpha() < 1.0);

        anim.elapsed = 1.0;
        assert_eq!(anim.alpha(), 0.0);
        assert!(anim.is_complete());
    }

    #[test]
    fn test_animation_clamping() {
        let mut anim = FadeAnimation::fade_in(1.0);

        anim.elapsed = 2.0; // Exceed duration
        assert_eq!(anim.progress(), 1.0); // Should be clamped
    }

    #[test]
    fn test_easing_functions() {
        use scarab_client::ui::animations::easing::*;

        // Test boundary conditions
        assert_eq!(ease_in_cubic(0.0), 0.0);
        assert_eq!(ease_in_cubic(1.0), 1.0);

        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);

        assert_eq!(ease_in_out_cubic(0.0), 0.0);
        assert_eq!(ease_in_out_cubic(1.0), 1.0);

        // Test midpoint behavior
        let mid_in = ease_in_cubic(0.5);
        let mid_out = ease_out_cubic(0.5);
        assert!(mid_in < 0.5); // Ease in should be slower at start
        assert!(mid_out > 0.5); // Ease out should be faster at start
    }

    #[test]
    fn test_60fps_animation_smoothness() {
        let mut anim = FadeAnimation::fade_in(1.0);
        let frame_time = 1.0 / 60.0; // 16.67ms per frame

        let mut last_alpha = 0.0;
        for _ in 0..60 {
            anim.elapsed += frame_time;
            let alpha = anim.alpha();

            // Alpha should increase monotonically
            assert!(alpha >= last_alpha);

            // Change should be smooth (not too large)
            let delta = alpha - last_alpha;
            assert!(delta < 0.5, "Alpha change too large: {}", delta);

            last_alpha = alpha;
        }

        // Ensure completion
        anim.elapsed = 1.0;
        assert!(anim.is_complete());
    }
}

#[cfg(test)]
mod visual_selection_tests {
    use scarab_client::ui::visual_selection::SelectionState;
    use scarab_client::ui::{SelectionMode, SelectionRegion};

    #[test]
    fn test_selection_region_contains() {
        let region = SelectionRegion::new(5, 5, 10, 10);

        assert!(region.contains(5, 5));
        assert!(region.contains(10, 10));
        assert!(region.contains(7, 7));
        assert!(!region.contains(4, 5));
        assert!(!region.contains(11, 10));
    }

    #[test]
    fn test_selection_region_normalize() {
        let mut region = SelectionRegion::new(10, 10, 5, 5);
        region.normalize();

        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 5);
        assert_eq!(region.end_x, 10);
        assert_eq!(region.end_y, 10);
    }

    #[test]
    fn test_selection_region_is_empty() {
        let region1 = SelectionRegion::new(5, 5, 5, 5);
        assert!(region1.is_empty());

        let region2 = SelectionRegion::new(5, 5, 10, 10);
        assert!(!region2.is_empty());
    }

    #[test]
    fn test_selection_state_lifecycle() {
        let mut state = SelectionState::default();

        assert!(!state.active);

        state.start_selection(5, 5, SelectionMode::Character);
        assert!(state.active);
        assert_eq!(state.region.start_x, 5);
        assert_eq!(state.region.start_y, 5);

        state.update_selection(10, 10);
        assert_eq!(state.region.end_x, 10);
        assert_eq!(state.region.end_y, 10);

        state.end_selection();
        assert!(!state.active);
    }

    #[test]
    fn test_selection_modes() {
        let mut state = SelectionState::default();

        state.start_selection(0, 0, SelectionMode::Character);
        assert_eq!(state.mode, SelectionMode::Character);

        state.start_selection(0, 0, SelectionMode::Line);
        assert_eq!(state.mode, SelectionMode::Line);

        state.start_selection(0, 0, SelectionMode::Block);
        assert_eq!(state.mode, SelectionMode::Block);
    }

    #[test]
    fn test_selection_clear() {
        let mut state = SelectionState::default();

        state.start_selection(5, 5, SelectionMode::Character);
        state.update_selection(10, 10);

        state.clear();

        assert!(!state.active);
        assert!(state.region.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    // Integration tests would go here
    // These would test the interaction between different UI components
    // For example: leader key triggering command palette, etc.
}
