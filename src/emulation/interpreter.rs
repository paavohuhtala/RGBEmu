
use emulation::bitutils::*;
use emulation::device::{Device, ReadWriteRegisters};
use emulation::registers::{StatusFlag, Registers};
use emulation::instruction_decoder::decode_instruction;
use emulation::instruction::{Operand8, Operand16, ConditionCode};
use emulation::instruction::Instruction::*;
use emulation::instruction::Operand8::*;
use emulation::instruction::Operand16::*;
use emulation::mmu::{MMU};
use emulation::address_mapper::{Addressable};

fn check_condition(device: &Device, condition: ConditionCode) -> bool {
  match condition {
    ConditionCode::Zero(status) => device.regs.get_flag(StatusFlag::Z) == status,
    ConditionCode::Carry(status) => device.regs.get_flag(StatusFlag::C) == status
  }
}

fn push_16(memory: &mut MMU, registers: &mut Registers, value: u16) {
  let BytePair { high, low } = u16_to_pair(value);
  memory.write_addr_8(registers.sp, low);
  memory.write_addr_8(registers.sp - 1, high);
  registers.sp -= 2;
}

fn pop_16(memory: &MMU, registers: &mut Registers) -> u16 {
  let high = memory.read_addr_8(registers.sp + 1);
  let low = memory.read_addr_8(registers.sp + 2);
  registers.sp += 2;
  u16_from_bytes(high, low)
}

pub fn rotate_left_carry(device: &mut Device, operand: Operand8, modify_zero: bool) {
  let op = operand.get(device);
  let last_bit = op.get_bit(7);
  let rotated = (op << 1).set_bit_to(0, last_bit);
  
  device.regs.set_flag_to(StatusFlag::C, last_bit);
  device.set_operand_8(operand, rotated);
  device.regs.clear_flag(StatusFlag::H);

  if modify_zero {
    device.regs.set_flag_to(StatusFlag::Z, rotated == 0);
  }
}

pub fn run_cycle(device: &mut Device) -> i32 {
  let instruction = decode_instruction(device);
  println!("{:?}", instruction);
  println!("{:?}", device.regs);
  match instruction {
    MoveImmediate16 {to, value} => { device.set_operand_16(to, value); 12 },
    XorOperandWithA(operand) => {
      let a = device.get_operand_8(A);
      let res = a ^ device.get_operand_8(operand);
      device.set_operand_8(A, res);
      device.regs.set_flag_to(StatusFlag::Z, res == 0);
      device.regs.clear_flag(StatusFlag::N);
      device.regs.clear_flag(StatusFlag::H);
      device.regs.clear_flag(StatusFlag::C);

      if operand.is_memref() || operand.is_immediate() {8} else {4}
    },
    TestBit(n, operand) => {
      let op = device.get_operand_8(operand);
      device.regs.set_flag_to(StatusFlag::Z, !op.get_bit(n));
      8
    },
    ConditionalRelativeJump(condition, offset) => {
      if check_condition(device, condition) {
        device.regs.pc = (device.regs.pc as i32 + offset as i32) as u16;
        12
      } else {
        8
      }
    },
    MoveOperand8 {to, from} => {
      let value = device.get_operand_8(from);
      device.set_operand_8(to, value);
      4
    },
    LoadAHigh(offset) => {
      let addr = 0xFF00 + (offset as u16);
      let value = device.memory.read_addr_8(addr);
      A.set(device, value);
      12
    },
    StoreAHighC => {
      let c = device.get_operand_8(C);
      let a = device.get_operand_8(A);
      let address = 0xFF00 + c as u16;
      device.memory.write_addr_8(address, a);
      8
    },
    StoreAHigh(offset) => {
      let a = device.get_operand_8(A);
      let address = 0xFF00 + offset as u16;
      device.memory.write_addr_8(address, a);
      12
    }
    IncrementOperand8(operand) => {
      let inc = device.get_operand_8(operand).wrapping_add(1);
      device.set_operand_8(operand, inc);
      device.regs.set_flag_to(StatusFlag::Z, inc == 0);
      device.regs.clear_flag(StatusFlag::N);
      // TODO: handle half-carry
      if operand.is_memref() {12} else {4}
    },
    DecrementOperand8(operand) => {
      let dec = device.get_operand_8(operand).wrapping_sub(1);
      device.set_operand_8(operand, dec);
      device.regs.set_flag_to(StatusFlag::Z, dec == 0);
      device.regs.set_flag(StatusFlag::N);
      // TODO: handle half-carry
      if operand.is_memref() {12} else {4}
    }
    LoadAIndirect(operand) => {
      let op = device.get_operand_16(operand);
      let value = device.memory.read_addr_8(op);
      device.set_operand_8(A, value);
      8
    },
    Call(addr) => {
      let return_addr = device.regs.pc;
      push_16(&mut device.memory, &mut device.regs, return_addr);
      let stack_space = device.memory.high_ram[100 .. 127].iter().collect::<Vec<_>>();
      println!("Stack space: {:?}", stack_space);
      device.regs.pc = addr;
      24
    },
    Return => {
      let addr = pop_16(&device.memory, &mut device.regs);
      device.regs.pc = addr;
      16
    },
    Push(operand) => {
      let op = device.get_operand_16(operand);
      push_16(&mut device.memory, &mut device.regs, op);
      16
    },
    Pop(operand) => {
      let value = pop_16(&device.memory, &mut device.regs);
      device.set_operand_16(operand, value);
      12
    },
    RotateLeft(operand) => {
      let op = operand.get(device);
      let last_bit = op.get_bit(7);
      let carry_bit = device.regs.get_flag(StatusFlag::C);
      let rotated = (op << 1).set_bit_to(0, carry_bit);

      device.regs.set_flag_to(StatusFlag::C, last_bit);
      device.set_operand_8(operand, rotated);
      device.regs.set_flag_to(StatusFlag::Z, rotated == 0);
      device.regs.clear_flag(StatusFlag::H);

      if operand.is_memref() { 16 } else { 8 }
    },
    RotateLeftCarry(operand) => {
      rotate_left_carry(device, operand, true);
      if operand.is_memref() { 16 } else { 8 }
    },
    RotateLeftCarryA => {
      rotate_left_carry(device, A, false);
      4
    },
    StoreAIndirectHLIncrement => {
      let hl = HL.get(device);
      let a = A.get(device);
      device.memory.write_addr_8(hl, a);
      device.regs.set_hl(hl + 1);
      8
    },
    StoreAIndirectHLDecrement => {
      let hl = HL.get(device);
      let a = A.get(device);
      device.memory.write_addr_8(hl, a);
      device.regs.set_hl(hl - 1);
      8
    },
    IncrementOperand16(operand) => {
      let op = operand.get(device);
      device.set_operand_16(operand, op.wrapping_add(1));
      8
    },
    CompareOperandWithA(operand) => {
      let a = A.get(device);
      let op = operand.get(device);
      let subbed = a.wrapping_sub(op);

      device.regs.set_flag(StatusFlag::N);
      device.regs.set_flag_to(StatusFlag::Z, subbed == 0);
      device.regs.set_flag_to(StatusFlag::C, op > a);
      // TODO: handle half carry
      if operand.is_memref() || operand.is_immediate() { 8 } else { 4 }
    },
    StoreA(addr) => {
      let a = A.get(device);
      device.memory.write_addr_8(addr, a);
      16
    },
    RelativeJump(offset) => {
      device.regs.pc = (device.regs.pc as i32 + offset as i32) as u16;
      12
    },
    _ => panic!("Unimplemented instruction: {:?}", instruction)
  }
}
