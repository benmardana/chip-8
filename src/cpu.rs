#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    ClearScreen,
    Jump(u16),
    SetRegister(usize, u8),
    AddToRegister(usize, u8),
    Draw(usize, usize, u16),
    SetIndex(u16),
    None,
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

    pub fn set_mem(&mut self, index: usize, value: u16) {
        self.memory[index] = value;
    }

    pub fn get_mem(&self, index: usize) -> u16 {
        let part_one = self.memory[index as usize];
        let _part_two = self.memory[(index + 1) as usize];
        return part_one;
    }

    pub fn get_index(&self) -> u16 {
        self.i
    }

    pub fn set_index(&mut self, value: u16) {
        self.i = value;
    }

    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub fn get_register(&mut self, register: usize) -> u8 {
        self.registers[register]
    }

    pub fn set_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if value > 15 {
            return Err("register out of bounds".to_string());
        }
        self.registers[register] = value;
        Ok(())
    }

    pub fn add_to_register(&mut self, register: usize, value: u8) -> Result<(), String> {
        if value > 15 {
            return Err("register out of bounds".to_string());
        }
        self.registers[register] += value;
        Ok(())
    }

    pub fn fetch(&mut self) -> u16 {
        let instruction = self.read_current_instruction();
        self.pc += 2;
        instruction
    }

    fn read_current_instruction(&mut self) -> u16 {
        let part_one = self.memory[self.pc as usize];
        let part_two = self.memory[(self.pc + 1) as usize];
        return (part_one << 8) | part_two as u16;
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
            (0x1, _, _, _) => OpCode::Jump(nnn),
            (0x6, x, _, _) => OpCode::SetRegister(x, nn),
            (0x7, x, _, _) => OpCode::AddToRegister(x, nn),
            (0xA, _, _, _) => OpCode::SetIndex(nnn),
            (0xD, x, y, n) => OpCode::Draw(x, y, n),
            _ => OpCode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_decode() {
        let mut chip_8 = CPU::new();
        chip_8.set_mem(0xA0, 0x00E0);
        chip_8.set_mem(0xA1, 0x1123);
        chip_8.set_mem(0xA2, 0x6123);
        chip_8.set_mem(0xA3, 0x7123);
        chip_8.set_mem(0xA4, 0xA123);
        chip_8.set_mem(0xA5, 0xD123);
        chip_8.set_mem(0xA6, 0xC123);

        let mut instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::ClearScreen);

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::Jump(291));

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::SetRegister(1, 35));

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::AddToRegister(1, 35));

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::SetIndex(291));

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::Draw(1, 2, 3));

        instruction = chip_8.fetch();
        assert_eq!(chip_8.decode(instruction), OpCode::None);

        return;
    }
}
