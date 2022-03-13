use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use crate::ecs::storage::Table;

use super::{
    ticks::{ComponentTicks, Tick},
    Component, ComponentId, ComponentStatus, Components,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Bundle: Send + Sync + 'static {
    fn component_ids(components: &mut Components) -> Vec<ComponentId>;

    unsafe fn from_components(func: impl FnMut() -> *mut u8) -> Self
    where
        Self: Sized;

    fn get_components(self, func: impl FnMut(*mut u8));
}

macro_rules! bundle_impl {
    ($($name: ident),*) => {
        unsafe impl<$($name: Component),*> Bundle for ($($name,)*) {
            fn component_ids(_components: &mut Components) -> Vec<ComponentId> {
                vec![$(_components.get_or_insert::<$name>()),*]
            }

            #[allow(unused_variables, unused_mut, clippy::unused_unit)]
            unsafe fn from_components(mut func: impl FnMut() -> *mut u8) -> Self {
                #[allow(non_snake_case)]
                let ($(mut $name,)*) = (
                    $(func().cast::<$name>(),)*
                );
                ($($name.read(),)*)
            }

            #[allow(unused_variables, unused_mut)]
            fn get_components(self, mut func: impl FnMut(*mut u8)) {
                #[allow(non_snake_case)]
                let ($(mut $name,)*) = self;
                $(
                    func((&mut $name as *mut $name).cast::<u8>());
                    std::mem::forget($name);
                )*
            }
        }
    }
}

all_tuples!(bundle_impl);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BundleId(usize);

impl BundleId {
    #[inline]
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct BundleInfo {
    pub(crate) id: BundleId,
    pub(crate) component_ids: Vec<ComponentId>,
}

impl BundleInfo {
    #[inline]
    pub unsafe fn write_components<T: Bundle>(
        &self,
        table: &mut Table,
        table_row: usize,
        bundle: T,
        bundle_status: &[ComponentStatus],
        change_tick: Tick,
    ) {
        let mut bundle_component = 0;
        bundle.get_components(|component_ptr| {
            let component_id = *self.component_ids.get_unchecked(bundle_component);
            let status = bundle_status.get_unchecked(bundle_component);
            let column = table.get_column_mut(component_id).unwrap();

            match status {
                ComponentStatus::Added => {
                    column.initialize(table_row, component_ptr, ComponentTicks::new(change_tick));
                }
                ComponentStatus::Mutated => {
                    column.replace(table_row, component_ptr, change_tick);
                }
            }

            bundle_component += 1;
        });
    }
}

#[derive(Default)]
pub struct Bundles {
    bundle_infos: Vec<BundleInfo>,
    bundle_ids: HashMap<TypeId, BundleId>,
}

impl Bundles {
    pub fn init_info<'a, T: Bundle>(&'a mut self, components: &mut Components) -> &'a BundleInfo {
        let bundle_infos = &mut self.bundle_infos;
        let id = self.bundle_ids.entry(TypeId::of::<T>()).or_insert_with(|| {
            let component_ids = T::component_ids(components);
            let id = BundleId(bundle_infos.len());
            let bundle_info = initialize_bundle(type_name::<T>(), component_ids, id);
            bundle_infos.push(bundle_info);
            id
        });
        unsafe { self.bundle_infos.get_unchecked(id.0) }
    }
}

// TODO: Are (A, B) and (B, A) different bundles? Sort components to treat them as the same bundle?
fn initialize_bundle(
    bundle_type_name: &'static str,
    component_ids: Vec<ComponentId>,
    id: BundleId,
) -> BundleInfo {
    let mut deduped = component_ids.clone();
    deduped.sort();
    deduped.dedup();
    if deduped.len() != component_ids.len() {
        panic!("Bundle {} has duplicate components", bundle_type_name);
    }

    BundleInfo { id, component_ids }
}
