use rand::{thread_rng, Rng};
use std::{fs::read, ops::Add, path::Path};

use crate::renderer::{GRID_X_SIZE, GRID_Y_SIZE};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    ClearScreen,
    Jump(u16),
    SetRegister(usize, u8),
    AddToRegister(usize, u8),
    Draw(usize, usize, u16),
    SetIndex(u16),
    CallSubroutine(u16),
    ReturnFromSubroutine,
    Skip,
    Add(usize, usize),
    Subtract(usize, usize),
    ShiftRight(usize),
    ShiftLeft(usize),
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
        self.skip();
        instruction
    }

    pub fn decode(&mut self, instruction: u16) -> OpCode {
        let kind = (instruction & 0xF000) >> 12;
        let x = ((instruction & 0x0F00) >> 8) as usize;
        let y = ((instruction & 0x00F0) >> 4) as usize;
        let n = instruction & 0x000F;
        let nn = (instruction & 0x00FF) as u8;
        let nnn = instruction & 0x0FFF;

        match (kind, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => OpCode::ClearScreen,
            (0x1, _, _, _) => OpCode::Jump(nnn),
            (0x0, 0x0, 0xE, 0xE) => OpCode::ReturnFromSubroutine,
            (0x2, _, _, _) => OpCode::CallSubroutine(nnn),
            (0x3, x, _, _) => {
                if self.get_register(x).unwrap() == nn {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x4, x, _, _) => {
                if self.get_register(x).unwrap() != nn {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x5, x, y, 0x0) => {
                if self.get_register(x).unwrap() == self.get_register(y).unwrap() {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x9, x, y, 0x0) => {
                if self.get_register(x).unwrap() != self.get_register(y).unwrap() {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x6, x, _, _) => OpCode::SetRegister(x, nn),
            (0x7, x, _, _) => OpCode::AddToRegister(x, nn),
            (0x8, x, y, 0x0) => OpCode::SetRegister(x, y.try_into().unwrap()),
            (0x8, x, y, 0x1) => OpCode::SetRegister(x, (x | y).try_into().unwrap()),
            (0x8, x, y, 0x2) => OpCode::SetRegister(x, (x & y).try_into().unwrap()),
            (0x8, x, y, 0x3) => OpCode::SetRegister(x, (x ^ y).try_into().unwrap()),
            (0x8, x, y, 0x4) => OpCode::Add(x, y),
            (0x8, x, y, 0x5) => OpCode::Subtract(x, y),
            (0x8, x, _, 0x6) => OpCode::ShiftRight(x),
            (0x8, x, y, 0x7) => OpCode::Subtract(y, x),
            (0x8, x, _, 0xE) => OpCode::ShiftLeft(x),
            (0xA, _, _, _) => OpCode::SetIndex(nnn),
            (0xB, _, _, _) => OpCode::Jump(
                nnn.add(TryInto::<u16>::try_into(self.get_register(0).unwrap()).unwrap()),
            ),
            (0xC, x, _, _) => OpCode::SetRegister(x, thread_rng().gen::<u8>() & nn),
            (0xD, x, y, n) => OpCode::Draw(x, y, n),
            (0xE, x, 0x9, 0xE) => {
                let key = self.get_register(x).unwrap();
                if self.key_pressed(key) {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0xE, x, 0xA, 0x1) => {
                let key = self.get_register(x).unwrap();
                if !self.key_pressed(key) {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
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
            OpCode::CallSubroutine(n) => self.call_subroutine(n),
            OpCode::ReturnFromSubroutine => self.return_from_subroutine(),
            OpCode::Skip => self.skip(),
            OpCode::ToDo => todo!(),
            OpCode::NoOp => (),
            OpCode::Add(x, y) => self.add(x, y),
            OpCode::Subtract(x, y) => self.subtract(x, y),
            OpCode::ShiftRight(x) => self.shift_right(x),
            OpCode::ShiftLeft(x) => self.shift_left(x),
        };
    }

    fn add(&mut self, x: usize, y: usize) {
        let vx = self.get_register(x).unwrap();
        let vy = self.get_register(y).unwrap();
        match vx.checked_add(vy) {
            None => self.set_carry(1),
            _ => {}
        };
        self.set_register(x, vx.wrapping_add(vy).try_into().unwrap())
            .unwrap();
    }

    fn subtract(&mut self, x: usize, y: usize) {
        let vx = self.get_register(x).unwrap();
        let vy = self.get_register(y).unwrap();
        if vx > vy {
            self.set_carry(1)
        } else {
            self.set_carry(0)
        }
        self.set_register(x, vx.wrapping_sub(vy).try_into().unwrap())
            .unwrap();
    }

    fn shift_right(&mut self, x: usize) {
        // Set VF to 1 if the bit that was shifted out was 1, or 0 if it was 0
        let shifted_bit = (x & 0b0001) >> 3;
        self.set_carry(shifted_bit.try_into().unwrap());
        // Shift the value of VX one bit to the right
        let vx = self.get_register(x).unwrap();
        self.set_register(x, vx >> 1).unwrap();
    }

    fn shift_left(&mut self, x: usize) {
        // Set VF to 1 if the bit that was shifted out was 1, or 0 if it was 0
        let shifted_bit = (x & 0b1000) >> 3;
        self.set_carry(shifted_bit.try_into().unwrap());
        // Shift the value of VX one bit to the left
        let vx = self.get_register(x).unwrap();
        self.set_register(x, vx << 1).unwrap();
    }

    fn skip(&mut self) {
        self.pc += 2
    }

    fn get_index(&self) -> u16 {
        self.i
    }

    fn set_index(&mut self, value: u16) {
        self.i = value
    }

    fn set_pc(&mut self, value: u16) {
        self.pc = value
    }

    fn return_from_subroutine(&mut self) {
        let last_address = self.stack.pop().unwrap();
        self.set_pc(last_address)
    }

    fn call_subroutine(&mut self, address: u16) {
        let current_pc = self.pc;
        self.stack.push(current_pc);
        self.set_pc(address)
    }

    fn key_pressed(&self, key: u8) -> bool {
        key.is_power_of_two()
    }

    fn get_register(&mut self, register: usize) -> Result<u8, String> {
        if register > 15 {
            return Err(format!("register out of bounds - {}", register));
        }
        Ok(self.registers[register])
    }

    fn set_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if register > 15 {
            return Err(format!("register out of bounds - {}", register));
        }
        self.registers[register] = value;
        Ok(())
    }

    fn set_carry(&mut self, value: u8) {
        self.registers[0xF] = value;
    }

    fn add_to_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if register > 15 {
            return Err(format!("register out of bounds - {}", register));
        }
        let _ = self.registers[register].wrapping_add(value);
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
        let mut x_coord = self.get_register(x).unwrap() % 64;
        let mut y_coord = self.get_register(y).unwrap() % 32;
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
            x_coord = self.get_register(x).unwrap() % 64;

            // Increment Y (VY is not incremented)
            y_coord += 1;
            // Stop if you reach the bottom edge of the screen
            if y_coord == 31 {
                break;
            }
        }
    }
}
