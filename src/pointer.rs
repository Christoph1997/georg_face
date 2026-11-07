use bevy::{input::touch::Touches, prelude::*};

/// Mouse and touch positions in physical pixels (the unit UI nodes use).
///
/// Bevy's `Interaction` prefers the OS mouse cursor position over touch
/// positions, so on a touchscreen (where the cursor sits elsewhere or is
/// stale) taps hit the wrong spot. This gives every pointer its own position.
pub struct Pointers {
    /// Pointers that went down this frame.
    pub pressed: Vec<Vec2>,
    /// Pointers currently held down.
    pub held: Vec<Vec2>,
    /// Mouse cursor, if it is inside the window (for hover effects).
    pub cursor: Option<Vec2>,
}

impl Pointers {
    pub fn gather(window: &Window, mouse: &ButtonInput<MouseButton>, touches: &Touches) -> Self {
        // Touch positions are logical pixels, UI nodes use physical pixels.
        let scale = window.scale_factor();
        let cursor = window.physical_cursor_position();

        let mut pressed: Vec<Vec2> = touches
            .iter_just_pressed()
            .map(|t| t.position() * scale)
            .collect();
        let mut held: Vec<Vec2> = touches.iter().map(|t| t.position() * scale).collect();

        if let Some(p) = cursor {
            if mouse.just_pressed(MouseButton::Left) {
                pressed.push(p);
            }
            if mouse.pressed(MouseButton::Left) {
                held.push(p);
            }
        }

        Self { pressed, held, cursor }
    }

    /// A pointer went down on this node this frame.
    pub fn pressed_on(&self, node: &ComputedNode, transform: &UiGlobalTransform) -> bool {
        self.pressed.iter().any(|p| node.contains_point(*transform, *p))
    }

    /// A pointer is currently held down on this node.
    pub fn held_on(&self, node: &ComputedNode, transform: &UiGlobalTransform) -> bool {
        self.held.iter().any(|p| node.contains_point(*transform, *p))
    }

    /// The mouse cursor is over this node.
    pub fn hovering(&self, node: &ComputedNode, transform: &UiGlobalTransform) -> bool {
        self.cursor.is_some_and(|p| node.contains_point(*transform, p))
    }
}
