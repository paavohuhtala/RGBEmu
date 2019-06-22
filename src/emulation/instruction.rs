use crate::emulation::device::{Device, ReadWriteRegisters};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    MemoryReference,
    Immediate(u8)
}

impl Operand8 {
    pub fn decode(value: u8) -> Option<Operand8> {
        match value {
            0b111 => Some(Operand8::A),
            0b000 => Some(Operand8::B),
            0b001 => Some(Operand8::C),
            0b010 => Some(Operand8::D),
            0b011 => Some(Operand8::E),
            0b100 => Some(Operand8::H),
            0b101 => Some(Operand8::L),
            0b110 => Some(Operand8::MemoryReference),
            _ => None
        }
    }

    pub fn is_memref(self) -> bool {
        match self {
            Operand8::MemoryReference => true,
            _ => false
        }
    }

    pub fn is_immediate(self) -> bool {
        match self {
            Operand8::Immediate(_) => true,
            _ => false
        }
    }

    pub fn get(self, device: &Device) -> u8 {
        device.get_operand_8(self)
    }

    pub fn set(self, device: &mut Device, value: u8) {
        device.set_operand_8(self, value);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand16 {
    BC,
    DE,
    HL,
    SP
}

impl Operand16 {
    pub fn decode(value: u8) -> Option<Operand16> {
        match value {
            0b00 => Some(Operand16::BC),
            0b01 => Some(Operand16::DE),
            0b10 => Some(Operand16::HL),
            0b11 => Some(Operand16::SP),
            _ => None
        }
    }

    pub fn get(self, device: &Device) -> u16 {
        device.get_operand_16(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum ConditionCode {
    Zero(bool),
    Carry(bool)
}

impl ConditionCode {
    pub fn decode(value: u8) -> Option<ConditionCode> {
        match value {
            0b000 => Some(ConditionCode::Zero(false)),
            0b001 => Some(ConditionCode::Zero(true)),
            0b010 => Some(ConditionCode::Carry(false)),
            0b011 => Some(ConditionCode::Carry(true)),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Unknown(u16),
    MoveOperand8 { to: Operand8, from: Operand8 },
    MoveImmediate16 { to: Operand16, value: u16 },
    LoadA(u16),
    StoreA(u16),
    LoadAIndirectHLIncrement,
    StoreAIndirectHLIncrement,
    LoadAIndirectHLDecrement,
    StoreAIndirectHLDecrement,
    LoadAIndirect(Operand16),
    StoreAIndirect(Operand16),
    LoadAHigh(u8),
    StoreAHigh(u8),
    StoreAHighC,
    MoveSPOffsetToHL(u8),
    MoveHLToSP,
    StoreSP(u16),
    AddOperandToA(Operand8),
    AddOperandToACarry(Operand8),
    SubtractOperandFromA(Operand8),
    SubtractOperandFromABorrow(Operand8),
    IncrementOperand8(Operand8),
    DecrementOperand8(Operand8),
    IncrementOperand16(Operand16),
    DecrementOperand16(Operand16),
    AddOperandToHL(Operand16),
    BCDCorrectA,
    AndOperandWithA(Operand8),
    OrOperandWithA(Operand8),
    XorOperandWithA(Operand8),
    CompareOperandWithA(Operand8),
    RotateLeftA,
    RotateRightA,
    RotateLeftCarryA,
    RotateRightCarryA,
    ComplementA,
    ComplementCarry,
    SetCarry,
    Jump(u16),
    ConditionalJump(ConditionCode, u16),
    RelativeJump(i8),
    ConditionalRelativeJump(ConditionCode, i8),
    Call(u16),
    ConditionalCall(ConditionCode, u16),
    Return,
    ReturnFromInterrupt,
    ConditionalReturn(ConditionCode),
    Restart(u8),
    JumpToHL,
    Push(Operand16),
    Pop(Operand16),
    EnableInterrupts,
    DisableInterrupts,
    Halt,
    Nop,
    RotateLeft(Operand8),
    RotateRight(Operand8),
    RotateLeftCarry(Operand8),
    RotateRightCarry(Operand8),
    ShiftLeftArithmetic(Operand8),
    ShiftLeftLogical(Operand8),
    ShiftRightArithmetic(Operand8),
    ShiftRightLogical(Operand8),
    TestBit(u8, Operand8),
    SetBit(u8, Operand8),
    ClearBit(u8, Operand8),
    AddSignedImmediateToSP(i8),
    Stop
}
