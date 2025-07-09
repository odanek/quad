use std::{fmt, marker::PhantomData};

use super::{ResMut, Resource};

pub trait Event: Send + Sync + 'static {}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EventId<T> {
    pub id: usize,
    _marker: PhantomData<fn() -> T>,
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

impl<T> Resource for Events<T> where T: Send + Sync + 'static {}

fn map_instance_event_with_id<T>(event_instance: &EventInstance<T>) -> (&T, EventId<T>) {
    (&event_instance.event, event_instance.event_id)
}

pub struct EventReader<'w, 's, T: Event> {
    events: &'w Events<T>,
    last_event_count: &'s mut usize,
}

impl<'w, 's, T: Event> EventReader<'w, 's, T> {
    pub(crate) fn new(events: &'w Events<T>, last_event_count: &'s mut usize) -> Self {
        Self {
            events,
            last_event_count,
        }
    }

    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &T> {
        self.iter_with_id().map(|(event, _id)| event)
    }

    pub fn iter_with_id(&mut self) -> impl DoubleEndedIterator<Item = (&T, EventId<T>)> {
        internal_event_reader(self.last_event_count, self.events)
    }
}

pub struct EventWriter<'w, T: Event> {
    events: &'w mut Events<T>,
}

impl<'w, T: Event> EventWriter<'w, T> {
    pub(crate) fn new(events: &'w mut Events<T>) -> Self {
        Self { events }
    }

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
    let a_index = (*last_event_count).saturating_sub(events.a_start_event_count);
    let b_index = (*last_event_count).saturating_sub(events.b_start_event_count);
    *last_event_count = events.event_count;
    match events.state {
        State::A => events
            .events_b
            .get(b_index..)
            .unwrap_or(&[])
            .iter()
            .map(map_instance_event_with_id)
            .chain(
                events
                    .events_a
                    .get(a_index..)
                    .unwrap_or(&[])
                    .iter()
                    .map(map_instance_event_with_id),
            ),
        State::B => events
            .events_a
            .get(a_index..)
            .unwrap_or(&[])
            .iter()
            .map(map_instance_event_with_id)
            .chain(
                events
                    .events_b
                    .get(b_index..)
                    .unwrap_or(&[])
                    .iter()
                    .map(map_instance_event_with_id),
            ),
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

        self.event_count = event_count;
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::ecs::{Event, Events};

    use super::internal_event_reader;

    #[derive(Copy, Clone, PartialEq, Eq, Debug, Event)]
    struct TestEvent {
        i: usize,
    }

    pub struct ManualEventReader<T> {
        last_event_count: usize,
        _marker: PhantomData<T>,
    }

    impl<T: Event> ManualEventReader<T> {
        pub fn new() -> Self {
            ManualEventReader {
                last_event_count: 0,
                _marker: PhantomData,
            }
        }

        pub fn iter<'a>(
            &mut self,
            events: &'a Events<T>,
        ) -> impl DoubleEndedIterator<Item = &'a T> {
            internal_event_reader(&mut self.last_event_count, events).map(|(e, _)| e)
        }
    }

    fn get_events(
        events: &Events<TestEvent>,
        reader: &mut ManualEventReader<TestEvent>,
    ) -> Vec<TestEvent> {
        reader.iter(events).cloned().collect::<Vec<TestEvent>>()
    }

    #[test]
    fn test_events() {
        let mut events = Events::<TestEvent>::default();
        let event_0 = TestEvent { i: 0 };
        let event_1 = TestEvent { i: 1 };
        let event_2 = TestEvent { i: 2 };

        let mut reader_missed = ManualEventReader::new();
        let mut reader_a = ManualEventReader::new();

        events.send(event_0);

        assert_eq!(
            get_events(&events, &mut reader_a),
            vec![event_0],
            "reader_a created before event receives event"
        );
        assert_eq!(
            get_events(&events, &mut reader_a),
            vec![],
            "second iteration of reader_a created before event results in zero events"
        );

        let mut reader_b = ManualEventReader::new();

        assert_eq!(
            get_events(&events, &mut reader_b),
            vec![event_0],
            "reader_b created after event receives event"
        );
        assert_eq!(
            get_events(&events, &mut reader_b),
            vec![],
            "second iteration of reader_b created after event results in zero events"
        );

        events.send(event_1);

        let mut reader_c = ManualEventReader::new();

        assert_eq!(
            get_events(&events, &mut reader_c),
            vec![event_0, event_1],
            "reader_c created after two events receives both events"
        );
        assert_eq!(
            get_events(&events, &mut reader_c),
            vec![],
            "second iteration of reader_c created after two event results in zero events"
        );

        assert_eq!(
            get_events(&events, &mut reader_a),
            vec![event_1],
            "reader_a receives next unread event"
        );

        events.update();

        let mut reader_d = ManualEventReader::new();

        events.send(event_2);

        assert_eq!(
            get_events(&events, &mut reader_a),
            vec![event_2],
            "reader_a receives event created after update"
        );
        assert_eq!(
            get_events(&events, &mut reader_b),
            vec![event_1, event_2],
            "reader_b receives events created before and after update"
        );
        assert_eq!(
            get_events(&events, &mut reader_d),
            vec![event_0, event_1, event_2],
            "reader_d receives all events created before and after update"
        );

        events.update();

        assert_eq!(
            get_events(&events, &mut reader_missed),
            vec![event_2],
            "reader_missed missed events unread after two update() calls"
        );
    }
}
