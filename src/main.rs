mod cpu;
mod renderer;
extern crate sdl2;

use cpu::CPU;
use renderer::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() {
    let path = std::env::args().nth(1).expect("no ch8 file provided");

    let mut cpu = CPU::new();
    cpu.load_program(&std::path::PathBuf::from(path));

    let mut renderer = Renderer::new().unwrap();

    'running: loop {
        for event in renderer.event_pump().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let instruction = cpu.fetch();
        let opcode = cpu.decode(instruction);
        cpu.execute(opcode);

        renderer.draw_screen(cpu.screen);
        sleep();
    }
}

fn sleep() {
    ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 700));
}
