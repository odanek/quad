#![forbid(unsafe_code)]

#[allow(clippy::module_inception)]
mod audio;
mod audio_output;
mod audio_source;

pub use audio::*;
pub use audio_output::*;
pub use audio_source::*;
use rodio::{OutputStream, OutputStreamHandle};

use crate::app::{App, Stage};

pub struct AudioDevice {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
}

impl Default for AudioDevice {
    fn default() -> Self {
        if let Ok((stream, stream_handle)) = OutputStream::try_default() {
            Self {
                _stream: Some(stream),
                stream_handle: Some(stream_handle),
            }
        } else {
            log::warn!("No audio device found.");
            Self {
                _stream: None,
                stream_handle: None,
            }
        }
    }
}

pub fn audio_plugin(app: &mut App, audio_device: &AudioDevice) {
    app.add_asset::<AudioSource>()
        .add_asset::<AudioSink>()
        .insert_resource(AudioOutput::<AudioSource>::new(
            audio_device.stream_handle.clone(),
        ))
        .init_resource::<Audio<AudioSource>>()
        .add_system_to_stage(Stage::PostUpdate, &play_queued_audio_system::<AudioSource>);

    app.init_asset_loader::<AudioLoader>();
}
