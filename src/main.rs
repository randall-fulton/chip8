use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, self};
use std::fs;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("usage: chip8 romfile");
        return Ok(())
    }

    let rom = match std::path::PathBuf::from_str(args[1].as_str()) {
        Ok(path) => path,
        Err(_) => {
            println!("romfile does not exist");
            return Ok(())
        }
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut emu = Chip8::new();
    emu.load(rom)?;

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => emu.push_key(&keycode),
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        emu.tick(&mut canvas);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 1000));
    }

    Ok(())
}

enum Instruction {
    ClearDisplay,
    ReturnFromSubroutine,
    Jump(u16),
    CallSubroutine(u16),
    SkipRegEqByte(u8, u8),
    SkipRegNotEqByte(u8, u8),
    SkipRegEqReg(u8, u8),
    SetRegToByte(u8, u8),
    AddByteToReg(u8, u8),
    MoveValue(u8, u8),
    OrRegs(u8, u8),
    AndRegs(u8, u8),
    XorRegs(u8, u8),
    AddRegs(u8, u8),
    SubRegs(u8, u8),
    ShiftRight(u8),
    ReverseSubRegs(u8, u8),
    ShiftLeft(u8),
    SkipRegNotEqReg(u8, u8),
    SetI(u16),
    JumpV0PlusByte(u16),
    SetRegToRandPlusByte(u8, u8),
    DrawSprite(u8, u8, u8),
    SkipIfKey(u8),
    SkipIfNotKey(u8),
    LoadDelayToReg(u8),
    LoadKeyToReg(u8),
    SetDelayToReg(u8),
    SetSoundToReg(u8),
    AddRegToI(u8),
    SetIToDigitSpriteLoc(u8),
    StoreNumberFromRegToI(u8),
    StoreRegsToMem(u8),
    LoadRegsFromMem(u8),
}

struct Chip8 {
    memory: [u8; 4096],
    registers: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: [u16; 16],
    pixels: [bool; 64 * 32],
    last_clock: time::Instant,
    events: Vec<u8>,
}

impl Chip8 {
    const ROWS: u16 = 32;
    const COLS: u16 = 64;

    const KEYMAP: [Keycode; 16] = [
        Keycode::Num6, Keycode::Num7, Keycode::Num8,    // 1-3
        Keycode::Y, Keycode::U, Keycode::I,             // 4-6
        Keycode::J, Keycode::K, Keycode::L,             // 7-9
        Keycode::Comma, Keycode::M, Keycode::Period,    // 0-B
        Keycode::Num0, Keycode::P, Keycode::Semicolon,  // C-E
        Keycode::Slash,                                 // F
    ];

    pub fn new() -> Self {
        let mut res = Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            pixels: [false; (Self::ROWS * Self::COLS) as usize],
            last_clock: time::Instant::now(),
            events: Vec::new(),
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

    pub fn load(self: &mut Self, rom_path: PathBuf) -> std::io::Result<()> {
        let contents = fs::read(rom_path)?;

        self.memory[0x200..0x200 + contents.len()].copy_from_slice(contents.as_slice());

        Ok(())
    }

    pub fn tick(self: &mut Self, canvas: &mut Canvas<Window>) {
        let raw_instruction = self.fetch();
        match Self::decode(raw_instruction) {
            Some(instr) => self.execute(instr),
            None => (),
        }
        self.render(canvas);
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

    fn fetch(self: &mut Self) -> u16 {
        let mut instruction: u16 = self.memory[self.pc] as u16;
        instruction <<= 8;
        instruction |= self.memory[self.pc + 1] as u16;
        self.pc += 2;
        instruction
    }

    fn decode(raw: u16) -> Option<Instruction> {
        use Instruction::*;
        match raw {
            0x00E0 => Some(ClearDisplay),
            0x00EE => Some(ReturnFromSubroutine),
            0x1000..=0x1FFF => Some(Jump(raw & 0x0FFF)),
            0x2000..=0x2FFF => Some(CallSubroutine(raw & 0x0FFF)),
            0x3000..=0x3FFF => Some(SkipRegEqByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x4000..=0x4FFF => Some(SkipRegNotEqByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x5000..=0x5FFF => match raw & 0xF {
                0x0 => Some(SkipRegEqReg(
                    ((raw & 0x0F00) >> 8) as u8,
                    ((raw & 0xF0) >> 4) as u8,
                )),
                _ => None,
            },
            0x6000..=0x6FFF => Some(SetRegToByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x7000..=0x7FFF => Some(AddByteToReg(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x8000..=0x8FFF => match raw & 0xF {
                0x0 => Some(MoveValue(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x1 => Some(OrRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x2 => Some(AndRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x3 => Some(XorRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x4 => Some(AddRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x5 => Some(SubRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x6 => Some(ShiftRight(((raw & 0x0F00) >> 8) as u8)),
                0x7 => Some(ReverseSubRegs(
                    ((raw & 0x0F00) >> 8) as u8,
                    (raw & 0xFF) as u8,
                )),
                0xE => Some(ShiftLeft(((raw & 0x0F00) >> 8) as u8)),
                _ => None,
            },
            0x9000..=0x9FFF => match raw & 0xF {
                0x0 => Some(SkipRegNotEqReg(
                    ((raw & 0x0F00) >> 8) as u8,
                    ((raw & 0xF0) as u8) >> 4,
                )),
                _ => None,
            },
            0xA000..=0xAFFF => Some(SetI(raw & 0x0FFF)),
            0xB000..=0xBFFF => Some(JumpV0PlusByte(raw & 0x0FFF)),
            0xC000..=0xCFFF => Some(SetRegToRandPlusByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0xD000..=0xDFFF => Some(DrawSprite(
                ((raw & 0x0F00) >> 8) as u8,
                ((raw & 0xF0) >> 4) as u8,
                (raw & 0xF) as u8,
            )),
            0xE000..=0xEFFF => match raw & 0xFF {
                0x9E => Some(SkipIfKey(((raw & 0x0F00) >> 8) as u8)),
                0xA1 => Some(SkipIfNotKey(((raw & 0x0F00) >> 8) as u8)),
                _ => None,
            },
            0xF000..=0xFFFF => match raw & 0xFF {
                0x07 => Some(LoadDelayToReg(((raw & 0x0F00) >> 8) as u8)),
                0x0A => Some(LoadKeyToReg(((raw & 0x0F00) >> 8) as u8)),
                0x15 => Some(SetDelayToReg(((raw & 0x0F00) >> 8) as u8)),
                0x18 => Some(SetSoundToReg(((raw & 0x0F00) >> 8) as u8)),
                0x1E => Some(AddRegToI(((raw & 0x0F00) >> 8) as u8)),
                0x29 => Some(SetIToDigitSpriteLoc(((raw & 0x0F00) >> 8) as u8)),
                0x33 => Some(StoreNumberFromRegToI(((raw & 0x0F00) >> 8) as u8)),
                0x55 => Some(StoreRegsToMem(((raw & 0x0F00) >> 8) as u8)),
                0x65 => Some(LoadRegsFromMem(((raw & 0x0F00) >> 8) as u8)),
                _ => None,
            },
            _ => None,
        }
    }

    fn execute(self: &mut Self, instruction: Instruction) {
        use Instruction::*;

        match instruction {
            ClearDisplay => self.pixels = [false; 64 * 32],
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
                let x = self.registers[reg_x as usize] as usize;
                let y = self.registers[reg_y as usize] as usize;

                let mut collision = false;
                for idx in 0..size {
                    let row = self.memory[self.i as usize + idx as usize];

                    let pixel_idx = x as usize + (Self::COLS * (y as u16 + idx as u16)) as usize;
                    if pixel_idx + 7 > self.pixels.len() {
                        continue
                    }

                    let existing = Self::pixels_to_byte(&self.pixels[pixel_idx..pixel_idx + 8]);

                    self.pixels[pixel_idx..pixel_idx + 8].clone_from_slice(&Self::byte_to_pixels(row^existing));

                    let collide = (row & existing) != 0;
                    collision = collision || collide;
                }
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

    fn render(self: &mut Self, canvas: &mut Canvas<Window>) {
        let (width, height) = canvas.output_size().unwrap();
        let pixel_width = width / Self::COLS as u32;
        let pixel_height = height / Self::ROWS as u32;

        for y in 0..Self::ROWS {
            for x in 0..Self::COLS {
                if self.pixels[(y * Self::COLS + x) as usize] {
                    canvas.set_draw_color(Color::WHITE);
                } else {
                    canvas.set_draw_color(Color::BLACK);
                }
                canvas.fill_rect(Rect::new(
                    x as i32 * pixel_width as i32,
                    y as i32 * pixel_height as i32,
                    pixel_width,
                    pixel_height
                )).unwrap();
            }
        }
    }

    fn push_key(self: &mut Self, keycode: &Keycode) {
        if let Some(key) = Self::KEYMAP.iter().position(|&el| el == *keycode) {
            self.events.push(key as u8);
        }
    }

    fn pixels_to_byte(pixels: &[bool]) -> u8 {
        let mut byte = 0;

        for pixel in pixels {
            byte <<= 1;
            byte += if *pixel { 1 } else { 0 }
        }

        byte
    }

    fn byte_to_pixels(byte: u8) -> [bool; 8] {
        let mut byte = byte;
        let mut pixels = [false; 8];

        for i in 0..8 {
            pixels[7 - i] = (byte & 0x1) != 0;
            byte >>= 1;
        }

        pixels
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fx29() {
        let mut emu = Chip8::new();
        emu.registers[0x0] = 213;
        emu.i = 0x200;

        emu.execute(Instruction::StoreNumberFromRegToI(0x0));

        assert_eq!(emu.memory[0x200], 2);
        assert_eq!(emu.memory[0x201], 1);
        assert_eq!(emu.memory[0x202], 3);
    }

    #[test]
    fn pixels_to_byte() {
        let pixels = [true, false, false, true];

        assert_eq!(0x9, Chip8::pixels_to_byte(&pixels));
    }

    #[test]
    fn byte_to_pixels() {
        let expected = [false, false, false, false, false, true, false, true];
        assert_eq!(expected, Chip8::byte_to_pixels(0x5));
    }
}
