use super::ResourceContainer;

pub struct World {
    resources: ResourceContainer
    // Struct of arrays
}

impl Default for World {
    fn default() -> Self {
        World {
            resources: Default::default()
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

    pub fn get_resource<'a, T: 'static>(&'a self) -> &'a T {
        self.resources.get()
    }

    pub fn get_resource_mut<'a, T: 'static>(&'a mut self) -> &'a mut T {
        self.resources.get_mut()
    }
}