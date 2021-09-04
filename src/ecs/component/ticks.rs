#[derive(Clone, Debug)]
pub struct ComponentTicks {
    pub(crate) added: u32,
    pub(crate) changed: u32,
}

impl ComponentTicks {
    #[inline]
    pub(crate) fn new(change_tick: u32) -> Self {
        Self {
            added: change_tick,
            changed: change_tick,
        }
    }

    #[inline]
    pub fn is_added(&self, last_change_tick: u32, change_tick: u32) -> bool {
        let component_delta = change_tick.wrapping_sub(self.added);
        let system_delta = change_tick.wrapping_sub(last_change_tick);
        component_delta < system_delta
    }

    #[inline]
    pub fn is_changed(&self, last_change_tick: u32, change_tick: u32) -> bool {
        let component_delta = change_tick.wrapping_sub(self.changed);
        let system_delta = change_tick.wrapping_sub(last_change_tick);
        component_delta < system_delta
    }
    

    #[inline]
    pub fn set_changed(&mut self, change_tick: u32) {
        self.changed = change_tick;
    }
}
