
use std::fmt;
use std::fmt::*;

use crate::emulation::instruction::{Operand8, Operand16};
use crate::emulation::instruction::*;
use crate::emulation::instruction::Instruction::*;
use crate::emulation::instruction::Operand8::*;
use crate::emulation::instruction::Operand16::*;

impl Display for Operand8 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      A => write!(f, "a"),
      B => write!(f, "b"),
      C => write!(f, "c"),
      D => write!(f, "d"),
      E => write!(f, "e"),
      H => write!(f, "h"),
      L => write!(f, "l"),
      Immediate(value) => write!(f, "${:X}", value),
      MemoryReference => write!(f, "(hl)")
    }
  }
}

impl Display for Operand16 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      BC => write!(f, "bc"),
      DE => write!(f, "de"),
      HL => write!(f, "hl"),
      SP => write!(f, "sp"),
    }
  }
}

pub fn to_asm(instruction: &Instruction) -> String {
  match *instruction {
    AndOperandWithA(op) => format!("and {}", op),
    LoadAHigh(offs) => format!("ldh a, (${})", offs),
    _ => format!("{:?}", instruction)
  }
}
