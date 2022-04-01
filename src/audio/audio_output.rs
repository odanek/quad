use rodio::{OutputStreamHandle, Sink, Source};
use std::marker::PhantomData;
use uuid::{uuid, Uuid};

use crate::{
    asset::{Asset, Assets},
    ecs::{Res, ResMut, Resource},
    reflect::TypeUuid,
};

use super::{Audio, AudioSource, Decodable};

/// Used internally to play audio on the current "audio device"
#[derive(Resource)]
pub struct AudioOutput<Source = AudioSource>
where
    Source: Decodable,
{
    stream_handle: Option<OutputStreamHandle>,
    phantom: PhantomData<Source>,
}

impl<Source> AudioOutput<Source>
where
    Source: Decodable,
{
    pub fn new(stream_handle: Option<OutputStreamHandle>) -> Self {
        Self {
            stream_handle,
            phantom: PhantomData,
        }
    }
}

impl<Source> AudioOutput<Source>
where
    Source: Asset + Decodable,
{
    fn play_source(&self, audio_source: &Source, repeat: bool) -> Option<Sink> {
        if let Some(stream_handle) = &self.stream_handle {
            let sink = Sink::try_new(stream_handle).unwrap();
            if repeat {
                sink.append(audio_source.decoder().repeat_infinite());
            } else {
                sink.append(audio_source.decoder());
            }
            Some(sink)
        } else {
            None
        }
    }

    fn try_play_queued(
        &self,
        audio_sources: &Assets<Source>,
        audio: &mut Audio<Source>,
        sinks: &mut Assets<AudioSink>,
    ) {
        let mut queue = audio.queue.write();
        let len = queue.len();
        let mut i = 0;
        while i < len {
            let config = queue.pop_front().unwrap();
            if let Some(audio_source) = audio_sources.get(&config.source_handle) {
                if let Some(sink) = self.play_source(audio_source, config.repeat) {
                    // don't keep the strong handle. there is no way to return it to the user here as it is async
                    let _ = sinks.set(config.sink_handle, AudioSink { sink: Some(sink) });
                }
            } else {
                // audio source hasn't loaded yet. add it back to the queue
                queue.push_back(config);
            }
            i += 1;
        }
    }
}

/// Plays audio currently queued in the [`Audio`] resource through the [`AudioOutput`] resource
pub fn play_queued_audio_system<Source: Asset>(
    audio_output: Res<AudioOutput<Source>>,
    mut audio: ResMut<Audio<Source>>,
    mut sinks: ResMut<Assets<AudioSink>>,
    audio_sources: Res<Assets<Source>>,
) where
    Source: Decodable,
{
    audio_output.try_play_queued(audio_sources.as_ref(), audio.as_mut(), sinks.as_mut());
}

/// Asset controlling the playback of a sound
pub struct AudioSink {
    // This field is an Option in order to allow us to have a safe drop that will detach the sink.
    // It will never be None during its life
    sink: Option<Sink>,
}

impl TypeUuid for AudioSink {
    const TYPE_UUID: Uuid = uuid!("8BEE570C-57C2-4FC0-8CFB-983A22F7D981");
}

impl Drop for AudioSink {
    fn drop(&mut self) {
        self.sink.take().unwrap().detach();
    }
}

impl AudioSink {
    /// Gets the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0`
    /// will multiply each sample by this value.
    pub fn volume(&self) -> f32 {
        self.sink.as_ref().unwrap().volume()
    }

    /// Changes the volume of the sound.
    ///
    /// The value `1.0` is the "normal" volume (unfiltered input). Any value other than `1.0`
    /// will multiply each sample by this value.
    pub fn set_volume(&self, volume: f32) {
        self.sink.as_ref().unwrap().set_volume(volume);
    }

    /// Gets the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0`
    /// will change the play speed of the sound.
    pub fn speed(&self) -> f32 {
        self.sink.as_ref().unwrap().speed()
    }

    /// Changes the speed of the sound.
    ///
    /// The value `1.0` is the "normal" speed (unfiltered input). Any value other than `1.0`
    /// will change the play speed of the sound.
    pub fn set_speed(&self, speed: f32) {
        self.sink.as_ref().unwrap().set_speed(speed);
    }

    /// Resumes playback of a paused sink.
    ///
    /// No effect if not paused.
    pub fn play(&self) {
        self.sink.as_ref().unwrap().play();
    }

    /// Pauses playback of this sink.
    ///
    /// No effect if already paused.
    /// A paused sink can be resumed with [`play`](Self::play).
    pub fn pause(&self) {
        self.sink.as_ref().unwrap().pause();
    }

    /// Is this sink paused?
    ///
    /// Sinks can be paused and resumed using [`pause`](Self::pause) and [`play`](Self::play).
    pub fn is_paused(&self) -> bool {
        self.sink.as_ref().unwrap().is_paused()
    }
}
