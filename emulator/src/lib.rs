use std::path::PathBuf;
use std::time;
use std::fs;

use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

mod instruction;
mod display;

use crate::instruction::Instruction;
use crate::display::Display;

pub struct Chip8 {
    display: Display,
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    last_clock: time::Instant,
    events: Vec<u8>,
}

impl Chip8 {
    const KEYMAP: [Keycode; 16] = [
        Keycode::Num7, Keycode::Num8, Keycode::Num9,    // 1-3
        Keycode::U, Keycode::I, Keycode::O,             // 4-6
        Keycode::J, Keycode::K, Keycode::L,             // 7-9
        Keycode::Comma, Keycode::M, Keycode::Period,    // 0-B
        Keycode::Num0, Keycode::P, Keycode::Semicolon,  // C-E
        Keycode::Slash,                                 // F
    ];

    pub fn new(canvas: Option<Canvas<Window>>) -> Self {
        let mut res = Self{
            display: Display::new(canvas),
            memory: [0; 4096],
            registers: Default::default(),
            i: Default::default(),
            delay_timer: Default::default(),
            sound_timer: Default::default(),
            pc: Default::default(),
            sp: Default::default(),
            stack: Default::default(),
            last_clock: time::Instant::now(),
            events: Default::default(),
        };

        // Add digit sprites to memory
        res.memory[0..16 * 5].copy_from_slice(&[
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
        ]);

        res
    }

    pub fn load(&mut self, rom_path: PathBuf) -> std::io::Result<()> {
        let contents = fs::read(rom_path)?;

        self.memory[0x200..0x200 + contents.len()].copy_from_slice(contents.as_slice());

        Ok(())
    }

    pub fn tick(&mut self) {
        let raw_instruction = self.fetch();
        match Self::decode(raw_instruction) {
            Some(instr) => self.execute(instr),
            None => (),
        }
        self.render();
        if self.last_clock.elapsed() > time::Duration::from_millis(16) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
            self.last_clock = time::Instant::now();
        }
    }

    pub fn push_key(&mut self, keycode: &Keycode) {
        if let Some(key) = Self::KEYMAP.iter().position(|&el| el == *keycode) {
            self.events.push(key as u8);
        }
    }

    fn fetch(&mut self) -> u16 {
        let mut instruction: u16 = self.memory[self.pc] as u16;
        instruction <<= 8;
        instruction |= self.memory[self.pc + 1] as u16;
        self.pc += 2;
        instruction
    }

    fn decode(raw: u16) -> Option<Instruction> {
        match Instruction::try_from(raw) {
            Ok(instr) => Some(instr),
            Err(_) => None,
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        use Instruction::*;

        match instruction {
            ClearDisplay => self.display.clear(),
            Jump(addr) => self.pc = addr as usize,
            ReturnFromSubroutine => {
                self.pc = self.stack[self.sp] as usize;
                self.sp -= 1;
            }
            CallSubroutine(addr) => {
                self.sp += 1;
                self.stack[self.sp] = self.pc as u16;
                self.pc = addr as usize;
            }
            SkipRegEqByte(reg, val) => {
                if self.registers[reg as usize] == val {
                    self.pc += 2;
                }
            }
            SkipRegNotEqByte(reg, val) => {
                if self.registers[reg as usize] != val {
                    self.pc += 2;
                }
            }
            SkipRegEqReg(reg1, reg2) => {
                if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
                    self.pc += 2;
                }
            }
            SetRegToByte(reg, val) => self.registers[reg as usize] = val,
            AddByteToReg(reg, val) => self.registers[reg as usize] = self.registers[reg as usize].wrapping_add(val),
            MoveValue(reg1, reg2) => self.registers[reg1 as usize] = self.registers[reg2 as usize],
            OrRegs(reg1, reg2) => self.registers[reg1 as usize] |= self.registers[reg2 as usize],
            AndRegs(reg1, reg2) => self.registers[reg1 as usize] &= self.registers[reg2 as usize],
            XorRegs(reg1, reg2) => self.registers[reg1 as usize] ^= self.registers[reg2 as usize],
            AddRegs(reg1, reg2) => {
                let res =
                    self.registers[reg1 as usize] as u16 + self.registers[reg2 as usize] as u16;
                self.registers[0xF] = if res > 0xFF { 1 } else { 0 }; // carry
                self.registers[reg1 as usize] = (res & 0xFF) as u8;
            }
            SubRegs(reg1, reg2) => {
                let val1 = self.registers[reg1 as usize];
                let val2 = self.registers[reg2 as usize];
                self.registers[0xF] = if val1 > val2 { 1 } else { 0 }; // NOT borrow
                self.registers[reg1 as usize] = val1.wrapping_sub(val2);
            }
            ShiftRight(reg) => {
                let val = self.registers[reg as usize];
                self.registers[0xF] = if val % 2 == 1 { 1 } else { 0 }; // data loss
                self.registers[reg as usize] = val >> 1;
            }
            ReverseSubRegs(reg1, reg2) => {
                let val1 = self.registers[reg1 as usize];
                let val2 = self.registers[reg2 as usize];
                self.registers[0xF] = if val2 > val1 { 1 } else { 0 }; // NOT borrow
                self.registers[reg1 as usize] = val2.wrapping_sub(val1);
            }
            ShiftLeft(reg) => {
                let val = self.registers[reg as usize];
                self.registers[0xF] = if val & 0x80 != 0 { 1 } else { 0 }; // data loss
                self.registers[reg as usize] = val << 1;
            }
            SkipRegNotEqReg(reg1, reg2) => {
                if self.registers[reg1 as usize] != self.registers[reg2 as usize] {
                    self.pc += 2;
                }
            }
            SetI(addr) => self.i = addr,
            JumpV0PlusByte(addr) => self.pc = self.registers[0x0] as usize + addr as usize,
            SetRegToRandPlusByte(reg, val) => {
                self.registers[reg as usize] = rand::random::<u8>() & val;
            }
            DrawSprite(reg_x, reg_y, size) => {
                let x = self.registers[reg_x as usize];
                let y = self.registers[reg_y as usize];
                let sprite = &self.memory[self.i as usize..(self.i as usize + size as usize)];
                self.registers[0xF] = if self.display.blit_sprite(x, y, sprite) { 1 } else { 0 }
            },
            SkipIfKey(reg) => {
                let key_idx = self.registers[reg as usize];
                assert!((key_idx as usize) < Self::KEYMAP.len());
                if let Some(key) = self.events.pop() {
                    self.pc += if key == key_idx { 2 } else { 0 };
                }
            },
            SkipIfNotKey(reg) => {
                let key_idx = self.registers[reg as usize];
                assert!((key_idx as usize) < Self::KEYMAP.len());
                if let Some(key) = self.events.pop() {
                    self.pc += if key != key_idx { 2 } else { 0 };
                }
            },
            LoadDelayToReg(reg) => self.registers[reg as usize] = self.delay_timer,
            LoadKeyToReg(reg) => {
                loop {
                    if let Some(key) = self.events.pop() {
                        self.registers[reg as usize] = key as u8;
                        break;
                    }
                }
            },
            SetDelayToReg(reg) => self.delay_timer = self.registers[reg as usize],
            SetSoundToReg(reg) => self.sound_timer = self.registers[reg as usize],
            AddRegToI(reg) => self.i += self.registers[reg as usize] as u16,
            SetIToDigitSpriteLoc(reg) => {
                self.i = self.registers[reg as usize] as u16 * 5;
            },
            StoreNumberFromRegToI(reg) => {
                let val = self.registers[reg as usize];
                let hundreds = val / 100;
                let tens = (val - hundreds * 100) / 10;
                let ones = val - hundreds * 100 - tens * 10;

                self.memory[self.i as usize] = hundreds;
                self.memory[self.i as usize + 1] = tens;
                self.memory[self.i as usize + 2] = ones;
            },
            StoreRegsToMem(max_reg) => {
                for (reg, val) in self.registers[0..=(max_reg as usize)].iter().enumerate() {
                    self.memory[self.i as usize + reg] = *val;
                }
            },
            LoadRegsFromMem(max_reg) => {
                for (reg, val) in self.registers[0..=(max_reg as usize)]
                    .iter_mut()
                    .enumerate()
                {
                    *val = self.memory[self.i as usize + reg] as u8;
                }
            },
        }
    }

    fn render(&mut self) {
        self.display.render();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fx29() {
        let mut emu = Chip8::new(None);
        emu.registers[0x0] = 213;
        emu.i = 0x200;

        emu.execute(Instruction::StoreNumberFromRegToI(0x0));

        assert_eq!(emu.memory[0x200], 2);
        assert_eq!(emu.memory[0x201], 1);
        assert_eq!(emu.memory[0x202], 3);
    }
}
