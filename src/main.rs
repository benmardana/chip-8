mod args;
mod audio;
mod cpu;
mod renderer;
extern crate sdl2;

use anyhow::{Error, Result};
use args::parse_args;
use audio::AudioPlayer;
use cpu::Cpu;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, SystemTime};

const HZ: f64 = 3000.0;

fn main() -> Result<()> {
    let cpu = Mutex::new(Cpu::new().load(parse_args()?.path));
    let timer_arc = Arc::new(cpu);
    let cpu_lock = Arc::clone(&timer_arc);

    let sdl_context = sdl2::init().map_err(Error::msg)?;
    let mut renderer = Renderer::new(&sdl_context).map_err(Error::msg)?;
    let audio_player = AudioPlayer::new(&sdl_context).map_err(Error::msg)?;

    let mut event_pump = renderer.event_pump();

    thread::spawn(move || -> Result<()> {
        loop {
            let start = SystemTime::now();
            let mut guard = timer_arc.lock().unwrap();
            guard.tick_timers();
            drop(guard);
            sleep(Duration::from_secs_f64(1.0 / 60.0).saturating_sub(start.elapsed()?));
        }
    });

    'running: loop {
        let start = SystemTime::now();

        for event in event_pump.poll_iter() {
            if let Event::KeyDown {
                scancode: Some(x), ..
            } = event
            {
                match x {
                    Scancode::Escape => break 'running,
                    Scancode::X => 0x00,
                    Scancode::Num1 => 0x01,
                    Scancode::Num2 => 0x02,
                    Scancode::Num3 => 0x03,
                    Scancode::Q => 0x04,
                    Scancode::W => 0x05,
                    Scancode::E => 0x06,
                    Scancode::A => 0x07,
                    Scancode::S => 0x08,
                    Scancode::D => 0x09,
                    Scancode::Z => 0x0A,
                    Scancode::C => 0x0B,
                    Scancode::Num4 => 0x0C,
                    Scancode::R => 0x0D,
                    Scancode::F => 0x0E,
                    Scancode::V => 0x0F,
                    _ => 0x29,
                };
            }
        }
        let mut guard = cpu_lock.lock().unwrap();

        guard.tick(&event_pump);

        // handle output
        if guard.should_draw() {
            renderer.draw_screen(guard.screen);
        }

        if guard.should_beep() {
            audio_player.beep();
        } else {
            audio_player.stop_beep();
        }

        drop(guard);
        sleep(Duration::from_secs_f64(1.0 / HZ).saturating_sub(start.elapsed()?));
    }
    Ok(())
}
