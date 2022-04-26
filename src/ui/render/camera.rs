use crate::{
    ecs::{Commands, Res},
    render::{cameras::ActiveCamera, render_phase::RenderPhase},
    ui::entity::CameraUi,
};

use super::TransparentUi;

/// Inserts the [`RenderPhase`] into the UI camera
pub fn extract_ui_camera_phases(
    mut commands: Commands,
    active_camera: Res<ActiveCamera<CameraUi>>,
) {
    if let Some(entity) = active_camera.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<TransparentUi>::default());
    }
}
