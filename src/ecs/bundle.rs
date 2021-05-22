use std::{any::{TypeId, type_name}, collections::HashMap};

use super::component::{Component, ComponentId, Components, StorageType, type_info::TypeInfo};

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
    pub(crate) storage_types: Vec<StorageType>,
}

impl BundleInfo {
    #[inline]
    pub fn id(&self) -> BundleId {
        self.id
    }

    #[inline]
    pub fn components(&self) -> &[ComponentId] {
        &self.component_ids
    }

    #[inline]
    pub fn storage_types(&self) -> &[StorageType] {
        &self.storage_types
    }
}

#[derive(Default)]
pub struct Bundles {
    bundle_infos: Vec<BundleInfo>,
    bundle_ids: HashMap<TypeId, BundleId>,
}

impl Bundles {
    #[inline]
    pub fn get(&self, bundle_id: BundleId) -> Option<&BundleInfo> {
        self.bundle_infos.get(bundle_id.index())
    }

    #[inline]
    pub fn get_id(&self, type_id: TypeId) -> Option<BundleId> {
        self.bundle_ids.get(&type_id).cloned()
    }

    pub(crate) fn init_info<'a, T: Bundle>(
        &'a mut self,
        components: &mut Components,
    ) -> &'a BundleInfo {
        let bundle_infos = &mut self.bundle_infos;
        let id = self.bundle_ids.entry(TypeId::of::<T>()).or_insert_with(|| {
            let type_info = T::type_info();
            let id = BundleId(bundle_infos.len());
            let bundle_info =
                initialize_bundle(type_name::<T>(), &type_info, id, components);
            bundle_infos.push(bundle_info);
            id
        });
        unsafe { self.bundle_infos.get_unchecked(id.0) }
    }    
}

fn initialize_bundle(
    bundle_type_name: &'static str,
    type_info: &[TypeInfo],
    id: BundleId,
    components: &mut Components,
) -> BundleInfo {
    let mut component_ids = Vec::new();
    let mut storage_types = Vec::new();

    for type_info in type_info {
        let component_id = components.get_or_insert(&type_info);        
        let info = components.get_info(component_id).unwrap();
        component_ids.push(component_id);
        storage_types.push(info.storage_type());
    }

    let mut deduped = component_ids.clone();
    deduped.sort();
    deduped.dedup();
    if deduped.len() != component_ids.len() {
        panic!("Bundle {} has duplicate components", bundle_type_name);
    }

    BundleInfo {
        id,
        component_ids,
        storage_types,
    }
}
