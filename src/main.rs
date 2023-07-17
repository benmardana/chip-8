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
    let args = parse_args()?;
    let hertz = args.hertz.unwrap_or(HZ);
    let cpu = Mutex::new(Cpu::new().load(args.path));
    let timer_arc = Arc::new(cpu);
    let cpu_lock = Arc::clone(&timer_arc);

    let sdl_context = sdl2::init().map_err(Error::msg)?;
    let mut renderer = Renderer::new(&sdl_context).map_err(Error::msg)?;
    let audio_player = AudioPlayer::new(&sdl_context).map_err(Error::msg)?;

    let mut event_pump = renderer.event_pump();

    // Run timers in a 60hz cycle
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
                scancode: Some(Scancode::Escape),
                ..
            } = event
            {
                break 'running;
            }
        }

        let mut guard = cpu_lock.lock().unwrap();

        guard.tick(&event_pump);

        if guard.should_draw() {
            renderer.draw_screen(guard.screen);
        }

        if guard.should_beep() {
            audio_player.beep();
        } else {
            audio_player.stop_beep();
        }

        drop(guard);
        sleep(Duration::from_secs_f64(1.0 / hertz).saturating_sub(start.elapsed()?));
    }
    Ok(())
}
