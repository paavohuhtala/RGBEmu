
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

fn verify_instruction(expected: Instruction, bytes: Vec<u8>) {
  let mut device = MockDevice::from_bytes(bytes);
  let instruction = decode_instruction(&mut device);
  assert_eq!(expected, instruction);
}

#[test]
pub fn random_instructions() {
  verify_instruction(Nop, vec![0]);
  verify_instruction(OrOperandWithA(Operand::B), vec![0xB0]);
  verify_instruction(MoveOperand8 {to: Operand::MemoryReference, from: Operand::L}, vec![0x75]);
  verify_instruction(Instruction::SetBit(4, Operand::D), vec![0xCB, 0xE2]);
  verify_instruction(Instruction::SubtractOperandFromABorrow(Operand::A), vec![0x9F]);
}
