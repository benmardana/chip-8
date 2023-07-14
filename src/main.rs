mod audio;
mod cpu;
mod renderer;
extern crate sdl2;

use anyhow::{Error, Result};
use audio::AudioPlayer;
use cpu::{Cpu, OpCode};
use lexopt::Arg::{Long, Short, Value};
use lexopt::{Parser, ValueExt};
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

fn main() -> Result<()> {
    let mut cpu = Cpu::new().load(parse_args()?.path);

    let sdl_context = sdl2::init().map_err(Error::msg)?;
    let mut renderer = Renderer::new(&sdl_context).map_err(Error::msg)?;
    let audio_player = AudioPlayer::new(&sdl_context).map_err(Error::msg)?;

    let mut event_pump = renderer.event_pump();

    let mut cycle: usize = 0;
    'running: loop {
        let start = SystemTime::now();

        cycle += 1;
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

        if let Some(x) = cpu.awaiting_key {
            if let Some(key) = cpu.some_key_pressed(&event_pump) {
                cpu.set_register(x, key);
            }
        } else {
            let instruction = cpu.fetch();
            let opcode = cpu.decode(instruction);
            cpu.execute(opcode, &event_pump);
            if matches!(opcode, OpCode::Draw(..)) {
                renderer.draw_screen(cpu.screen);
            }
        }

        if cycle == 10 {
            println!("{:#?}", SystemTime::now());
            cpu.drop_timers();
            if cpu.should_beep() {
                audio_player.beep();
            } else {
                audio_player.stop_beep();
            }
            cycle = 0;
        }

        sleep(Duration::from_secs_f64(1.0 / 600.0).saturating_sub(start.elapsed()?));
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
