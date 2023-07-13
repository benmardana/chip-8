use anyhow::{Error, Result};
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus},
    Sdl,
};

pub struct AudioPlayer {
    device: AudioDevice<SquareWave>,
}

impl AudioPlayer {
    pub fn new(sdl_context: &Sdl) -> Result<AudioPlayer> {
        const AUDIO_SPEC_DESIRED: AudioSpecDesired = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };
        let audio_subsystem = sdl_context.audio().map_err(Error::msg)?;
        let device = audio_subsystem
            .open_playback(None, &AUDIO_SPEC_DESIRED, |spec| SquareWave {
                phase_inc: 220.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            })
            .map_err(Error::msg)?;
        Ok(AudioPlayer { device })
    }

    pub fn beep(&self) {
        if let AudioStatus::Paused = self.device.status() {
            self.device.resume()
        }
    }

    pub fn stop_beep(&self) {
        if let AudioStatus::Playing = self.device.status() {
            self.device.pause()
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
