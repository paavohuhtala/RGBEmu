
#[derive(Debug, PartialEq)]
pub enum Operand {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
  MemoryReference
}

impl Operand {
  pub fn decode(value: u8) -> Option<Operand> {
    match value {
      0b111 => Some(Operand::A),
      0b000 => Some(Operand::B),
      0b001 => Some(Operand::C),
      0b010 => Some(Operand::D),
      0b011 => Some(Operand::E),
      0b100 => Some(Operand::H),
      0b101 => Some(Operand::L),
      0b110 => Some(Operand::MemoryReference),
      _     => None
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum RegisterPair {
  BC,
  DE,
  HL,
  SP
}

impl RegisterPair {
  pub fn decode(value: u8) -> Option<RegisterPair> {
    match value {
      0b00 => Some(RegisterPair::BC),
      0b01 => Some(RegisterPair::DE),
      0b10 => Some(RegisterPair::HL),
      0b11 => Some(RegisterPair::SP),
      _    => None
    }
  }

  pub fn as_bc_or_de(self) -> Option<BCOrDE> {
    match self {
      RegisterPair::BC => Some(BCOrDE::BC),
      RegisterPair::DE => Some(BCOrDE::DE),
      _ => None
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum BCOrDE {
  BC,
  DE
}

impl BCOrDE {
  pub fn as_register_pair(self) -> RegisterPair {
    match self {
      BCOrDE::BC => RegisterPair::BC,
      BCOrDE::DE => RegisterPair::DE
    }
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
  MoveOperand8 {to: Operand, from: Operand},
  MoveImmediate8 {to: Operand, value: u8},
  MoveImmediate16 {to: RegisterPair, value: u16},
  LoadA(u16),
  StoreA(u16),
  LoadAIndirectHLIncrement,
  StoreAIndirectHLIncrement,
  LoadAIndirectHLDecrement,
  StoreAIndirectHLDecrement,
  LoadAIndirect(BCOrDE),
  StoreAIndirect(BCOrDE),
  LoadAHigh(u8),
  StoreAHigh(u8),
  StoreAHighC,
  AddOperandToA(Operand),
  AddImmediateToA(u8),
  AddOperandToACarry(Operand),
  AddImmediateToACarry(u8),
  SubtractOperandFromA(Operand),
  SubtractImmediateFromA(u8),
  SubtractOperandFromABorrow(Operand),
  SubtractImmediateFromABorrow(u8),
  IncrementOperand8(Operand),
  DecrementOperand8(Operand),
  IncrementOperand16(RegisterPair),
  DecrementOperand16(RegisterPair),
  AddOperandToHL(RegisterPair),
  BCDCorrectA,
  AndOperandWithA(Operand),
  AndImmediateWithA(u8),
  OrOperandWithA(Operand),
  OrImmediateWithA(u8),
  XorOperandWithA(Operand),
  XorImmediateWithA(u8),
  CompareOperandWithA(Operand),
  CompareImmediateWithA(u8),
  RotateALeft,
  RotateARight,
  RotateALeftCarry,
  RotateARightCarry,
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
  ConditionalReturn(ConditionCode),
  Restart(u8),
  JumpToHL,
  Push(RegisterPair),
  Pop(RegisterPair),
  MoveToSP(RegisterPair),
  EnableInterrupts,
  DisableInterrupts,
  Halt,
  Nop,
  RotateLeft(Operand),
  RotateRight(Operand),
  RotateLeftCarry(Operand),
  RotateRightCarry(Operand),
  ShiftLeftArithmetic(Operand),
  ShiftLeftLogical(Operand),
  ShiftRightArithmetic(Operand),
  ShiftRightLogical(Operand),
  TestBit(u8, Operand),
  SetBit(u8, Operand),
  ClearBi(u8, Operand),
  AddSignedImmediateToSP(i8),
  Stop  
}
