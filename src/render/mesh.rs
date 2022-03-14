#[allow(clippy::module_inception)]
mod mesh;
pub mod shape;

pub use mesh::*;

use crate::app::App;

use super::render_asset::render_asset_plugin;

// TODO Does quad need meshes at all?
fn mesh_plugin(app: &mut App) {
    app.add_asset::<Mesh>();
    render_asset_plugin::<Mesh>(app);
}
