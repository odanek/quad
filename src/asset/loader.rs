use downcast_rs::{impl_downcast, Downcast};

use crate::reflect::{TypeUuid, TypeUuidDynamic};

pub trait Asset: TypeUuid + AssetDynamic {}

pub trait AssetDynamic: Downcast + TypeUuidDynamic + Send + Sync + 'static {}
impl_downcast!(AssetDynamic);

impl<T> Asset for T where T: TypeUuid + AssetDynamic + TypeUuidDynamic {}

impl<T> AssetDynamic for T where T: Send + Sync + 'static + TypeUuidDynamic {}
