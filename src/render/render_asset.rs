use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

use crate::{
    app::{App, RenderStage},
    asset::{Asset, AssetEvent, Assets, Handle},
    ecs::{
        Commands, EventReader, IntoSystem, Res, ResMut, Resource, StaticSystemParam, SystemParam,
        SystemParamItem,
    },
};

pub enum PrepareAssetError<E: Send + Sync + 'static> {
    RetryNextUpdate(E),
}

/// Describes how an asset gets extracted and prepared for rendering.
///
/// In the [`RenderStage::Extract`](crate::RenderStage::Extract) step the asset is transferred
/// from the "app world" into the "render world".
/// Therefore it is converted into a [`RenderAsset::ExtractedAsset`], which may be the same type
/// as the render asset itself.
///
/// After that in the [`RenderStage::Prepare`](crate::RenderStage::Prepare) step the extracted asset
/// is transformed into its GPU-representation of type [`RenderAsset::PreparedAsset`].
pub trait RenderAsset: Asset {
    /// The representation of the the asset in the "render world".
    type ExtractedAsset: Send + Sync + 'static;
    /// The GPU-representation of the the asset.
    type PreparedAsset: Send + Sync + 'static;
    /// Specifies all ECS data required by [`RenderAsset::prepare_asset`].
    /// For convenience use the [`lifetimeless`](bevy_ecs::system::lifetimeless) [`SystemParam`].
    type Param: SystemParam;
    /// Converts the asset into a [`RenderAsset::ExtractedAsset`].
    fn extract_asset(&self) -> Self::ExtractedAsset;
    /// Prepares the `extracted asset` for the GPU by transforming it into
    /// a [`RenderAsset::PreparedAsset`]. Therefore ECS data may be accessed via the `param`.
    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>>;
}

pub fn render_asset_plugin<A: RenderAsset>(app: &mut App, render_app: &mut App) {
    render_app
        .init_resource::<ExtractedAssets<A>>()
        .init_resource::<RenderAssets<A>>()
        .init_resource::<PrepareNextFrameAssets<A>>()
        .add_system_to_stage(
            RenderStage::Extract,
            extract_render_asset::<A>.system(&mut app.world),
        )
        .add_system_to_stage(RenderStage::Prepare, prepare_assets::<A>);
}

/// Temporarily stores the extracted and removed assets of the current frame.
#[derive(Resource)]
pub struct ExtractedAssets<A: RenderAsset> {
    extracted: Vec<(Handle<A>, A::ExtractedAsset)>,
    removed: Vec<Handle<A>>,
}

impl<A: RenderAsset> Default for ExtractedAssets<A> {
    fn default() -> Self {
        Self {
            extracted: Default::default(),
            removed: Default::default(),
        }
    }
}

/// Stores all GPU representations ([`RenderAsset::PreparedAssets`](RenderAsset::PreparedAsset))
/// of [`RenderAssets`](RenderAsset) as long as they exist.
#[derive(Resource)]
pub struct RenderAssets<A: RenderAsset>(HashMap<Handle<A>, <A as RenderAsset>::PreparedAsset>);

impl<A: RenderAsset> Default for RenderAssets<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: RenderAsset> Deref for RenderAssets<A> {
    type Target = HashMap<Handle<A>, <A as RenderAsset>::PreparedAsset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A: RenderAsset> DerefMut for RenderAssets<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// This system extracts all crated or modified assets of the corresponding [`RenderAsset`] type
/// into the "render world".
fn extract_render_asset<A: RenderAsset>(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<A>>,
    assets: Res<Assets<A>>,
) {
    let mut changed_assets: HashSet<&Handle<A>> = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                changed_assets.insert(handle);
            }
            AssetEvent::Modified { handle } => {
                changed_assets.insert(handle);
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(handle) {
            extracted_assets.push((handle.clone_weak(), asset.extract_asset()));
        }
    }

    commands.insert_resource(ExtractedAssets {
        extracted: extracted_assets,
        removed,
    })
}

// TODO: consider storing inside system?
/// All assets that should be prepared next frame.
#[derive(Resource)]
pub struct PrepareNextFrameAssets<A: RenderAsset> {
    assets: Vec<(Handle<A>, A::ExtractedAsset)>,
}

impl<A: RenderAsset> Default for PrepareNextFrameAssets<A> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

fn prepare_assets<R: RenderAsset>(
    mut extracted_assets: ResMut<ExtractedAssets<R>>,
    mut render_assets: ResMut<RenderAssets<R>>,
    mut prepare_next_frame: ResMut<PrepareNextFrameAssets<R>>,
    param: StaticSystemParam<<R as RenderAsset>::Param>,
) {
    let mut param = param.into_inner();
    let mut queued_assets = std::mem::take(&mut prepare_next_frame.assets);
    for (handle, extracted_asset) in queued_assets.drain(..) {
        match R::prepare_asset(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                render_assets.insert(handle, prepared_asset);
            }
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                prepare_next_frame.assets.push((handle, extracted_asset));
            }
        }
    }

    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_assets.remove(&removed);
    }

    for (handle, extracted_asset) in std::mem::take(&mut extracted_assets.extracted) {
        match R::prepare_asset(extracted_asset, &mut param) {
            Ok(prepared_asset) => {
                render_assets.insert(handle, prepared_asset);
            }
            Err(PrepareAssetError::RetryNextUpdate(extracted_asset)) => {
                prepare_next_frame.assets.push((handle, extracted_asset));
            }
        }
    }
}
