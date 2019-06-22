use crate::emulation::device::{Device};
use crate::emulation::registers::{StatusFlag};
use crate::emulation::instruction::{ConditionCode};

fn check_condition(device: &Device, condition: ConditionCode) -> bool {
  match condition {
    ConditionCode::Zero(status) => device.regs.get_flag(StatusFlag::Z) == status,
    ConditionCode::Carry(status) => device.regs.get_flag(StatusFlag::C) == status
  }
}

pub fn conditional_jump(device: &mut Device, condition: ConditionCode, address: u16) -> u32 {
  if check_condition(device, condition) {
    device.regs.pc = address;
    16
  } else {
    12
  }
}

pub fn conditional_relative_jump(device: &mut Device, condition: ConditionCode, offset: i8) -> u32 {
  if check_condition(device, condition) {
    device.regs.pc = (device.regs.pc as i32 + offset as i32) as u16;
    12
  } else {
    8
  }
}

pub fn call(device: &mut Device, addr: u16) -> u32 {
  let return_addr = device.regs.pc;
  device.push_16(return_addr);
  device.regs.pc = addr;
  24
}

pub fn conditional_call(device: &mut Device, condition: ConditionCode, addr: u16) -> u32 {
  if check_condition(device, condition) {
    call(device, addr);
    24
  } else {
    12
  }
}

pub fn unconditional_return(device: &mut Device) -> u32 {
  let addr = device.pop_16();
  device.regs.pc = addr;
  16
}

pub fn conditional_return(device: &mut Device, condition: ConditionCode) -> u32 {
  if check_condition(device, condition) {
    unconditional_return(device);
    20
  } else {
    8
  }
}

pub fn return_from_interrupt(device: &mut Device) -> u32 {
  unconditional_return(device);
  device.interrupts_enabled = true;  
  16
}

pub fn restart(device: &mut Device, handler_id: u8) -> u32 {
  let handler_addr = (handler_id as u16) * 8;
  let pc = device.regs.pc;
  device.push_16(pc);
  device.regs.pc = handler_addr;
  16
}

pub fn jump_to_hl(device: &mut Device) -> u32 {
  let hl = device.regs.hl();
  device.regs.pc = hl;
  4
}
