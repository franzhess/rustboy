use ::sdl3::audio::{AudioFormat, AudioSpec, AudioStreamOwner};
use ::sdl3::Sdl;
use rustboy_core::AUDIO_OUTPUT_FREQUENCY;

pub struct Sound {
    stream: AudioStreamOwner,
}

impl Sound {
    pub fn new(sdl: &Sdl) -> Result<Self, sdl3::Error> {
        let audio = sdl.audio()?;
        let spec = AudioSpec {
            freq: Some(AUDIO_OUTPUT_FREQUENCY as i32),
            channels: Some(2),
            format: Some(AudioFormat::s16_sys()),
        };
        let device = audio.open_playback_device(&spec)?;
        Ok(Self {
            stream: device.open_device_stream(Some(&spec))?,
        })
    }

    pub fn queue(&mut self, data: Vec<i16>) -> Result<(), sdl3::Error> {
        self.stream.put_data_i16(&data)
    }

    pub fn play(&mut self) -> Result<(), sdl3::Error> {
        self.stream.resume()
    }
    pub fn stop(&mut self) -> Result<(), sdl3::Error> {
        self.stream.pause()
    }
}
