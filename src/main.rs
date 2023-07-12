mod audio;
mod cpu;
mod renderer;
extern crate sdl2;

use audio::AudioPlayer;
use cpu::CPU;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let path = std::env::args().nth(1).expect("no ch8 file provided");

    let mut cpu = CPU::new();
    cpu.load_program(&std::path::PathBuf::from(path));

    let sdl_context = sdl2::init().unwrap();
    let mut renderer = Renderer::new(&sdl_context).unwrap();
    let audio_player = AudioPlayer::new(&sdl_context).unwrap();

    let mut cycle: usize = 0;

    let mut event_pump = renderer.event_pump();

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
                cpu.set_register(x, key).unwrap();
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
        sleep(Duration::new(0, 2_000_000 as u32))
    }
}
