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
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const HZ: f64 = 2000.0;

fn main() -> Result<()> {
    let mut cpu = Cpu::new().load(parse_args()?.path);

    let sdl_context = sdl2::init().map_err(Error::msg)?;
    let mut renderer = Renderer::new(&sdl_context).map_err(Error::msg)?;
    let audio_player = AudioPlayer::new(&sdl_context).map_err(Error::msg)?;

    let mut event_pump = renderer.event_pump();

    let mut cycle: f64 = 0.0;
    'running: loop {
        let start = SystemTime::now();
        cycle += 1.0;

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

        cpu.tick(&event_pump);

        if cycle >= HZ / 60.0 {
            cpu.tick_timers();
            cycle = 0.0;
        }

        // handle output
        if cpu.should_draw() {
            renderer.draw_screen(cpu.screen);
        }

        if cpu.should_beep() {
            audio_player.beep();
        } else {
            audio_player.stop_beep();
        }

        println!("{:#?}", start.elapsed());
        sleep(Duration::from_secs_f64(1.0 / HZ).saturating_sub(start.elapsed()?));
    }
    Ok(())
}
