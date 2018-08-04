extern crate rgbemu;
use rgbemu::emulation::registers::{StatusFlag};

mod common;
use common::{run_program, read_address};

#[test]
fn store_sp() {
  let device = run_program(&[
    // LD SP, 0x1234
    0x31, 0x34, 0x12,
    // LD 0xCAAA, SP
    0x08, 0xAA, 0xCA,
    // HALT
    0x76
  ]);

  assert_eq!(0x34, read_address(&device, 0xCAAA));
  assert_eq!(0x12, read_address(&device, 0xCAAB));
}

#[test]
fn load_sp_offset_to_hl() {
  let device = run_program(&[
    // LD SP, 0x1233
    0x31, 0x33, 0x12,
    // LD HL, SP+1
    0xF8, 0x1,
    // HALT
    0x76
  ]);

  assert_eq!(0x1234, device.regs.hl());
  assert_eq!(false, device.regs.get_flag(StatusFlag::H));
  assert_eq!(false, device.regs.get_flag(StatusFlag::C));
}

#[test]
fn load_sp_offset_to_hl_carry() {
  let device = run_program(&[
    // LD SP, 0xFFFF
    0x31, 0xFE, 0xFF,
    // LD HL, SP+1
    0xF8, 0x2,
    // HALT
    0x76
  ]);

  assert_eq!(0x0000, device.regs.hl());
  assert_eq!(true, device.regs.get_flag(StatusFlag::H));
  assert_eq!(true, device.regs.get_flag(StatusFlag::C));
}

#[test]
fn push_de() {
  let device = run_program(&[
    // LD DE, 0xDEAD
    0x11, 0xAD, 0xDE,
    // PUSH DE
    0xD5,
    // HALT
    0x76
  ]);

  assert_eq!(0xAD, read_address(&device, device.regs.sp + 2));
  assert_eq!(0xDE, read_address(&device, device.regs.sp + 1));
}