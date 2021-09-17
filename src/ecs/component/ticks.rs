// Assuming 256 fps and 128 systems this will wrap after 36 hours
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Tick(u32);

impl Tick {
    pub(crate) fn new(tick: u32) -> Self {
        Tick(tick)
    }
}

#[derive(Clone, Debug)]
pub struct ComponentTicks {
    added: Tick,
    changed: Tick,
}

impl ComponentTicks {
    #[inline]
    pub(crate) fn new(change_tick: Tick) -> Self {
        Self {
            added: change_tick,
            changed: change_tick,
        }
    }

    #[inline]
    pub fn is_added(&self, last_change_tick: Tick) -> bool {
        self.added > last_change_tick
    }

    #[inline]
    pub fn is_changed(&self, last_change_tick: Tick) -> bool {
        self.changed > last_change_tick
    }

    #[inline]
    pub fn set_changed(&mut self, change_tick: Tick) {
        self.changed = change_tick;
    }
}
