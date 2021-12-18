mod button;
mod gamepad;
mod keyboard;
mod keycode;
mod mouse;
mod touch;

pub use gamepad::GamepadInput;
pub use keyboard::{KeyInput, KeyboardInput};
pub use keycode::KeyCode;
pub use mouse::*;
pub use touch::*;

use crate::{
    app::{App, Stage},
    ecs::ResMut,
};

pub fn input_plugin(app: &mut App) {
    app.init_resource::<KeyboardInput>()
        .init_resource::<MouseInput>()
        .init_resource::<Touches>();

    app.add_event::<KeyInput>()
        .add_event::<MouseButtonInput>()
        .add_event::<MouseWheel>()
        .add_event::<MouseMotion>();

    app.add_system_to_stage(Stage::Flush, &input_flush_system);
}

fn input_flush_system(
    mut keyboard: ResMut<KeyboardInput>,
    mut mouse: ResMut<MouseInput>,
    mut touches: ResMut<Touches>,
) {
    keyboard.flush();
    mouse.flush();
    touches.flush();
}
