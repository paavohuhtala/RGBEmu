
use emulation::bitutils::*;

use emulation::device::{Device, ReadWriteRegisters};
use emulation::registers::{StatusFlag};
use emulation::instruction::{Operand8, Operand16};
use emulation::instruction::Operand8::*;

use std::ops::*;

fn add_operand_to_operand(device: &mut Device, a: Operand8, b: Operand8) {
  let op_a = a.get(device);
  let op_b = b.get(device);

  let CarryAddResult { result, carry, half_carry } = carry_add_8(op_a, op_b);

  device.regs.clear_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::Z, result == 0);
  device.regs.set_flag_to(StatusFlag::H, half_carry);
  device.regs.set_flag_to(StatusFlag::C, carry);

  a.set(device, result);
}

pub fn add_operand_8_to_a(device: &mut Device, operand: Operand8) -> u32 {
  add_operand_to_operand(device, A, operand);
  if operand.is_memref() || operand.is_immediate() { 8 } else { 4 }
}

pub fn subtract_operand_8_from_a(device: &mut Device, operand: Operand8) -> u32 {
  let a = A.get(device);
  let op = operand.get(device);
  
  let CarryAddResult { result, carry, half_carry } = borrow_sub_8(a, op);

  device.regs.set_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::Z, result == 0);
  device.regs.set_flag_to(StatusFlag::C, carry);
  device.regs.set_flag_to(StatusFlag::H, half_carry);

  device.regs.a = result;

  if operand.is_memref() || operand.is_immediate() { 8 } else { 4 }
}

pub fn increment_operand_8(device: &mut Device, operand: Operand8) -> u32 {
  let a = operand.get(device);
  let CarryAddResult { result, carry, half_carry } = carry_add_8(a, 1);

  operand.set(device, result);

  device.regs.clear_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::Z, result == 0);
  device.regs.set_flag_to(StatusFlag::H, half_carry);

  if operand.is_memref() { 12 } else { 4 }
}

pub fn decrement_operand_8(device: &mut Device, operand: Operand8) -> u32 {
  let op = operand.get(device);

  let CarryAddResult { result, .. } = borrow_sub_8(op, 1);

  device.regs.set_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::Z, result == 0);
  device.regs.set_flag_to(StatusFlag::H, op & 0xf == 0);

  operand.set(device, result);

  if operand.is_memref() { 12 } else { 4 }
}

pub fn add_operand_and_carry_to_a(device: &mut Device, operand: Operand8) -> u32 {
  let a = A.get(device);
  let op = operand.get(device);
  let carry = device.regs.get_flag(StatusFlag::C);

  let CarryAddResult { result, carry, half_carry } = carry_add_8_prev_carry(a, op, carry);

  device.regs.clear_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::Z, result == 0);
  device.regs.set_flag_to(StatusFlag::C, carry);
  device.regs.set_flag_to(StatusFlag::Z, half_carry);

  A.set(device, result);

  if operand.is_memref() || operand.is_immediate() { 8 } else { 4 }
}

pub fn add_operand_to_hl(device: &mut Device, operand: Operand16) -> u32 {
  let op = operand.get(device);
  let hl = device.regs.hl();

  let CarryAddResult { result, carry, half_carry } = carry_add_16(hl, op);

  device.regs.set_hl(result);
  device.regs.clear_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::C, carry);
  device.regs.set_flag_to(StatusFlag::H, half_carry);
  8
}

pub fn set_carry_flag(device: &mut Device) -> u32 {
  device.regs.set_flag(StatusFlag::C);
  device.regs.clear_flag(StatusFlag::N);
  device.regs.clear_flag(StatusFlag::H);
  4
}

pub fn flip_carry_flag(device: &mut Device) -> u32 {
  let carry = device.regs.get_flag(StatusFlag::C);
  device.regs.set_flag_to(StatusFlag::C, !carry);
  device.regs.clear_flag(StatusFlag::N);
  device.regs.clear_flag(StatusFlag::H);
  4
}

pub fn increment_operand_16(device: &mut Device, operand: Operand16) -> u32 {
  let op = operand.get(device);
  device.set_operand_16(operand, op.wrapping_add(1));
  8
}

pub fn decrement_operand_16(device: &mut Device, operand: Operand16) -> u32 {
  let op = operand.get(device);
  device.set_operand_16(operand, op.wrapping_sub(1));
  8
}
