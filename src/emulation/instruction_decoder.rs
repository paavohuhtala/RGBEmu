
use emulation::instruction::{Instruction, Operand, RegisterPair, BCOrDE, ConditionCode};
use emulation::instruction::Instruction::*;

pub trait ReadOnlyByteStream {
  fn read_next_byte(&mut self) -> u8;
  fn get_stream_position(&self) -> u16;

  fn read_next_16(&mut self) -> u16 {
    let low = self.read_next_byte();
    let high = self.read_next_byte();

    ((high as u16) << 8) | (low as u16)
  }
}

mod bitutils {
  // Based on https://www.reddit.com/r/rust/comments/2d7rrj/bit_level_pattern_matching/cjmxjtn
  #[inline]
  pub fn to_bit_tuple(byte: u8) -> (u8, u8, u8, u8, u8, u8, u8, u8) {
    (byte >> 7 & 1,
     byte >> 6 & 1,
     byte >> 5 & 1,
     byte >> 4 & 1,
     byte >> 3 & 1,
     byte >> 2 & 1,
     byte >> 1 & 1,
     byte & 1)
  }
  
  #[inline]
  pub fn to_byte_2(b1: u8, b0: u8) -> u8 {
    b1 << 1 | b0
  }

  #[inline]
  pub fn to_byte_3(b2: u8, b1: u8, b0: u8) -> u8 {
    b2 << 2 | b1 << 1 | b0
  }
}

fn as_register_pair(b1: u8, b0: u8) -> RegisterPair {
  RegisterPair::decode(bitutils::to_byte_2(b1, b0)).unwrap()
}

fn as_bc_or_de(b1: u8, b0: u8) -> BCOrDE {
  RegisterPair::decode(bitutils::to_byte_2(b1, b0)).and_then(RegisterPair::as_bc_or_de).unwrap()
}

fn to_operand(b2: u8, b1: u8, b0: u8) -> Operand {
  Operand::decode(bitutils::to_byte_3(b2, b1, b0)).unwrap()
}

fn to_condition_code(b2: u8, b1: u8, b0: u8) -> ConditionCode {
  ConditionCode::decode(bitutils::to_byte_3(b2, b1, b0)).unwrap()
}

pub fn decode_instruction(device: &mut ReadOnlyByteStream) -> Instruction {
  print!("${:04x} ", device.get_stream_position());
  let first_byte = device.read_next_byte();
  println!("0x{:X}", first_byte);
  let first_bits = bitutils::to_bit_tuple(first_byte);

  match first_bits {
    (0, 0, 0, 0, 0, 0, 0, 0) => Nop,
    (0, 1, 1, 1, 0, 1, 1, 0) => Halt,
    (0, 0, 0, 1, 0, 0, 0, 0) => Stop,
    (0, 1, d2, d1, d0, s2, s1, s0) => {
      let to =  to_operand(d2, d1, d0);
      let from = to_operand(s2, s1, s0);
      MoveOperand8 {to: to, from: from}
    },
    (0, 0, d2, d1, d0, 1, 1, 0) => {
      let to = to_operand(d2, d1, d0);
      let immediate = device.read_next_byte();
      MoveImmediate8 {to: to, value: immediate}
    },
    (0, 0, r1, r0, 0, 0, 0, 1) => {
      let to = as_register_pair(r1, r0);
      let immediate = device.read_next_16();
      MoveImmediate16 {to: to, value: immediate}
    },
    (1, 1, 1, 1, 1, 0, 1, 0) => LoadA(device.read_next_16()),
    (1, 1, 1, 0, 1, 0, 1, 0) => StoreA(device.read_next_16()),
    (0, 0, 1, 0, 1, 0, 1, 0) => LoadAIndirectHLIncrement,
    (0, 0, 1, 0, 0, 0, 1, 0) => StoreAIndirectHLIncrement,
    (0, 0, 1, 1, 1, 0, 1, 0) => LoadAIndirectHLDecrement,
    (0, 0, 1, 1, 0, 0, 1, 0) => StoreAIndirectHLDecrement,
    (0, 0, r1, r0, 1, 0, 1, 0) => LoadAIndirect(as_bc_or_de(r1, r0)),
    (0, 0, r1, r0, 0, 0, 1, 0) => StoreAIndirect(as_bc_or_de(r1, r0)),
    (1, 1, 1, 1, 0, 0, 0, 0) => LoadAHigh(device.read_next_byte()),
    (1, 1, 1, 0, 0, 0, 0, 0) => StoreAHigh(device.read_next_byte()),
    (1, 1, 1, 0, 0, 0, 1, 0) => StoreAHighC,
    (1, 0, 0, 0, carry, s2, s1, s0) => {
      let register = to_operand(s2, s1, s0);
      if carry == 1 {
        AddOperandToACarry(register)
      } else {
        AddOperandToA(register)
      }
    },
    (1, 1, 0, 0, carry, 1, 1, 0) => {
      let immediate = device.read_next_byte();
      if carry == 1 {
        AddImmediateToACarry(immediate)
      } else {
        AddImmediateToA(immediate)
      }
    },
    (1, 0, 0, 1, carry, s2, s1, s0) => {
      let register = to_operand(s2, s1, s0);
      if carry == 1 {
        SubtractOperandFromABorrow(register)
      } else {
        SubtractOperandFromA(register)
      }
    },
    (1, 1, 0, 1, carry, 1, 1, 0) => {
      let immediate = device.read_next_byte();
      if carry == 1 {
        SubtractImmediateFromABorrow(immediate)
      } else {
        SubtractImmediateFromA(immediate)
      }
    },
    (0, 0, d2, d1, d0, 1, 0, decr) => {
      let register = to_operand(d2, d1, d0);
      if decr == 1 {
        DecrementOperand8(register)
      } else {
        IncrementOperand8(register)
      }
    },
    (0, 0, r1, r0, decr, 0, 1, 1) => {
      let register_pair = as_register_pair(r1, r0);
      if decr == 1 {
        DecrementOperand16(register_pair)
      } else {
        IncrementOperand16(register_pair)
      }
    },
    (0, 0, r1, r0, 1, 0, 0, 1) => AddOperandToHL(as_register_pair(r1, r0)),
    (0, 0, 1, 0, 0, 1, 1, 1) => BCDCorrectA,
    (1, 0, 1, 0, 0, s2, s1, s0) => AndOperandWithA(to_operand(s2, s1, s0)),
    (1, 1, 1, 0, 0, 1, 1, 0) => AndImmediateWithA(device.read_next_byte()),
    (1, 0, 1, 1, 0, s2, s1, s0) => OrOperandWithA(to_operand(s2, s1, s0)),
    (1, 1, 1, 1, 0, 1, 1, 0) => OrImmediateWithA(device.read_next_byte()),
    (1, 0, 1, 0, 1, s2, s1, s0) => XorOperandWithA(to_operand(s2, s1, s0)),
    (1, 1, 1, 0, 1, 1, 1, 0) => XorImmediateWithA(device.read_next_byte()),
    (1, 0, 1, 1, 1, s2, s1, s0) => CompareOperandWithA(to_operand(s2, s1, s0)),
    (1, 1, 1, 1, 1, 1, 1, 0) => CompareImmediateWithA(device.read_next_byte()),
    (0, 0, 0, carry, right, 1, 1, 1) => {
      match (carry == 1, right == 1) {
        (false, false) => RotateALeft,
        (true, false) => RotateALeftCarry,
        (false, true) => RotateARight,
        (true, true) => RotateARightCarry
      }
    },
    (0, 0, 1, 0, 1, 1, 1, 1) => ComplementA,
    (0, 0, 1, 1, 1, 1, 1, 1) => ComplementCarry,
    (0, 0, 1, 1, 0, 1, 1, 1) => SetCarry,
    (1, 1, 0, 0, 0, 0, 1, 1) => Jump(device.read_next_16()),
    (1, 1, c2, c1, c0, 0, 1, 0) => ConditionalJump(to_condition_code(c2, c1, c0), device.read_next_16()),
    (0, 0, 0, 1, 1, 0, 0, 0) => RelativeJump(device.read_next_byte() as i8),
    (0, 0, 1, 0, 0, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Zero(false), device.read_next_byte() as i8),
    (0, 0, 1, 1, 0, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Carry(false), device.read_next_byte() as i8),
    (0, 0, 1, 0, 1, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Zero(true), device.read_next_byte() as i8),
    (0, 0, 1, 1, 1, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Carry(true), device.read_next_byte() as i8),
    (1, 1, 0, 0, 1, 1, 0, 1) => Call(device.read_next_16()),
    (1, 1, c2, c1, c0, 1, 0, 0) => ConditionalCall(to_condition_code(c2, c1, c0), device.read_next_16()),
    (1, 1, 0, 0, 1, 0, 0, 1) => Return,
    (1, 1, c2, c1, c0, 0, 0, 0) => ConditionalReturn(to_condition_code(c2, c1, c0)),
    (1, 1, n2, n1, n0, 1, 1, 1) => Restart(bitutils::to_byte_3(n2, n1, n0)),
    (1, 1, 1, 0, 1, 0, 0, 1) => JumpToHL,
    (1, 1, r1, r0, 0, 1, 0, 1) => Push(as_register_pair(r1, r0)),
    (1, 1, r1, r0, 0, 0, 0, 1) => Pop(as_register_pair(r1, r0)),
    (1, 1, 1, 1, enable, 0, 1, 1) => {
      if enable == 1 {
        EnableInterrupts
      } else {
        DisableInterrupts
      }
    },
    // Bit instructions
    _ if first_byte == 0xCB => {
      println!("bit instructions");
      let next_byte = device.read_next_byte();
      println!("0x{:X}", next_byte);
      let next_bits = bitutils::to_bit_tuple(next_byte);
      match next_bits {
        (0, 0, 0, no_carry, 0, d2, d1, d0) => {
          let register = to_operand(d2, d1, d0);
          if no_carry == 1 {
            RotateLeft(register)
          } else {
            RotateLeftCarry(register)
          }
        },
        (0, 0, 0, no_carry, 1, d2, d1, d0) => {
          let register = to_operand(d2, d1, d0);
          if no_carry == 1 {
            RotateRight(register)
          } else {
            RotateRightCarry(register)
          }
        },
        (0, 0, 1, logical, 0, d2, d1, d0) => {
          let register = to_operand(d2, d1, d0);
          if logical == 1 {
            // Technically an undocumented instruction
            ShiftLeftLogical(register)
          } else {
            ShiftLeftArithmetic(register)
          }
        },
        (0, 0, 1, logical, 1, d2, d1, d0) => {
          let register = to_operand(d2, d1, d0);
          if logical == 1 {
            ShiftRightLogical(register)
          } else {
            ShiftRightArithmetic(register)
          }
        },
        (0, 1, b2, b1, b0, d2, d1, d0) => TestBit(bitutils::to_byte_3(b2, b1, b0), to_operand(d2, d1, d0)),
        (1, 0, b2, b1, b0, d2, d1, d0) => ClearBi(bitutils::to_byte_3(b2, b1, b0), to_operand(d2, d1, d0)),
        (1, 1, b2, b1, b0, d2, d1, d0) => SetBit(bitutils::to_byte_3(b2, b1, b0), to_operand(d2, d1, d0)),
        _ => panic!("This shouldn't happen.")
      }
    }
    _ => panic!("Unimplemented instruction: 0x{:02X}", first_byte)
  }
}