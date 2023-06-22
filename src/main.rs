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

    let mut screen: [[u8; 64]; 32] = [[0; GRID_X_SIZE as usize]; GRID_Y_SIZE as usize];

    let mut chip_8 = CPU::new();
    let program = read("./src/ibm.ch8").unwrap();
    program.iter().enumerate().for_each(|(i, &x)| {
        chip_8.set_mem(0x200 + i, x.into());
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

        let instruction = chip_8.fetch();
        let opcode = chip_8.decode(instruction);

        match opcode {
            cpu::OpCode::ClearScreen => {
                screen.iter_mut().for_each(|row| {
                    row.iter_mut().for_each(|pixel| {
                        *pixel = 0;
                    })
                });
            }
            cpu::OpCode::Jump(n) => chip_8.set_pc(n),
            cpu::OpCode::SetRegister(x, n) => chip_8.set_register(x, n).unwrap(),
            cpu::OpCode::AddToRegister(x, n) => chip_8.add_to_register(x, n).unwrap(),
            cpu::OpCode::Draw(x, y, n) => {
                let mut x_coord = chip_8.get_register(x) % 64;
                let mut y_coord = chip_8.get_register(y) % 32;
                let _ = chip_8.set_register(0xF, 0);
                for sprite_row in 0..n {
                    let sprite_index = (chip_8.get_index() + sprite_row) as usize;

                    // get nth sprite counting from memory address in I
                    let sprite_byte = chip_8.get_mem(sprite_index) as u8;

                    // For each of the 8 pixels/bits in this sprite row (from left to right, ie. from most to least significant bit):
                    for bit in 0..8 {
                        let sprite_pixel = sprite_byte & (0x80u8 >> bit);
                        let screen_pixel = &mut screen[y_coord as usize][x_coord as usize];

                        // If the current pixel in the sprite row is on and the pixel at coordinates X,Y on the screen is also on, turn off the pixel and set VF to 1
                        if sprite_pixel != 0 {
                            if *screen_pixel == 1 {
                                *screen_pixel = 0;
                                let _ = chip_8.set_register(0xF, 1);
                            } else {
                                // Or if the current pixel in the sprite row is on and the screen pixel is not, draw the pixel at the X and Y coordinates
                                *screen_pixel = 1;
                            }
                        }
                        // If you reach the right edge of the screen, stop drawing this row
                        if x_coord == 63 {
                            break;
                        }
                        // Increment X (VX is not incremented)
                        x_coord += 1;
                        // END FOR
                    }
                    x_coord = chip_8.get_register(x) % 64;

                    // Increment Y (VY is not incremented)
                    y_coord += 1;
                    // Stop if you reach the bottom edge of the screen
                    if y_coord == 31 {
                        break;
                    }
                }
            }
            cpu::OpCode::SetIndex(n) => chip_8.set_index(n),
            cpu::OpCode::None => (),
        };

        renderer.draw_background();
        screen.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, pixel)| {
                if *pixel != 0 {
                    renderer.draw_pixel(x.try_into().unwrap(), y.try_into().unwrap());
                }
            })
        });
        renderer.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 700));
    }
}
