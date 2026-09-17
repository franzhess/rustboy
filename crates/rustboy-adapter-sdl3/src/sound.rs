use ::sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};
use ::sdl3::Sdl;
use rustboy_core::AUDIO_OUTPUT_FREQUENCY;

pub struct Sound {
    stream: AudioStreamOwner,
}

impl Sound {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let audio = sdl.audio().map_err(|error| error.to_string())?;
        let spec = AudioSpec {
            freq: Some(AUDIO_OUTPUT_FREQUENCY as i32),
            channels: Some(2),
            format: Some(AudioFormat::s16_sys()),
        };
        let device = audio
            .open_playback_device(&spec)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stream: device
                .open_device_stream(Some(&spec))
                .map_err(|error| error.to_string())?,
        })
    }

    pub fn queue(&mut self, data: Vec<i16>) -> Result<(), String> {
        self.stream
            .put_data_i16(&data)
            .map_err(|error| error.to_string())
    }

    pub fn play(&mut self) -> Result<(), String> {
        self.stream.resume().map_err(|error| error.to_string())
    }
    pub fn stop(&mut self) -> Result<(), String> {
        self.stream.pause().map_err(|error| error.to_string())
    }
}
