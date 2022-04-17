use std::fmt;

pub enum Instruction {
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

impl TryFrom<u16> for Instruction {
    type Error = InstructionError;

    fn try_from(raw: u16) -> Result<Self, Self::Error> {
        match raw {
            0x00E0 => Ok(Self::ClearDisplay),
            0x00EE => Ok(Self::ReturnFromSubroutine),
            0x1000..=0x1FFF => Ok(Self::Jump(raw & 0x0FFF)),
            0x2000..=0x2FFF => Ok(Self::CallSubroutine(raw & 0x0FFF)),
            0x3000..=0x3FFF => Ok(Self::SkipRegEqByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x4000..=0x4FFF => Ok(Self::SkipRegNotEqByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x5000..=0x5FFF => match raw & 0xF {
                0x0 => Ok(Self::SkipRegEqReg(
                    ((raw & 0x0F00) >> 8) as u8,
                    ((raw & 0xF0) >> 4) as u8,
                )),
                _ => Err(InstructionError::Invalid(raw)),
            },
            0x6000..=0x6FFF => Ok(Self::SetRegToByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x7000..=0x7FFF => Ok(Self::AddByteToReg(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0x8000..=0x8FFF => match raw & 0xF {
                0x0 => Ok(Self::MoveValue(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x1 => Ok(Self::OrRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x2 => Ok(Self::AndRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x3 => Ok(Self::XorRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x4 => Ok(Self::AddRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x5 => Ok(Self::SubRegs(((raw & 0x0F00) >> 8) as u8, ((raw & 0xF0) >> 4) as u8)),
                0x6 => Ok(Self::ShiftRight(((raw & 0x0F00) >> 8) as u8)),
                0x7 => Ok(Self::ReverseSubRegs(
                    ((raw & 0x0F00) >> 8) as u8,
                    (raw & 0xFF) as u8,
                )),
                0xE => Ok(Self::ShiftLeft(((raw & 0x0F00) >> 8) as u8)),
                _ => Err(InstructionError::Invalid(raw)),
            },
            0x9000..=0x9FFF => match raw & 0xF {
                0x0 => Ok(Self::SkipRegNotEqReg(
                    ((raw & 0x0F00) >> 8) as u8,
                    ((raw & 0xF0) as u8) >> 4,
                )),
                _ => Err(InstructionError::Invalid(raw)),
            },
            0xA000..=0xAFFF => Ok(Self::SetI(raw & 0x0FFF)),
            0xB000..=0xBFFF => Ok(Self::JumpV0PlusByte(raw & 0x0FFF)),
            0xC000..=0xCFFF => Ok(Self::SetRegToRandPlusByte(
                ((raw & 0x0F00) >> 8) as u8,
                (raw & 0xFF) as u8,
            )),
            0xD000..=0xDFFF => Ok(Self::DrawSprite(
                ((raw & 0x0F00) >> 8) as u8,
                ((raw & 0xF0) >> 4) as u8,
                (raw & 0xF) as u8,
            )),
            0xE000..=0xEFFF => match raw & 0xFF {
                0x9E => Ok(Self::SkipIfKey(((raw & 0x0F00) >> 8) as u8)),
                0xA1 => Ok(Self::SkipIfNotKey(((raw & 0x0F00) >> 8) as u8)),
                _ => Err(InstructionError::Invalid(raw)),
            },
            0xF000..=0xFFFF => match raw & 0xFF {
                0x07 => Ok(Self::LoadDelayToReg(((raw & 0x0F00) >> 8) as u8)),
                0x0A => Ok(Self::LoadKeyToReg(((raw & 0x0F00) >> 8) as u8)),
                0x15 => Ok(Self::SetDelayToReg(((raw & 0x0F00) >> 8) as u8)),
                0x18 => Ok(Self::SetSoundToReg(((raw & 0x0F00) >> 8) as u8)),
                0x1E => Ok(Self::AddRegToI(((raw & 0x0F00) >> 8) as u8)),
                0x29 => Ok(Self::SetIToDigitSpriteLoc(((raw & 0x0F00) >> 8) as u8)),
                0x33 => Ok(Self::StoreNumberFromRegToI(((raw & 0x0F00) >> 8) as u8)),
                0x55 => Ok(Self::StoreRegsToMem(((raw & 0x0F00) >> 8) as u8)),
                0x65 => Ok(Self::LoadRegsFromMem(((raw & 0x0F00) >> 8) as u8)),
                _ => Err(InstructionError::Invalid(raw)),
            },
            _ => Err(InstructionError::Invalid(raw)),
        }
    }
}

pub enum InstructionError {
    Invalid(u16),
}

impl fmt::Debug for InstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(raw) => write!(f, "invalid opcode {:X}", raw),
        }
    }
}
