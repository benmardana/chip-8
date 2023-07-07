use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus},
    Sdl,
};

pub struct AudioPlayer {
    device: AudioDevice<SquareWave>,
}

impl AudioPlayer {
    pub fn new(sdl_context: &Sdl) -> Result<AudioPlayer, String> {
        const AUDIO_SPEC_DESIRED: AudioSpecDesired = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };
        let audio_subsystem = sdl_context.audio().unwrap();
        let device = audio_subsystem
            .open_playback(None, &AUDIO_SPEC_DESIRED, |spec| SquareWave {
                phase_inc: 220.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            })
            .unwrap();
        Ok(AudioPlayer { device })
    }

    pub fn beep(&self) {
        match self.device.status() {
            AudioStatus::Paused => self.device.resume(),
            _ => (),
        }
    }

    pub fn stop_beep(&self) {
        match self.device.status() {
            AudioStatus::Playing => self.device.pause(),
            _ => (),
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
