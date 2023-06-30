mod cpu;
mod renderer;
extern crate sdl2;

use cpu::CPU;
use renderer::{Renderer, DOT_SIZE_IN_PXS, GRID_X_SIZE, GRID_Y_SIZE};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::read;
use std::time::Duration;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "chip-8",
            GRID_X_SIZE * DOT_SIZE_IN_PXS,
            GRID_Y_SIZE * DOT_SIZE_IN_PXS,
        )
        .position_centered()
        .opengl()
        .build()
        .expect("unable to create window");

    let mut renderer = Renderer::new(window).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut cpu = CPU::new();
    let program = read("./src/ibm.ch8").unwrap();
    program.iter().enumerate().for_each(|(i, &x)| {
        cpu.set_mem(0x200 + i, x.into());
    });

    'running: loop {
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

        let instruction = cpu.fetch();
        let opcode = cpu.decode(instruction);

        match opcode {
            cpu::OpCode::ClearScreen => renderer.clear_screen(),
            cpu::OpCode::Jump(n) => cpu.set_pc(n),
            cpu::OpCode::SetRegister(x, n) => cpu.set_register(x, n).unwrap(),
            cpu::OpCode::AddToRegister(x, n) => cpu.add_to_register(x, n).unwrap(),
            cpu::OpCode::Draw(x, y, n) => cpu.update_screen(x, y, n),
            cpu::OpCode::SetIndex(n) => cpu.set_index(n),
            cpu::OpCode::None => (),
        };

        renderer.draw_screen(cpu.screen);
        renderer.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 700));
    }
}
