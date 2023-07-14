use rand::{thread_rng, Rng};
use sdl2::{keyboard::Scancode, EventPump};
use std::{fs::read, ops::Add};

use crate::renderer::{GRID_X_SIZE, GRID_Y_SIZE};

#[derive(Copy, Clone, Debug, PartialEq)]
enum OpCode {
    ClearScreen,
    Jump(u16),
    SetRegister(usize, u8),
    AddToRegister(usize, u8),
    Draw(usize, usize, u16),
    SetIndex(u16),
    CallSubroutine(u16),
    ReturnFromSubroutine,
    Skip,
    SkipIfKey(usize, bool),
    Add(usize, usize),
    Subtract(usize, usize, usize),
    ShiftRight(usize),
    ShiftLeft(usize),
    SetDelayTimer(u8),
    SetSoundTimer(u8),
    GetKey(usize),
    BinaryConversion(usize),
    StoreMemory(usize),
    LoadMemory(usize),
    NoOp,
}

#[derive(Debug)]
pub struct Cpu {
    pc: u16,
    i: u16,
    memory: [u16; 4096],
    stack: Vec<u16>,
    registers: [u8; 16],
    delay_timer: u8,
    sound_timer: u8,
    awaiting_key_register: Option<usize>,
    key_down: Option<u8>,
    screen_mutated: bool,
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

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Cpu {
            pc: 0x200,
            i: 0x0,
            memory: [0u16; 4096],
            stack: Vec::new(),
            registers: [0u8; 16],
            delay_timer: 0u8,
            sound_timer: 0u8,
            awaiting_key_register: None,
            key_down: None,
            screen_mutated: false,
            screen: [[0; GRID_X_SIZE as usize]; GRID_Y_SIZE as usize],
        };
        FONT.iter().enumerate().for_each(|(i, &x)| {
            cpu.memory[i] = x;
        });
        cpu
    }
}

impl Cpu {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load(mut self, path: String) -> Cpu {
        let bytes = read(std::path::PathBuf::from(path)).unwrap();
        bytes.iter().enumerate().for_each(|(i, &x)| {
            self.memory[0x200 + i] = x.into();
        });
        self
    }

    pub fn tick(&mut self, event_pump: &EventPump) {
        if let Some(register) = self.awaiting_key_register {
            let keyboard_state = event_pump.keyboard_state();
            if let Some(key) = keyboard_state.pressed_scancodes().next() {
                self.key_down = Some(self.unmap(key));
                return;
            }
            if let Some(key_down) = self.key_down {
                if !keyboard_state
                    .pressed_scancodes()
                    .any(|scancode| scancode == self.map(key_down))
                {
                    self.set_register(register, key_down);
                    self.awaiting_key_register = None;
                    self.key_down = None;
                }
            }
        } else {
            let instruction = self.fetch();
            let opcode = self.decode(instruction);
            self.execute(opcode, event_pump);

            match opcode {
                OpCode::ClearScreen | OpCode::Draw(..) => self.screen_mutated = true,
                _ => self.screen_mutated = false,
            }
        }
    }

    pub fn tick_timers(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn should_draw(&self) -> bool {
        self.screen_mutated
    }

    fn fetch(&mut self) -> u16 {
        let instruction = self.read_current_instruction();
        self.skip();
        instruction
    }

    fn decode(&mut self, instruction: u16) -> OpCode {
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
                if self.get_register(x) == nn {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x4, x, _, _) => {
                if self.get_register(x) != nn {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x5, x, y, 0x0) => {
                if self.get_register(x) == self.get_register(y) {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x9, x, y, 0x0) => {
                if self.get_register(x) != self.get_register(y) {
                    return OpCode::Skip;
                }
                OpCode::NoOp
            }
            (0x6, x, _, _) => OpCode::SetRegister(x, nn),
            (0x7, x, _, _) => OpCode::AddToRegister(x, nn),
            (0x8, x, y, 0x0) => OpCode::SetRegister(x, self.get_register(y)),
            (0x8, x, y, 0x1) => OpCode::SetRegister(
                x,
                (usize::from(self.get_register(x)) | usize::from(self.get_register(y)))
                    .try_into()
                    .unwrap(),
            ),
            (0x8, x, y, 0x2) => OpCode::SetRegister(
                x,
                (usize::from(self.get_register(x)) & usize::from(self.get_register(y)))
                    .try_into()
                    .unwrap(),
            ),
            (0x8, x, y, 0x3) => OpCode::SetRegister(
                x,
                (usize::from(self.get_register(x)) ^ usize::from(self.get_register(y)))
                    .try_into()
                    .unwrap(),
            ),
            (0x8, x, y, 0x4) => OpCode::Add(x, y),
            (0x8, x, y, 0x5) => OpCode::Subtract(x, y, x),
            (0x8, x, _, 0x6) => OpCode::ShiftRight(x),
            (0x8, x, y, 0x7) => OpCode::Subtract(y, x, x),
            (0x8, x, _, 0xE) => OpCode::ShiftLeft(x),
            (0xA, _, _, _) => OpCode::SetIndex(nnn),
            (0xB, _, _, _) => OpCode::Jump(nnn.add(u16::from(self.get_register(0)))),
            (0xC, x, _, _) => OpCode::SetRegister(x, thread_rng().gen::<u8>() & nn),
            (0xD, x, y, n) => OpCode::Draw(x, y, n),
            (0xE, x, 0x9, 0xE) => OpCode::SkipIfKey(x, true),
            (0xE, x, 0xA, 0x1) => OpCode::SkipIfKey(x, false),
            (0xF, x, 0x0, 0x7) => OpCode::SetRegister(x, self.delay_timer),
            (0xF, x, 0x1, 0x5) => OpCode::SetDelayTimer(self.get_register(x)),
            (0xF, x, 0x1, 0x8) => OpCode::SetSoundTimer(self.get_register(x)),
            (0xF, x, 0x1, 0xE) => {
                OpCode::SetIndex(self.get_index().add(u16::from(self.get_register(x))))
            }
            (0xF, x, 0x0, 0xA) => OpCode::GetKey(x),
            (0xF, x, 0x2, 0x9) => OpCode::SetIndex(self.get_register(x).into()),
            (0xF, x, 0x3, 0x3) => OpCode::BinaryConversion(x),
            (0xF, x, 0x5, 0x5) => OpCode::StoreMemory(x),
            (0xF, x, 0x6, 0x5) => OpCode::LoadMemory(x),
            (0x0, _, _, _) => OpCode::NoOp,
            _ => OpCode::NoOp,
        }
    }

    fn execute(&mut self, opcode: OpCode, event_pump: &EventPump) {
        match opcode {
            OpCode::ClearScreen => self.clear_screen(),
            OpCode::Jump(n) => self.set_pc(n),
            OpCode::SetRegister(x, n) => self.set_register(x, n),
            OpCode::AddToRegister(x, n) => self.add_to_register(x, n),
            OpCode::Draw(x, y, n) => self.update_screen(x, y, n),
            OpCode::SetIndex(n) => self.set_index(n),
            OpCode::CallSubroutine(n) => self.call_subroutine(n),
            OpCode::ReturnFromSubroutine => self.return_from_subroutine(),
            OpCode::Skip => self.skip(),
            OpCode::SkipIfKey(x, pressed) => {
                let key = self.get_register(x);
                self.skip_if_key(key, pressed, event_pump);
            }
            OpCode::NoOp => (),
            OpCode::Add(x, y) => self.add(x, y),
            OpCode::Subtract(x, y, n) => self.subtract(x, y, n),
            OpCode::ShiftRight(x) => self.shift_right(x),
            OpCode::ShiftLeft(x) => self.shift_left(x),
            OpCode::SetDelayTimer(x) => self.set_delay_timer(x),
            OpCode::SetSoundTimer(x) => self.set_sound_timer(x),
            OpCode::GetKey(key) => self.set_waiting_key(Some(key)),
            OpCode::BinaryConversion(x) => self.binary_conversion(x),
            OpCode::StoreMemory(x) => self.store_memory(x),
            OpCode::LoadMemory(x) => self.load_memory(x),
        };
    }

    fn skip_if_key(&mut self, key: u8, pressed: bool, event_pump: &EventPump) -> bool {
        let key_pressed = event_pump
            .keyboard_state()
            .is_scancode_pressed(self.map(key));
        if pressed == key_pressed {
            self.skip();
        }
        pressed == key_pressed
    }

    fn store_memory(&mut self, register: usize) {
        let index = self.get_index();
        for x in 0..register + 1 {
            let vx = u16::from(self.get_register(x));
            self.memory[usize::from(index) + x] = vx;
        }
    }

    fn load_memory(&mut self, register: usize) {
        let index = self.get_index();
        for x in 0..register + 1 {
            let value: u8 = self.memory[usize::from(index) + x].try_into().unwrap();
            self.set_register(x, value);
        }
    }

    fn binary_conversion(&mut self, register: usize) {
        let value = self.get_register(register);
        let (ones, tens, hundreds) = (value % 10, (value / 10) % 10, (value / 10) / 10);

        let index = self.get_index();
        self.memory[usize::from(index)] = hundreds.into();
        self.memory[usize::from(index + 1)] = tens.into();
        self.memory[usize::from(index + 2)] = ones.into();
    }

    fn set_waiting_key(&mut self, register: Option<usize>) {
        self.awaiting_key_register = register
    }

    fn set_delay_timer(&mut self, val: u8) {
        self.delay_timer = val;
    }

    fn set_sound_timer(&mut self, val: u8) {
        self.sound_timer = val;
    }

    fn add(&mut self, x: usize, y: usize) {
        let vx = self.get_register(x);
        let vy = self.get_register(y);
        if vx.checked_add(vy).is_none() {
            self.set_carry(1)
        };
        self.set_register(x, vx.wrapping_add(vy));
    }

    fn subtract(&mut self, x: usize, y: usize, destination: usize) {
        let vx = self.get_register(x);
        let vy = self.get_register(y);
        if vx > vy {
            self.set_carry(1)
        } else {
            self.set_carry(0)
        }
        self.set_register(destination, vx.wrapping_sub(vy));
    }

    fn shift_right(&mut self, x: usize) {
        // Set VF to 1 if the bit that was shifted out was 1, or 0 if it was 0
        let shifted_bit = (x & 0b0001) >> 3;
        self.set_carry(shifted_bit.try_into().unwrap());
        // Shift the value of VX one bit to the right
        let vx = self.get_register(x);
        self.set_register(x, vx >> 1);
    }

    fn shift_left(&mut self, x: usize) {
        // Set VF to 1 if the bit that was shifted out was 1, or 0 if it was 0
        let shifted_bit = (x & 0b1000) >> 3;
        self.set_carry(shifted_bit.try_into().unwrap());
        // Shift the value of VX one bit to the left
        let vx = self.get_register(x);
        self.set_register(x, vx << 1);
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

    fn get_register(&mut self, register: usize) -> u8 {
        self.registers[register]
    }

    fn set_register(&mut self, register: usize, value: u8) {
        self.registers[register] = value;
    }

    fn set_carry(&mut self, value: u8) {
        self.registers[0xF] = value;
    }

    fn add_to_register(&mut self, register: usize, value: u8) {
        self.set_register(register, self.registers[register].wrapping_add(value));
    }

    fn read_current_instruction(&mut self) -> u16 {
        let part_one = self.memory[self.pc as usize];
        let part_two = self.memory[(self.pc + 1) as usize];
        (part_one << 8) | part_two
    }

    fn clear_screen(&mut self) {
        self.screen.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|pixel| {
                *pixel = 0;
            })
        });
    }

    fn update_screen(&mut self, x: usize, y: usize, n: u16) {
        let mut x_coord = self.get_register(x) % 64;
        let mut y_coord = self.get_register(y) % 32;
        self.set_register(0xF, 0);
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
                        self.set_register(0xF, 1);
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

    fn map(&self, code: u8) -> Scancode {
        match code {
            0x00 => Scancode::X,
            0x01 => Scancode::Num1,
            0x02 => Scancode::Num2,
            0x03 => Scancode::Num3,
            0x04 => Scancode::Q,
            0x05 => Scancode::W,
            0x06 => Scancode::E,
            0x07 => Scancode::A,
            0x08 => Scancode::S,
            0x09 => Scancode::D,
            0x0A => Scancode::Z,
            0x0B => Scancode::C,
            0x0C => Scancode::Num4,
            0x0D => Scancode::R,
            0x0E => Scancode::F,
            0x0F => Scancode::V,
            _ => Scancode::Escape,
        }
    }

    fn unmap(&self, code: Scancode) -> u8 {
        match code {
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
            Scancode::Escape => 0x29,
            _ => 0x29,
        }
    }
}
