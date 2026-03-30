// Smooth animations for UI elements
// Provides fade in/out and other transitions

use bevy::prelude::*;

/// Plugin for UI animations
pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_fade_animations_system,
                update_slide_animations_system,
                cleanup_finished_animations_system,
            ),
        );
    }
}

/// Animation state for UI elements
#[derive(Component, Clone, Debug)]
pub enum AnimationState {
    FadeIn {
        duration: f32,
        elapsed: f32,
    },
    FadeOut {
        duration: f32,
        elapsed: f32,
    },
    SlideIn {
        direction: SlideDirection,
        duration: f32,
        elapsed: f32,
        start_pos: Vec2,
        end_pos: Vec2,
    },
    SlideOut {
        direction: SlideDirection,
        duration: f32,
        elapsed: f32,
        start_pos: Vec2,
        end_pos: Vec2,
    },
}

/// Direction for slide animations
#[derive(Clone, Debug, Copy)]
pub enum SlideDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Fade animation component
#[derive(Component, Clone, Debug)]
pub struct FadeAnimation {
    pub duration: f32,
    pub fade_in: bool,
    pub elapsed: f32,
}

impl FadeAnimation {
    pub fn fade_in(duration: f32) -> Self {
        Self {
            duration,
            fade_in: true,
            elapsed: 0.0,
        }
    }

    pub fn fade_out(duration: f32) -> Self {
        Self {
            duration,
            fade_in: false,
            elapsed: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn alpha(&self) -> f32 {
        let progress = self.progress();
        if self.fade_in {
            easing::ease_out_cubic(progress)
        } else {
            1.0 - easing::ease_in_cubic(progress)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Slide animation component
#[derive(Component, Clone, Debug)]
pub struct SlideAnimation {
    pub direction: SlideDirection,
    pub distance: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub initial_position: Vec2,
}

impl SlideAnimation {
    pub fn new(direction: SlideDirection, distance: f32, duration: f32) -> Self {
        Self {
            direction,
            distance,
            duration,
            elapsed: 0.0,
            initial_position: Vec2::ZERO,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn current_offset(&self) -> Vec2 {
        let progress = easing::ease_out_cubic(self.progress());
        let offset = self.distance * progress;

        match self.direction {
            SlideDirection::Left => Vec2::new(-offset, 0.0),
            SlideDirection::Right => Vec2::new(offset, 0.0),
            SlideDirection::Up => Vec2::new(0.0, offset),
            SlideDirection::Down => Vec2::new(0.0, -offset),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Marker for animation completion
#[derive(Component)]
pub struct AnimationComplete;

/// Update fade animations
fn update_fade_animations_system(
    time: Res<Time>,
    mut query: Query<(
        &mut FadeAnimation,
        Option<&mut BackgroundColor>,
        Option<&mut Text>,
    )>,
) {
    for (mut anim, bg_color, text) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        let alpha = anim.alpha();

        // Update background color alpha
        if let Some(mut bg) = bg_color {
            let mut color = bg.0;
            color.set_alpha(alpha);
            bg.0 = color;
        }

        // Update text color alpha (Bevy 0.15 doesn't have sections, text is simpler)
        // If more complex text styling is needed, this will need adjustment
        if let Some(mut _text) = text {
            // Note: In Bevy 0.15, Text doesn't have sections in the same way
            // This would need to be updated based on actual Bevy 0.15 text API
            // For now, leaving this as a placeholder
        }
    }
}

/// Update slide animations
fn update_slide_animations_system(
    time: Res<Time>,
    mut query: Query<(&mut SlideAnimation, &mut Node)>,
) {
    for (mut anim, mut node) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        let offset = anim.current_offset();
        let final_pos = anim.initial_position + offset;

        node.left = Val::Px(final_pos.x);
        node.top = Val::Px(final_pos.y);
    }
}

/// Clean up finished animations
fn cleanup_finished_animations_system(
    mut commands: Commands,
    fade_query: Query<(Entity, &FadeAnimation), Without<AnimationComplete>>,
    slide_query: Query<(Entity, &SlideAnimation), Without<AnimationComplete>>,
) {
    for (entity, anim) in fade_query.iter() {
        if anim.is_complete() {
            commands.entity(entity).insert(AnimationComplete);
        }
    }

    for (entity, anim) in slide_query.iter() {
        if anim.is_complete() {
            commands.entity(entity).insert(AnimationComplete);
        }
    }
}

/// Easing functions for smooth animations
pub mod easing {
    pub fn ease_in_cubic(t: f32) -> f32 {
        t * t * t
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        let t = t - 1.0;
        t * t * t + 1.0
    }

    pub fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let t = 2.0 * t - 2.0;
            1.0 + t * t * t / 2.0
        }
    }

    pub fn ease_in_quad(t: f32) -> f32 {
        t * t
    }

    pub fn ease_out_quad(t: f32) -> f32 {
        t * (2.0 - t)
    }

    pub fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }

    pub fn ease_in_sine(t: f32) -> f32 {
        1.0 - f32::cos(t * std::f32::consts::PI / 2.0)
    }

    pub fn ease_out_sine(t: f32) -> f32 {
        f32::sin(t * std::f32::consts::PI / 2.0)
    }

    pub fn ease_in_out_sine(t: f32) -> f32 {
        -(f32::cos(std::f32::consts::PI * t) - 1.0) / 2.0
    }
}

/// Bundle for animated UI elements
#[derive(Bundle)]
pub struct AnimatedUIBundle {
    pub node: Node,
    pub background_color: BackgroundColor,
    pub fade: FadeAnimation,
}

impl AnimatedUIBundle {
    pub fn fade_in(duration: f32) -> Self {
        Self {
            node: Node::default(),
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            fade: FadeAnimation::fade_in(duration),
        }
    }

    pub fn fade_out(duration: f32) -> Self {
        Self {
            node: Node::default(),
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            fade: FadeAnimation::fade_out(duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fade_animation_progress() {
        let mut anim = FadeAnimation::fade_in(1.0);
        assert_eq!(anim.progress(), 0.0);

        anim.elapsed = 0.5;
        assert_eq!(anim.progress(), 0.5);

        anim.elapsed = 1.0;
        assert_eq!(anim.progress(), 1.0);

        anim.elapsed = 2.0;
        assert_eq!(anim.progress(), 1.0); // Clamped
    }

    #[test]
    fn test_fade_in_alpha() {
        let mut anim = FadeAnimation::fade_in(1.0);

        anim.elapsed = 0.0;
        assert_eq!(anim.alpha(), 0.0);

        anim.elapsed = 1.0;
        assert_eq!(anim.alpha(), 1.0);
    }

    #[test]
    fn test_fade_out_alpha() {
        let mut anim = FadeAnimation::fade_out(1.0);

        anim.elapsed = 0.0;
        assert_eq!(anim.alpha(), 1.0);

        anim.elapsed = 1.0;
        assert_eq!(anim.alpha(), 0.0);
    }

    #[test]
    fn test_easing_functions() {
        use super::easing::*;

        assert_eq!(ease_in_cubic(0.0), 0.0);
        assert_eq!(ease_in_cubic(1.0), 1.0);

        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
    }
}
