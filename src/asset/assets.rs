use super::{
    handle::{Handle, HandleId, RefChange},
    loader::Asset,
};
use crate::ecs::Events;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::fmt::Debug;

pub enum AssetEvent<T: Asset> {
    Created { handle: Handle<T> },
    Modified { handle: Handle<T> },
    Removed { handle: Handle<T> },
}

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
