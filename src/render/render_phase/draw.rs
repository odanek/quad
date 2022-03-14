use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{any::TypeId, collections::HashMap, fmt::Debug, hash::Hash, ops::Range};

use crate::{
    ecs::{
        Entity, ReadOnlySystemParamFetch, Res, Resource, SystemParam, SystemParamItem, SystemState,
        World,
    },
    macros::all_tuples,
    render::render_resource::{CachedPipelineId, RenderPipelineCache},
};

use super::TrackedRenderPass;

/// A draw function which is used to draw a specific [`PhaseItem`].
///
/// They are the the general form of drawing items, whereas [`RenderCommands`](RenderCommand)
/// are more modular.
pub trait Draw<P: PhaseItem>: Send + Sync + 'static {
    /// Draws the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &P,
    );
}

/// An item which will be drawn to the screen. A phase item should be queued up for rendering
/// during the [`RenderStage::Queue`](crate::RenderStage::Queue) stage.
/// Afterwards it will be sorted and rendered automatically  in the
/// [`RenderStage::PhaseSort`](crate::RenderStage::PhaseSort) stage and
/// [`RenderStage::Render`](crate::RenderStage::Render) stage, respectively.
pub trait PhaseItem: Send + Sync + 'static {
    /// The type used for ordering the items. The smallest values are drawn first.
    type SortKey: Ord;
    /// Determines the order in which the items are drawn during the corresponding [`RenderPhase`](super::RenderPhase).
    fn sort_key(&self) -> Self::SortKey;
    /// Specifies the [`Draw`] function used to render the item.
    fn draw_function(&self) -> DrawFunctionId;
}

// TODO: make this generic?
/// /// A [`Draw`] function identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DrawFunctionId(usize);

/// Stores all draw functions for the [`PhaseItem`] type.
/// For retrieval they are associated with their [`TypeId`].
pub struct DrawFunctionsInternal<P: PhaseItem> {
    pub draw_functions: Vec<Box<dyn Draw<P>>>,
    pub indices: HashMap<TypeId, DrawFunctionId>,
}

impl<P: PhaseItem> DrawFunctionsInternal<P> {
    /// Adds the [`Draw`] function and associates it to its own type.
    pub fn add<T: Draw<P>>(&mut self, draw_function: T) -> DrawFunctionId {
        self.add_with::<T, T>(draw_function)
    }

    /// Adds the [`Draw`] function and associates it to the type `T`
    pub fn add_with<T: 'static, D: Draw<P>>(&mut self, draw_function: D) -> DrawFunctionId {
        self.draw_functions.push(Box::new(draw_function));
        let id = DrawFunctionId(self.draw_functions.len() - 1);
        self.indices.insert(TypeId::of::<T>(), id);
        id
    }

    /// Retrieves the [`Draw`] function corresponding to the `id` mutably.
    pub fn get_mut(&mut self, id: DrawFunctionId) -> Option<&mut dyn Draw<P>> {
        self.draw_functions.get_mut(id.0).map(|f| &mut **f)
    }

    /// Retrieves the id of the [`Draw`] function corresponding to their associated type `T`.
    pub fn get_id<T: 'static>(&self) -> Option<DrawFunctionId> {
        self.indices.get(&TypeId::of::<T>()).copied()
    }
}

/// Stores all draw functions for the [`PhaseItem`] type hidden behind a reader-writer lock.
/// To access them the [`DrawFunctions::read`] and [`DrawFunctions::write`] methods are used.
#[derive(Resource)]
pub struct DrawFunctions<P: PhaseItem> {
    internal: RwLock<DrawFunctionsInternal<P>>,
}

impl<P: PhaseItem> Default for DrawFunctions<P> {
    fn default() -> Self {
        Self {
            internal: RwLock::new(DrawFunctionsInternal {
                draw_functions: Vec::new(),
                indices: HashMap::default(),
            }),
        }
    }
}

impl<P: PhaseItem> DrawFunctions<P> {
    /// Accesses the draw functions in read mode.
    pub fn read(&self) -> RwLockReadGuard<'_, DrawFunctionsInternal<P>> {
        self.internal.read()
    }

    /// Accesses the draw functions in write mode.
    pub fn write(&self) -> RwLockWriteGuard<'_, DrawFunctionsInternal<P>> {
        self.internal.write()
    }
}

/// [`RenderCommand`] is a trait that runs an ECS query and produces one or more
/// [`TrackedRenderPass`] calls. Types implementing this trait can be composed (as tuples).
///
/// They can be registered as a [`Draw`] function via the
/// [`AddRenderCommand::add_render_command`] method.
///
pub trait RenderCommand<P: PhaseItem> {
    /// Specifies all ECS data required by [`RenderCommand::render`].
    /// All parameters have to be read only.
    type Param: SystemParam;

    /// Renders the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    fn render<'w>(
        view: Entity,
        item: &P,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult;
}

pub enum RenderCommandResult {
    Success,
    Failure,
}

pub trait EntityRenderCommand {
    type Param: SystemParam;
    fn render<'w>(
        view: Entity,
        item: Entity,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult;
}

pub trait EntityPhaseItem: PhaseItem {
    fn entity(&self) -> Entity;
}

pub trait CachedPipelinePhaseItem: PhaseItem {
    fn cached_pipeline(&self) -> CachedPipelineId;
}

/// A [`PhaseItem`] that can be batched dynamically.
///
/// Batching is an optimization that regroups multiple items in the same vertex buffer
/// to render them in a single draw call.
pub trait BatchedPhaseItem: EntityPhaseItem {
    /// Range in the vertex buffer of this item
    fn batch_range(&self) -> &Option<Range<u32>>;

    /// Range in the vertex buffer of this item
    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>>;

    /// Batches another item within this item if they are compatible.
    /// Items can be batched together if they have the same entity, and consecutive ranges.
    /// If batching is successful, the `other` item should be discarded from the render pass.
    #[inline]
    fn add_to_batch(&mut self, other: &Self) -> BatchResult {
        let self_entity = self.entity();
        if let (Some(self_batch_range), Some(other_batch_range)) = (
            self.batch_range_mut().as_mut(),
            other.batch_range().as_ref(),
        ) {
            // If the items are compatible, join their range into `self`
            if self_entity == other.entity() {
                if self_batch_range.end == other_batch_range.start {
                    self_batch_range.end = other_batch_range.end;
                    return BatchResult::Success;
                } else if self_batch_range.start == other_batch_range.end {
                    self_batch_range.start = other_batch_range.start;
                    return BatchResult::Success;
                }
            }
        }
        BatchResult::IncompatibleItems
    }
}

pub enum BatchResult {
    /// The `other` item was batched into `self`
    Success,
    /// `self` and `other` cannot be batched together
    IncompatibleItems,
}

impl<P: EntityPhaseItem, E: EntityRenderCommand> RenderCommand<P> for E {
    type Param = E::Param;

    #[inline]
    fn render<'w>(
        view: Entity,
        item: &P,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        <E as EntityRenderCommand>::render(view, item.entity(), param, pass)
    }
}

pub struct SetItemPipeline;
impl<P: CachedPipelinePhaseItem> RenderCommand<P> for SetItemPipeline {
    type Param = Res<'static, RenderPipelineCache>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: &P,
        pipeline_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(pipeline) = pipeline_cache.into_inner().get(item.cached_pipeline()) {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

macro_rules! render_command_tuple_impl {
    ($($name: ident),*) => {
        impl<P: PhaseItem, $($name: RenderCommand<P>),*> RenderCommand<P> for ($($name,)*) {
            type Param = ($($name::Param,)*);

            #[allow(non_snake_case)]
            fn render<'w>(
                _view: Entity,
                _item: &P,
                ($($name,)*): SystemParamItem<'w, '_, Self::Param>,
                _pass: &mut TrackedRenderPass<'w>,
            ) -> RenderCommandResult{
                $(if let RenderCommandResult::Failure = $name::render(_view, _item, $name, _pass) {
                    return RenderCommandResult::Failure;
                })*
                RenderCommandResult::Success
            }
        }
    };
}

all_tuples!(render_command_tuple_impl);

/// Wraps a [`RenderCommand`] into a state so that it can be used as a [`Draw`] function.
/// Therefore the [`RenderCommand::Param`] is queried from the ECS and passed to the command.
pub struct RenderCommandState<P: PhaseItem, C: RenderCommand<P>> {
    state: SystemState<C::Param>,
}

impl<P: PhaseItem, C: RenderCommand<P>> RenderCommandState<P, C> {
    pub fn new(world: &mut World) -> Self {
        Self {
            state: SystemState::new(world),
        }
    }
}

impl<P: PhaseItem, C: RenderCommand<P> + Send + Sync + 'static> Draw<P> for RenderCommandState<P, C>
where
    <C::Param as SystemParam>::Fetch: ReadOnlySystemParamFetch,
{
    /// Prepares the ECS parameters for the wrapped [`RenderCommand`] and then renders it.
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &P,
    ) {
        let param = self.state.get(world);
        C::render(view, item, param, pass);
    }
}
