use super::{
    handle::{Handle, HandleId, RefChange},
    loader::Asset,
};
use crate::ecs::{Event, EventWriter, Events, ResMut, Resource};
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::fmt::Debug;

pub enum AssetEvent<T: Asset> {
    Created { handle: Handle<T> },
    Modified { handle: Handle<T> },
    Removed { handle: Handle<T> },
}

impl<T: Asset> Event for AssetEvent<T> {}

impl<T: Asset> Debug for AssetEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetEvent::Created { handle } => f
                .debug_struct(&format!(
                    "AssetEvent<{}>::Created",
                    std::any::type_name::<T>()
                ))
                .field("handle", &handle.id)
                .finish(),
            AssetEvent::Modified { handle } => f
                .debug_struct(&format!(
                    "AssetEvent<{}>::Modified",
                    std::any::type_name::<T>()
                ))
                .field("handle", &handle.id)
                .finish(),
            AssetEvent::Removed { handle } => f
                .debug_struct(&format!(
                    "AssetEvent<{}>::Removed",
                    std::any::type_name::<T>()
                ))
                .field("handle", &handle.id)
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct Assets<T: Asset> {
    assets: HashMap<HandleId, T>,
    events: Events<AssetEvent<T>>,
    pub(crate) ref_change_sender: Sender<RefChange>,
}

impl<T: Asset> Resource for Assets<T> {}

impl<T: Asset> Assets<T> {
    pub(crate) fn new(ref_change_sender: Sender<RefChange>) -> Self {
        Assets {
            assets: HashMap::default(),
            events: Events::default(),
            ref_change_sender,
        }
    }

    pub fn add(&mut self, asset: T) -> Handle<T> {
        let id = HandleId::random::<T>();
        self.assets.insert(id, asset);
        self.events.send(AssetEvent::Created {
            handle: Handle::weak(id),
        });
        self.get_handle(id)
    }

    #[must_use = "not using the returned strong handle may result in the unexpected release of the asset"]
    pub fn set<H: Into<HandleId>>(&mut self, handle: H, asset: T) -> Handle<T> {
        let id: HandleId = handle.into();
        self.set_untracked(id, asset);
        self.get_handle(id)
    }

    pub fn set_untracked<H: Into<HandleId>>(&mut self, handle: H, asset: T) {
        let id: HandleId = handle.into();
        if self.assets.insert(id, asset).is_some() {
            self.events.send(AssetEvent::Modified {
                handle: Handle::weak(id),
            });
        } else {
            self.events.send(AssetEvent::Created {
                handle: Handle::weak(id),
            });
        }
    }

    pub fn get<H: Into<HandleId>>(&self, handle: H) -> Option<&T> {
        self.assets.get(&handle.into())
    }

    pub fn contains<H: Into<HandleId>>(&self, handle: H) -> bool {
        self.assets.contains_key(&handle.into())
    }

    pub fn get_mut<H: Into<HandleId>>(&mut self, handle: H) -> Option<&mut T> {
        let id: HandleId = handle.into();
        self.events.send(AssetEvent::Modified {
            handle: Handle::weak(id),
        });
        self.assets.get_mut(&id)
    }

    pub fn get_handle<H: Into<HandleId>>(&self, handle: H) -> Handle<T> {
        Handle::strong(handle.into(), self.ref_change_sender.clone())
    }

    pub fn get_or_insert_with<H: Into<HandleId>>(
        &mut self,
        handle: H,
        insert_fn: impl FnOnce() -> T,
    ) -> &mut T {
        let mut event = None;
        let id: HandleId = handle.into();
        let borrowed = self.assets.entry(id).or_insert_with(|| {
            event = Some(AssetEvent::Created {
                handle: Handle::weak(id),
            });
            insert_fn()
        });

        if let Some(event) = event {
            self.events.send(event);
        }
        borrowed
    }

    pub fn iter(&self) -> impl Iterator<Item = (HandleId, &T)> {
        self.assets.iter().map(|(k, v)| (*k, v))
    }

    pub fn ids(&self) -> impl Iterator<Item = HandleId> + '_ {
        self.assets.keys().cloned()
    }

    pub fn remove<H: Into<HandleId>>(&mut self, handle: H) -> Option<T> {
        let id: HandleId = handle.into();
        let asset = self.assets.remove(&id);
        if asset.is_some() {
            self.events.send(AssetEvent::Removed {
                handle: Handle::weak(id),
            });
        }
        asset
    }

    pub fn clear(&mut self) {
        self.assets.clear()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.assets.reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.assets.shrink_to_fit()
    }

    pub fn asset_event_system(
        mut events: EventWriter<AssetEvent<T>>,
        mut assets: ResMut<Assets<T>>,
    ) {
        // Check if the events are empty before calling `drain`. As `drain` triggers change detection.
        if !assets.events.is_empty() {
            events.send_batch(assets.events.drain())
        }
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

macro_rules! load_internal_asset {
    ($app: ident, $handle: ident, $path_str: expr, $loader: expr) => {{
        let mut assets = $app.world.resource_mut::<quad::asset::Assets<_>>();
        assets.set_untracked($handle, ($loader)(include_str!($path_str)));
    }};
}

pub(crate) use load_internal_asset;
