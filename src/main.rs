mod audio;
mod cpu;
mod renderer;
extern crate sdl2;

use anyhow::{Error, Result};
use audio::AudioPlayer;
use cpu::Cpu;
use lexopt::Arg::{Long, Short, Value};
use lexopt::{Parser, ValueExt};
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut cpu = Cpu::new().load(parse_args()?.path);

    let sdl_context = sdl2::init().unwrap();
    let mut renderer = Renderer::new(&sdl_context).unwrap();
    let audio_player = AudioPlayer::new(&sdl_context).unwrap();

    let mut event_pump = renderer.event_pump();

    let mut cycle: usize = 0;
    'running: loop {
        cycle += 1;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        if let Some(x) = cpu.awaiting_key {
            if let Some(key) = cpu.some_key_pressed(&event_pump) {
                cpu.set_register(x, key);
            }
        } else {
            let instruction = cpu.fetch();
            let opcode = cpu.decode(instruction);
            cpu.execute(opcode, &event_pump);
        }

        renderer.draw_screen(cpu.screen);

        // Every 8th tick, decrement timers.
        if cycle == 8 {
            cpu.drop_timers();
            if cpu.should_beep() {
                audio_player.beep();
            } else {
                audio_player.stop_beep();
            }
            cycle = 0;
        }

        // Sleep at a rate that emulates about 500Hz.
        sleep(Duration::new(0, 2_000_000u32))
    }
    Ok(())
}

struct Args {
    path: String,
}

fn parse_args() -> Result<Args> {
    let mut path = None;
    let mut parser = Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) if path.is_none() => {
                path = Some(val.string()?);
            }
            Long("help") => {
                println!("Usage: chip-8 PATH");
                std::process::exit(0);
            }
            Short('h') => {
                println!("Usage: chip-8 PATH");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(Args {
        path: path
            .ok_or("missing argument PATH".to_string())
            .map_err(Error::msg)?,
    })
}
