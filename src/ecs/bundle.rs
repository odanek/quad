use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use super::{
    component::{type_info::TypeInfo, Component, ComponentId, Components},
    storage::Table,
};

pub trait Bundle: Send + Sync + 'static {
    fn type_info() -> Vec<TypeInfo>;

    fn get_components(self, func: impl FnMut(*mut u8));
}

macro_rules! bundle_impl {
    ($($name: ident),*) => {
        impl<$($name: Component),*> Bundle for ($($name,)*) {
            fn type_info() -> Vec<TypeInfo> {
                vec![$(TypeInfo::of::<$name>()),*]
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
    pub(crate) unsafe fn write_components<T: Bundle>(
        &self,
        table: &Table,
        table_row: usize,
        bundle: T,
    ) {
        let mut bundle_component = 0;
        bundle.get_components(|component_ptr| {
            let component_id = *self.component_ids.get_unchecked(bundle_component);
            let column = table.get_column(component_id).unwrap();
            column.set_unchecked(table_row, component_ptr);
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
    pub(crate) fn init_info<'a, T: Bundle>(
        &'a mut self,
        components: &mut Components,
    ) -> &'a BundleInfo {
        let bundle_infos = &mut self.bundle_infos;
        let id = self.bundle_ids.entry(TypeId::of::<T>()).or_insert_with(|| {
            let type_info = T::type_info();
            let id = BundleId(bundle_infos.len());
            let bundle_info = initialize_bundle(type_name::<T>(), &type_info, id, components);
            bundle_infos.push(bundle_info);
            id
        });
        unsafe { self.bundle_infos.get_unchecked(id.0) }
    }
}

// TODO: Are (A, B) and (B, A) different bundles? Sort components to treat them as the same bundle?
fn initialize_bundle(
    bundle_type_name: &'static str,
    type_info: &[TypeInfo],
    id: BundleId,
    components: &mut Components,
) -> BundleInfo {
    let mut component_ids = Vec::new();

    for type_info in type_info {
        let component_id = components.get_or_insert(&type_info);
        component_ids.push(component_id);
    }

    let mut deduped = component_ids.clone();
    deduped.sort();
    deduped.dedup();
    if deduped.len() != component_ids.len() {
        panic!("Bundle {} has duplicate components", bundle_type_name);
    }

    BundleInfo { id, component_ids }
}
