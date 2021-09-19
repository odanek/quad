use std::{fmt, marker::PhantomData};

use log::trace;

use super::{Local, Res, ResMut};

pub trait Event: Send + Sync + 'static {}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EventId<T> {
    pub id: usize,
    _marker: PhantomData<T>,
}

impl<T> Copy for EventId<T> {}
impl<T> Clone for EventId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Display for EventId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl<T> fmt::Debug for EventId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "event<{}>#{}",
            std::any::type_name::<T>().split("::").last().unwrap(),
            self.id,
        )
    }
}

#[derive(Debug)]
struct EventInstance<T> {
    pub event_id: EventId<T>,
    pub event: T,
}

#[derive(Debug)]
enum State {
    A,
    B,
}

#[derive(Debug)]
pub struct Events<T> {
    events_a: Vec<EventInstance<T>>,
    events_b: Vec<EventInstance<T>>,
    a_start_event_count: usize,
    b_start_event_count: usize,
    event_count: usize,
    state: State,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Events {
            a_start_event_count: 0,
            b_start_event_count: 0,
            event_count: 0,
            events_a: Vec::new(),
            events_b: Vec::new(),
            state: State::A,
        }
    }
}

fn map_instance_event_with_id<T>(event_instance: &EventInstance<T>) -> (&T, EventId<T>) {
    (&event_instance.event, event_instance.event_id)
}

// #[derive(SystemParam)] TODO
pub struct EventReader<'w, 's, T: Event> {
    last_event_count: Local<'s, (usize, PhantomData<T>)>,
    events: Res<'w, Events<T>>,
}

/// Sends events of type `T`.
// #[derive(SystemParam)] TODO
pub struct EventWriter<'w, 's, T: Event> {
    events: ResMut<'w, Events<T>>,
    // #[system_param(ignore)] TODO
    marker: PhantomData<&'s usize>,
}

impl<'w, 's, T: Event> EventWriter<'w, 's, T> {
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }

    pub fn send_batch(&mut self, events: impl Iterator<Item = T>) {
        self.events.extend(events);
    }
}

fn internal_event_reader<'a, T>(
    last_event_count: &mut usize,
    events: &'a Events<T>,
) -> impl DoubleEndedIterator<Item = (&'a T, EventId<T>)> {
    let a_index = if *last_event_count > events.a_start_event_count {
        *last_event_count - events.a_start_event_count
    } else {
        0
    };
    let b_index = if *last_event_count > events.b_start_event_count {
        *last_event_count - events.b_start_event_count
    } else {
        0
    };
    *last_event_count = events.event_count;
    match events.state {
        State::A => events
            .events_b
            .get(b_index..)
            .unwrap_or_else(|| &[])
            .iter()
            .map(map_instance_event_with_id)
            .chain(
                events
                    .events_a
                    .get(a_index..)
                    .unwrap_or_else(|| &[])
                    .iter()
                    .map(map_instance_event_with_id),
            ),
        State::B => events
            .events_a
            .get(a_index..)
            .unwrap_or_else(|| &[])
            .iter()
            .map(map_instance_event_with_id)
            .chain(
                events
                    .events_b
                    .get(b_index..)
                    .unwrap_or_else(|| &[])
                    .iter()
                    .map(map_instance_event_with_id),
            ),
    }
}

impl<'w, 's, T: Event> EventReader<'w, 's, T> {
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &T> {
        self.iter_with_id().map(|(event, _id)| event)
    }

    pub fn iter_with_id(&mut self) -> impl DoubleEndedIterator<Item = (&T, EventId<T>)> {
        internal_event_reader(&mut self.last_event_count.0, &self.events)
    }
}

impl<T: Event> Events<T> {
    pub fn send(&mut self, event: T) {
        let event_id = EventId {
            id: self.event_count,
            _marker: PhantomData,
        };

        let event_instance = EventInstance { event_id, event };

        match self.state {
            State::A => self.events_a.push(event_instance),
            State::B => self.events_b.push(event_instance),
        }

        self.event_count += 1;
    }

    pub fn update(&mut self) {
        match self.state {
            State::A => {
                self.events_b = Vec::new();
                self.state = State::B;
                self.b_start_event_count = self.event_count;
            }
            State::B => {
                self.events_a = Vec::new();
                self.state = State::A;
                self.a_start_event_count = self.event_count;
            }
        }
    }

    // TODO
    pub fn update_system(mut events: ResMut<Self>) {
        events.update();
    }

    #[inline]
    fn reset_start_event_count(&mut self) {
        self.a_start_event_count = self.event_count;
        self.b_start_event_count = self.event_count;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.reset_start_event_count();
        self.events_a.clear();
        self.events_b.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events_a.is_empty() && self.events_b.is_empty()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.reset_start_event_count();

        let map = |i: EventInstance<T>| i.event;
        match self.state {
            State::A => self
                .events_b
                .drain(..)
                .map(map)
                .chain(self.events_a.drain(..).map(map)),
            State::B => self
                .events_a
                .drain(..)
                .map(map)
                .chain(self.events_b.drain(..).map(map)),
        }
    }
}

impl<T> std::iter::Extend<T> for Events<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut event_count = self.event_count;
        let events = iter.into_iter().map(|event| {
            let event_id = EventId {
                id: event_count,
                _marker: PhantomData,
            };
            event_count += 1;
            EventInstance { event_id, event }
        });

        match self.state {
            State::A => self.events_a.extend(events),
            State::B => self.events_b.extend(events),
        }

        trace!(
            "Events::extend() -> ids: ({}..{})",
            self.event_count,
            event_count
        );
        self.event_count = event_count;
    }
}
