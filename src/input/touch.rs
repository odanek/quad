use std::collections::HashMap;

use crate::{
    ecs::{Event, Resource},
    ty::Vec2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchId(u64);

impl TouchId {
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct TouchInput {
    pub phase: TouchPhase,
    pub position: Vec2,
    pub force: Option<ForceTouch>,
    pub id: TouchId,
}

impl TouchInput {
    pub(crate) fn from_winit_event(
        touch_input: winit::event::Touch,
        location: winit::dpi::LogicalPosition<f32>,
    ) -> TouchInput {
        TouchInput {
            phase: match touch_input.phase {
                winit::event::TouchPhase::Started => TouchPhase::Started,
                winit::event::TouchPhase::Moved => TouchPhase::Moved,
                winit::event::TouchPhase::Ended => TouchPhase::Ended,
                winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
            },
            position: Vec2::new(location.x as f32, location.y as f32),
            force: touch_input.force.map(|f| match f {
                winit::event::Force::Calibrated {
                    force,
                    max_possible_force,
                    altitude_angle,
                } => ForceTouch::Calibrated {
                    force,
                    max_possible_force,
                    altitude_angle,
                },
                winit::event::Force::Normalized(x) => ForceTouch::Normalized(x),
            }),
            id: TouchId(touch_input.id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForceTouch {
    Calibrated {
        force: f64,
        max_possible_force: f64,
        altitude_angle: Option<f64>,
    },
    Normalized(f64),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    id: TouchId,
    start_position: Vec2,
    start_force: Option<ForceTouch>,
    previous_position: Vec2,
    previous_force: Option<ForceTouch>,
    position: Vec2,
    force: Option<ForceTouch>,
}

impl Touch {
    pub fn delta(&self) -> Vec2 {
        self.position - self.previous_position
    }

    pub fn distance(&self) -> Vec2 {
        self.position - self.start_position
    }

    #[inline]
    pub fn id(&self) -> TouchId {
        self.id
    }

    #[inline]
    pub fn start_position(&self) -> Vec2 {
        self.start_position
    }

    #[inline]
    pub fn start_force(&self) -> Option<ForceTouch> {
        self.start_force
    }

    #[inline]
    pub fn previous_position(&self) -> Vec2 {
        self.previous_position
    }

    #[inline]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    #[inline]
    pub fn force(&self) -> Option<ForceTouch> {
        self.force
    }
}

impl From<&TouchInput> for Touch {
    fn from(input: &TouchInput) -> Touch {
        Touch {
            id: input.id,
            start_position: input.position,
            start_force: input.force,
            previous_position: input.position,
            previous_force: input.force,
            position: input.position,
            force: input.force,
        }
    }
}

#[derive(Debug, Clone, Default, Resource)]
pub struct Touches {
    pressed: HashMap<TouchId, Touch>,
    just_pressed: HashMap<TouchId, Touch>,
    just_released: HashMap<TouchId, Touch>,
    just_cancelled: HashMap<TouchId, Touch>,
}

impl Touches {
    pub fn iter(&self) -> impl Iterator<Item = &Touch> + '_ {
        self.pressed.values()
    }

    pub fn get_pressed(&self, id: TouchId) -> Option<&Touch> {
        self.pressed.get(&id)
    }

    pub fn just_pressed(&self, id: TouchId) -> bool {
        self.just_pressed.contains_key(&id)
    }

    pub fn iter_just_pressed(&self) -> impl Iterator<Item = &Touch> {
        self.just_pressed.values()
    }

    pub fn get_released(&self, id: TouchId) -> Option<&Touch> {
        self.just_released.get(&id)
    }

    pub fn just_released(&self, id: TouchId) -> bool {
        self.just_released.contains_key(&id)
    }

    pub fn iter_just_released(&self) -> impl Iterator<Item = &Touch> {
        self.just_released.values()
    }

    pub fn just_cancelled(&self, id: TouchId) -> bool {
        self.just_cancelled.contains_key(&id)
    }

    pub fn iter_just_cancelled(&self) -> impl Iterator<Item = &Touch> {
        self.just_cancelled.values()
    }

    pub(crate) fn process_event(&mut self, event: &TouchInput) {
        match event.phase {
            TouchPhase::Started => {
                self.pressed.insert(event.id, event.into());
                self.just_pressed.insert(event.id, event.into());
            }
            TouchPhase::Moved => {
                if let Some(mut new_touch) = self.pressed.get(&event.id).cloned() {
                    new_touch.previous_position = new_touch.position;
                    new_touch.previous_force = new_touch.force;
                    new_touch.position = event.position;
                    new_touch.force = event.force;
                    self.pressed.insert(event.id, new_touch);
                }
            }
            TouchPhase::Ended => {
                self.just_released.insert(event.id, event.into());
                self.pressed.remove_entry(&event.id);
            }
            TouchPhase::Cancelled => {
                self.just_cancelled.insert(event.id, event.into());
                self.pressed.remove_entry(&event.id);
            }
        };
    }

    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.just_cancelled.clear();
    }
}
