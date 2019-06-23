use crate::emulation::bitutils::*;

use crate::emulation::device::Device;
use crate::emulation::instruction::Operand8;
use crate::emulation::instruction::Operand8::*;
use crate::emulation::registers::StatusFlag;

use std::ops::*;

#[derive(Debug)]
enum BitwiseOp {
    And,
    Or,
    Xor
}

fn do_bitwise_op(device: &mut Device, operand: Operand8, operation: BitwiseOp) -> u32 {
    let a = A.get(device);
    let op = operand.get(device);

    let op_f = match operation {
        BitwiseOp::And => u8::bitand,
        BitwiseOp::Or => u8::bitor,
        BitwiseOp::Xor => u8::bitxor
    };

    let res = op_f(a, op);

    device.regs.clear_flag(StatusFlag::C);
    device.regs.clear_flag(StatusFlag::H);
    device.regs.clear_flag(StatusFlag::N);
    device.regs.set_flag_to(StatusFlag::Z, res == 0);

    device.regs.a = res;

    if operand.is_memref() || operand.is_immediate() {
        8
    } else {
        4
    }
}

pub fn and_with_a(device: &mut Device, operand: Operand8) -> u32 {
    do_bitwise_op(device, operand, BitwiseOp::And)
}

pub fn or_with_a(device: &mut Device, operand: Operand8) -> u32 {
    do_bitwise_op(device, operand, BitwiseOp::Or)
}

pub fn xor_with_a(device: &mut Device, operand: Operand8) -> u32 {
    do_bitwise_op(device, operand, BitwiseOp::Xor)
}

pub fn test_bit(device: &mut Device, operand: Operand8, n: u8) -> u32 {
    let op = operand.get(device);
    device.regs.set_flag_to(StatusFlag::Z, !op.get_bit(n));

    if operand.is_memref() {
        12
    } else {
        8
    }
}

pub fn shift_left_logical(device: &mut Device, operand: Operand8) -> u32 {
    let mut op = operand.get(device);
    let last_bit = op.get_bit(7);

    op = (op << 1).set_bit(0);

    device.regs.set_flag_to(StatusFlag::C, last_bit);
    device.regs.set_flag_to(StatusFlag::Z, op == 0);
    device.regs.clear_flag(StatusFlag::H);

    operand.set(device, op);

    if operand.is_memref() {
        12
    } else {
        8
    }
}

enum Bit7Value {
    Zero,
    Unchanged
}

fn shift_right_internal(device: &mut Device, operand: Operand8, bit_7: Bit7Value) -> u32 {
    let mut op = operand.get(device);
    let first_bit = op.get_bit(0);
    let last_bit = op.get_bit(7);

    op = (op >> 1).set_bit_to(
        7,
        match bit_7 {
            Bit7Value::Zero => false,
            Bit7Value::Unchanged => last_bit
        }
    );

    device.regs.set_flag_to(StatusFlag::C, first_bit);
    device.regs.set_flag_to(StatusFlag::Z, op == 0);
    device.regs.clear_flag(StatusFlag::H);

    operand.set(device, op);

    if operand.is_memref() {
        12
    } else {
        8
    }
}

pub fn shift_right_logical(device: &mut Device, operand: Operand8) -> u32 {
    shift_right_internal(device, operand, Bit7Value::Zero)
}

pub fn shift_right(device: &mut Device, operand: Operand8) -> u32 {
    shift_right_internal(device, operand, Bit7Value::Unchanged)
}

pub fn shift_left(device: &mut Device, operand: Operand8) -> u32 {
    let mut op = operand.get(device);
    let last_bit = op.get_bit(7);

    op = op << 1;

    device.regs.set_flag_to(StatusFlag::C, last_bit);
    device.regs.set_flag_to(StatusFlag::Z, op == 0);
    device.regs.clear_flag(StatusFlag::H);

    operand.set(device, op);

    if operand.is_memref() {
        12
    } else {
        8
    }
}

fn rotate_right_internal(device: &mut Device, operand: Operand8, modify_zero: bool) {
    let mut op = operand.get(device);
    let first_bit = op.get_bit(0);
    let carry_bit = device.regs.get_flag(StatusFlag::C);

    op = (op >> 1).set_bit_to(7, carry_bit);

    device.regs.set_flag_to(StatusFlag::C, first_bit);
    device.regs.clear_flag(StatusFlag::H);
    device.regs.clear_flag(StatusFlag::N);

    if modify_zero {
        device.regs.set_flag_to(StatusFlag::Z, op == 0);
    }

    operand.set(device, op);
}

pub fn rotate_right(device: &mut Device, operand: Operand8) -> u32 {
    rotate_right_internal(device, operand, true);
    if operand.is_memref() {
        12
    } else {
        8
    }
}

pub fn rotate_right_a(device: &mut Device) -> u32 {
    rotate_right_internal(device, Operand8::A, false);
    4
}

fn rotate_left_internal(device: &mut Device, operand: Operand8, modify_zero: bool) {
    let op = operand.get(device);
    let last_bit = op.get_bit(7);
    let carry_bit = device.regs.get_flag(StatusFlag::C);

    let rotated = (op << 1).set_bit_to(0, carry_bit);

    device.regs.set_flag_to(StatusFlag::C, last_bit);
    device.regs.clear_flag(StatusFlag::H);
    device.regs.clear_flag(StatusFlag::N);

    if modify_zero {
        device.regs.set_flag_to(StatusFlag::Z, rotated == 0);
    }

    operand.set(device, rotated);
}

pub fn rotate_left(device: &mut Device, operand: Operand8) -> u32 {
    rotate_left_internal(device, operand, true);
    if operand.is_memref() {
        12
    } else {
        8
    }
}

pub fn rotate_left_a(device: &mut Device) -> u32 {
    rotate_left_internal(device, Operand8::A, false);
    4
}

fn rotate_left_carry_internal(device: &mut Device, operand: Operand8, modify_zero: bool) {
    let op = operand.get(device);
    let last_bit = op.get_bit(7);
    let rotated = op.rotate_left(1);

    device.regs.set_flag_to(StatusFlag::C, last_bit);
    device.regs.clear_flag(StatusFlag::H);
    device.regs.clear_flag(StatusFlag::N);

    if modify_zero {
        device.regs.set_flag_to(StatusFlag::Z, rotated == 0);
    }

    operand.set(device, rotated);
}

pub fn rotate_left_carry_a(device: &mut Device) -> u32 {
    rotate_left_carry_internal(device, Operand8::A, false);
    4
}

pub fn rotate_left_carry(device: &mut Device, operand: Operand8) -> u32 {
    rotate_left_carry_internal(device, operand, true);
    if operand.is_memref() {
        16
    } else {
        8
    }
}

fn rotate_right_carry_internal(device: &mut Device, operand: Operand8, modify_zero: bool) {
    let op = operand.get(device);
    let first_bit = op.get_bit(0);
    let rotated = op.rotate_right(1);

    device.regs.set_flag_to(StatusFlag::C, first_bit);
    device
        .regs
        .set_flag_to(StatusFlag::Z, modify_zero && rotated == 0);
    device.regs.clear_flag(StatusFlag::H);

    operand.set(device, rotated);
}

pub fn rotate_right_carry_a(device: &mut Device) -> u32 {
    rotate_right_carry_internal(device, Operand8::A, false);
    4
}

pub fn rotate_right_carry(device: &mut Device, operand: Operand8) -> u32 {
    rotate_right_carry_internal(device, operand, true);
    if operand.is_memref() {
        16
    } else {
        8
    }
}

pub fn complement_a(device: &mut Device) -> u32 {
    let a = A.get(device);
    A.set(device, !a);
    4
}

pub fn clear_bit(device: &mut Device, operand: Operand8, bit_index: u8) -> u32 {
    let op = operand.get(device);
    operand.set(device, op.clear_bit(bit_index));

    if operand.is_memref() {
        16
    } else {
        8
    }
}

pub fn set_bit(device: &mut Device, operand: Operand8, bit_index: u8) -> u32 {
    let op = operand.get(device);
    operand.set(device, op.set_bit(bit_index));

    if operand.is_memref() {
        16
    } else {
        8
    }
}

pub fn swap_nibbles(device: &mut Device, operand: Operand8) -> u32 {
    let op = operand.get(device);
    let upper = op & 0b1111_0000;
    let lower = op & 0b0000_1111;
    let swapped = lower << 4 | upper >> 4;

    operand.set(device, swapped);
    device.regs.set_flag_to(StatusFlag::Z, swapped == 0);
    device.regs.clear_flag(StatusFlag::N);
    device.regs.clear_flag(StatusFlag::H);
    device.regs.clear_flag(StatusFlag::C);

    if operand.is_memref() {
        16
    } else {
        8
    }
}
