
use emulation::instruction::*;
use emulation::instruction::Instruction::*;
use emulation::instruction_decoder::*;

struct MockDevice {
  data: Vec<u8>,
  pc: u16
}

impl ReadOnlyByteStream for MockDevice {
  fn read_next_byte(&mut self) -> u8 {
    let pc = self.pc;
    self.pc += 1;
    self.data[pc as usize]
  }

  fn get_stream_position(&self) -> u16 {
    self.pc
  }
}

impl MockDevice {
  pub fn from_bytes(bytes: Vec<u8>) -> MockDevice {
    MockDevice { data: bytes, pc: 0}
  }
}

fn decode_bytes(bytes: Vec<u8>) -> Instruction {
  let mut device = MockDevice::from_bytes(bytes);
  decode_instruction(&mut device)
}

fn verify_instruction(expected: Instruction, bytes: Vec<u8>) {
  let instruction = decode_bytes(bytes);
  assert_eq!(expected, instruction);
}

#[test]
pub fn random_instructions() {
  verify_instruction(Nop, vec![0]);
  verify_instruction(OrOperandWithA(Operand8::B), vec![0xB0]);
  verify_instruction(MoveOperand8 {to: Operand8::MemoryReference, from: Operand8::L}, vec![0x75]);
  verify_instruction(Instruction::SetBit(4, Operand8::D), vec![0xCB, 0xE2]);
  verify_instruction(Instruction::SubtractOperandFromABorrow(Operand8::A), vec![0x9F]);
}

use std::panic::catch_unwind;

#[test]
pub fn removed_instructions_should_panic() {
  let removed_instructions = vec![
    vec![0xD3],
    vec![0xDB],
    vec![0xDD],
    vec![0xE3],
    vec![0xE4],
    vec![0xEB],
    vec![0xEC],
    vec![0xF2],
    vec![0xF4],
    vec![0xFC],
    vec![0xFD]];

  for i in removed_instructions {
    let i_copy = i.clone();
    let decoded = catch_unwind(||decode_bytes(i));

    if decoded.is_ok() {
      panic!("Trying to decode instruction {:?} should've paniced, but was {:?}.", i_copy, decoded.unwrap());
    }
  }
}
