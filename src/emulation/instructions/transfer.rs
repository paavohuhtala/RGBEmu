
use crate::emulation::bitutils::*;
use crate::emulation::device::{Device, ReadWriteRegisters};
use crate::emulation::registers::{StatusFlag};
use crate::emulation::instruction::{Operand8, Operand16};

pub fn move_immediate_16(device: &mut Device, to: Operand16, value: u16) -> u32 {
  device.set_operand_16(to, value);
  12
}

pub fn move_immediate_8(device: &mut Device, to: Operand8, value: u8) -> u32 {
  device.set_operand_8(to, value);
  12
}

pub fn move_hl_to_sp(device: &mut Device) -> u32 {
  let hl = device.regs.hl();
  device.regs.sp = hl;
  8
}

pub fn move_sp_with_offset_to_hl(device: &mut Device, offset: u8) -> u32 {
  let sp = device.regs.sp;
  let CarryAddResult { result, carry, half_carry } = carry_add_16(sp, offset as u16);

  device.regs.set_hl(result);
  device.regs.clear_flag(StatusFlag::Z);
  device.regs.clear_flag(StatusFlag::N);
  device.regs.set_flag_to(StatusFlag::H, half_carry);
  device.regs.set_flag_to(StatusFlag::C, carry);

  12
}

pub fn push_16(device: &mut Device, operand: Operand16) -> u32 {
  let op = operand.get(device);
  device.push_16(op);
  16
}

pub fn pop_16(device: &mut Device, operand: Operand16) -> u32 {
  let value = device.pop_16();
  device.set_operand_16(operand, value);
  12
}

pub fn store_sp(device: &mut Device, address: u16) -> u32 {
  let sp = device.regs.sp;
  device.write_addr_16(address, sp);
  12
}

pub fn move_sp_offset_to_hl(device: &mut Device, offset: u8) -> u32 {
  let a = offset as u16;
  let b = device.regs.sp;
  let CarryAddResult { result, carry, half_carry } = carry_add_16(a, b);

  device.regs.set_flag_to(StatusFlag::H, half_carry);
  device.regs.set_flag_to(StatusFlag::C, carry);
  device.regs.set_hl(result);

  12
}