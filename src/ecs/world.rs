use std::cell::{Ref, RefMut};

use super::ResourceContainer;

pub struct World {
    resources: ResourceContainer, // Struct of arrays
}

impl Default for World {
    fn default() -> Self {
        World {
            resources: Default::default(),
        }
    }
}

impl World {
    pub fn add_resource<T: 'static>(&mut self, resource: Box<T>) {
        self.resources.add(resource);
    }

    pub fn remove_resource<T: 'static>(&mut self) -> Box<T> {
        self.resources.remove()
    }

    pub fn get_resource<T: 'static>(&self) -> Ref<T> {
        self.resources.get()
    }

    pub fn get_resource_mut<T: 'static>(& self) -> RefMut<T> {
        self.resources.get_mut()
    }
}
