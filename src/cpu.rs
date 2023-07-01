use std::{fs::read, path::Path};

use crate::renderer::{GRID_X_SIZE, GRID_Y_SIZE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    ClearScreen,
    Jump(u16),
    SetRegister(usize, u8),
    AddToRegister(usize, u8),
    Draw(usize, usize, u16),
    SetIndex(u16),
    ToDo,
    NoOp,
}

#[derive(Debug)]
pub struct CPU {
    pc: u16,
    i: u16,
    memory: [u16; 4096],
    stack: Vec<u16>,
    registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    pub screen: [[u8; 64]; 32],
}

const FONT: [u16; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Default for CPU {
    fn default() -> Self {
        let mut cpu = CPU {
            pc: 0x200,
            i: 0x0,
            memory: [0u16; 4096],
            stack: Vec::new(),
            registers: [0u8; 16],
            delay_timer: u8::MAX,
            sound_timer: u8::MAX,
            screen: [[0; GRID_X_SIZE as usize]; GRID_Y_SIZE as usize],
        };
        FONT.iter().enumerate().for_each(|(i, &x)| {
            cpu.memory[i] = x;
        });
        cpu
    }
}

impl CPU {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_program(&mut self, path: &Path) {
        let bytes = read(path).unwrap();
        bytes.iter().enumerate().for_each(|(i, &x)| {
            self.memory[0x200 + i] = x.into();
        });
    }

    pub fn fetch(&mut self) -> u16 {
        let instruction = self.read_current_instruction();
        self.pc += 2;
        instruction
    }

    pub fn decode(&self, instruction: u16) -> OpCode {
        let kind = (instruction & 0xF000) >> 12;
        let x = ((instruction & 0x0F00) >> 8) as usize;
        let y = ((instruction & 0x00F0) >> 4) as usize;
        let n = instruction & 0x000F;
        let nn = (instruction & 0x00FF) as u8;
        let nnn = instruction & 0x0FFF;

        match (kind, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => OpCode::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => OpCode::ToDo,
            (0x1, _, _, _) => OpCode::Jump(nnn),
            (0x2, _, _, _) => OpCode::ToDo,
            (0x3, x, _, _) => OpCode::ToDo,
            (0x4, x, _, _) => OpCode::ToDo,
            (0x5, x, _, _) => OpCode::ToDo,
            (0x6, x, _, _) => OpCode::SetRegister(x, nn),
            (0x7, x, _, _) => OpCode::AddToRegister(x, nn),
            (0x8, x, y, 0x0) => OpCode::ToDo,
            (0x8, x, y, 0x1) => OpCode::ToDo,
            (0x8, x, y, 0x2) => OpCode::ToDo,
            (0x8, x, y, 0x3) => OpCode::ToDo,
            (0x8, x, y, 0x4) => OpCode::ToDo,
            (0x8, x, y, 0x5) => OpCode::ToDo,
            (0x8, x, y, 0x6) => OpCode::ToDo,
            (0x8, x, y, 0x7) => OpCode::ToDo,
            (0x8, x, y, 0x8) => OpCode::ToDo,
            (0x9, x, _, _) => OpCode::ToDo,
            (0xA, _, _, _) => OpCode::SetIndex(nnn),
            (0xB, _, _, _) => OpCode::ToDo,
            (0xC, x, _, _) => OpCode::ToDo,
            (0xD, x, y, n) => OpCode::Draw(x, y, n),
            (0xE, x, 0x9, 0xE) => OpCode::ToDo,
            (0xE, x, 0xA, 0x1) => OpCode::ToDo,
            (0xF, x, 0x0, 0x7) => OpCode::ToDo,
            (0xF, x, 0x1, 0x5) => OpCode::ToDo,
            (0xF, x, 0x1, 0x8) => OpCode::ToDo,
            (0xF, x, 0x1, 0xE) => OpCode::ToDo,
            (0xF, x, 0x0, 0xA) => OpCode::ToDo,
            (0xF, x, 0x2, 0x9) => OpCode::ToDo,
            (0xF, x, 0x3, 0x3) => OpCode::ToDo,
            (0xF, x, 0x5, 0x5) => OpCode::ToDo,
            (0xF, x, 0x6, 0x5) => OpCode::ToDo,
            (0x0, _, _, _) => OpCode::NoOp,
            _ => OpCode::NoOp,
        }
    }

    pub fn execute(&mut self, opcode: OpCode) {
        match opcode {
            OpCode::ClearScreen => self.clear_screen(),
            OpCode::Jump(n) => self.set_pc(n),
            OpCode::SetRegister(x, n) => self.set_register(x, n).unwrap(),
            OpCode::AddToRegister(x, n) => self.add_to_register(x, n).unwrap(),
            OpCode::Draw(x, y, n) => self.update_screen(x, y, n),
            OpCode::SetIndex(n) => self.set_index(n),
            OpCode::ToDo => todo!(),
            OpCode::NoOp => (),
        };
    }

    fn get_index(&self) -> u16 {
        self.i
    }

    fn set_index(&mut self, value: u16) {
        self.i = value;
    }

    fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    fn get_register(&mut self, register: usize) -> u8 {
        self.registers[register]
    }

    fn set_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if value > 15 {
            return Err("register out of bounds".to_string());
        }
        self.registers[register] = value;
        Ok(())
    }

    fn add_to_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if value > 15 {
            return Err("register out of bounds".to_string());
        }
        self.registers[register] += value;
        Ok(())
    }

    fn read_current_instruction(&mut self) -> u16 {
        let part_one = self.memory[self.pc as usize];
        let part_two = self.memory[(self.pc + 1) as usize];
        return (part_one << 8) | part_two as u16;
    }

    fn clear_screen(&mut self) {
        self.screen.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|pixel| {
                *pixel = 0;
            })
        });
    }

    fn update_screen(&mut self, x: usize, y: usize, n: u16) -> () {
        let mut x_coord = self.get_register(x) % 64;
        let mut y_coord = self.get_register(y) % 32;
        let _ = self.set_register(0xF, 0);
        for sprite_row in 0..n {
            let sprite_index = (self.get_index() + sprite_row) as usize;

            // get nth sprite counting from memory address in I
            let sprite_byte = self.memory[sprite_index] as u8;

            // For each of the 8 pixels/bits in this sprite row (from left to right, ie. from most to least significant bit):
            for bit in 0..8 {
                let sprite_pixel = sprite_byte & (0x80u8 >> bit);
                let screen_pixel = &mut self.screen[y_coord as usize][x_coord as usize];

                // If the current pixel in the sprite row is on and the pixel at coordinates X,Y on the screen is also on, turn off the pixel and set VF to 1
                if sprite_pixel != 0 {
                    if *screen_pixel == 1 {
                        *screen_pixel = 0;
                        let _ = self.set_register(0xF, 1);
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
            x_coord = self.get_register(x) % 64;

            // Increment Y (VY is not incremented)
            y_coord += 1;
            // Stop if you reach the bottom edge of the screen
            if y_coord == 31 {
                break;
            }
        }
    }
}
