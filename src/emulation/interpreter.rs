use crate::emulation::address_mapper::Addressable;
use crate::emulation::bitutils::*;
use crate::emulation::device::{DebugState, Device, ReadWriteRegisters};
use crate::emulation::instruction::Instruction::*;
use crate::emulation::instruction::Operand16::*;
use crate::emulation::instruction::Operand8::*;
use crate::emulation::instruction_decoder::decode_instruction;
use crate::emulation::registers::StatusFlag;

use crate::emulation::instructions::*;

#[allow(dead_code)]
fn dump_memory(device: &Device) {
    let mut buffer = vec![0; 0xFFFF];
    device.bus.read_to_buffer(&mut buffer, 0, 0xFFFF);
    use std::io::Write;
    std::fs::File::create("dump.bin")
        .unwrap()
        .write_all(&buffer)
        .unwrap();
}

#[allow(dead_code)]
fn print_state(device: &Device) {
    println!("af = {:04X}", device.regs.af());
    println!("bc = {:04X}", device.regs.bc());
    println!("de = {:04X}", device.regs.de());
    println!("hl = {:04X}", device.regs.hl());
    println!("sp = {:04X}", device.regs.sp);
    println!("pc = {:04X}", device.regs.pc);
}

pub fn run_instruction(device: &mut Device) -> u32 {
    // print!("${:04x} ", device.regs.pc);
    let instruction = decode_instruction(device);
    // println!("{:?}", instruction);

    if let Unknown(opcode) = instruction {
        panic!("Unimplemented instruction: 0x{:04X}", opcode)
    }

    if let DebugState::HandlingBreakpoint = device.debug_state {
        println!("BEFORE: {:?}, f: {:08b}", device.regs, device.regs.f);
        println!("{:?}", instruction);
        device.pause();
    }

    match instruction {
        Nop => 4,

        //
        // 8-bit transfers
        //
        MoveOperand8 { to, from } => {
            let value = device.get_operand_8(from);
            device.set_operand_8(to, value);
            if from.is_memref() {
                8
            } else {
                4
            }
        }
        LoadAHigh(offset) => {
            let addr = 0xFF00 + (offset as u16);
            let value = device.bus.read_addr_8(addr);
            A.set(device, value);
            12
        }
        StoreAHighC => {
            let c = device.get_operand_8(C);
            let a = device.get_operand_8(A);
            let address = 0xFF00 + c as u16;
            device.write_addr_8(address, a);
            8
        }
        StoreAHigh(offset) => {
            let a = device.get_operand_8(A);
            let address = 0xFF00 + offset as u16;
            device.write_addr_8(address, a);
            12
        }
        LoadAIndirect(operand) => {
            let op = device.get_operand_16(operand);
            let value = device.bus.read_addr_8(op);
            device.set_operand_8(A, value);
            8
        }
        StoreAIndirectHLIncrement => {
            let hl = HL.get(device);
            let a = A.get(device);
            device.write_addr_8(hl, a);
            device.regs.set_hl(hl + 1);
            8
        }
        StoreAIndirectHLDecrement => {
            let hl = HL.get(device);
            let a = A.get(device);
            device.write_addr_8(hl, a);
            device.regs.set_hl(hl - 1);
            8
        }
        LoadAIndirectHLIncrement => {
            let hl = device.regs.hl();
            let a = device.bus.read_addr_8(hl);
            device.regs.a = a;
            device.regs.set_hl(hl.wrapping_add(1));
            8
        }
        LoadAIndirectHLDecrement => {
            let hl = device.regs.hl();
            let a = device.bus.read_addr_8(hl);
            device.regs.a = a;
            device.regs.set_hl(hl.wrapping_sub(1));
            8
        }
        StoreAIndirect(operand) => {
            let addr = operand.get(device);
            let a = device.regs.a;
            device.bus.write_addr_8(addr, a);
            8
        }
        StoreA(addr) => {
            let a = A.get(device);
            device.write_addr_8(addr, a);
            16
        }
        LoadA(addr) => {
            let value = device.bus.read_addr_8(addr);
            device.regs.a = value;
            16
        }

        //
        // 16-bit transfers
        //
        MoveImmediate16 { to, value } => move_immediate_16(device, to, value),
        Push(operand) => push_16(device, operand),
        Pop(operand) => pop_16(device, operand),
        StoreSP(address) => store_sp(device, address),
        MoveSPOffsetToHL(offset) => move_sp_offset_to_hl(device, offset),

        //
        // 8-bit ALU
        //
        IncrementOperand8(operand) => increment_operand_8(device, operand),
        DecrementOperand8(operand) => decrement_operand_8(device, operand),
        SubtractOperandFromA(operand) => subtract_operand_8_from_a(device, operand),
        AddOperandToA(operand) => add_operand_8_to_a(device, operand),
        AddOperandToACarry(operand) => add_operand_and_carry_to_a(device, operand),
        CompareOperandWithA(operand) => {
            let a = A.get(device);
            let op = operand.get(device);

            let CarryAddResult {
                result,
                carry,
                half_carry
            } = borrow_sub_8(a, op);

            device.regs.set_flag(StatusFlag::N);
            device.regs.set_flag_to(StatusFlag::Z, result == 0);
            device.regs.set_flag_to(StatusFlag::C, carry);
            device.regs.set_flag_to(StatusFlag::H, half_carry);

            if operand.is_memref() || operand.is_immediate() {
                8
            } else {
                4
            }
        }
        SetCarry => set_carry_flag(device),
        ComplementCarry => flip_carry_flag(device),

        //
        // 16-bit ALU
        //
        IncrementOperand16(operand) => increment_operand_16(device, operand),
        DecrementOperand16(operand) => decrement_operand_16(device, operand),
        AddOperandToHL(operand) => add_operand_to_hl(device, operand),

        //
        // Bitwise
        //
        AndOperandWithA(operand) => and_with_a(device, operand),
        OrOperandWithA(operand) => or_with_a(device, operand),
        XorOperandWithA(operand) => xor_with_a(device, operand),
        RotateLeft(operand) => rotate_left(device, operand),
        RotateLeftA => rotate_left_a(device),
        RotateRight(operand) => rotate_right(device, operand),
        RotateRightA => rotate_right_a(device),
        RotateLeftCarry(operand) => rotate_left_carry(device, operand),
        RotateLeftCarryA => rotate_left_carry_a(device),
        RotateRightCarry(operand) => rotate_right_carry(device, operand),
        RotateRightCarryA => rotate_right_carry_a(device),
        ShiftLeftArithmetic(operand) => shift_left(device, operand),
        ShiftLeftLogical(operand) => shift_left_logical(device, operand),
        ShiftRightLogical(operand) => shift_right_logical(device, operand),
        ComplementA => complement_a(device),
        TestBit(n, operand) => test_bit(device, operand, n),
        ClearBit(n, operand) => clear_bit(device, operand, n),
        SetBit(n, operand) => set_bit(device, operand, n),

        //
        // Control
        //
        Jump(addr) => {
            device.regs.pc = addr;
            16
        }
        RelativeJump(offset) => {
            device.regs.pc = (device.regs.pc as i32 + offset as i32) as u16;
            12
        }
        JumpToHL => jump_to_hl(device),
        ConditionalJump(condition, address) => conditional_jump(device, condition, address),
        ConditionalRelativeJump(condition, offset) => {
            conditional_relative_jump(device, condition, offset)
        }
        Call(addr) => call(device, addr),
        ConditionalCall(condition, addr) => conditional_call(device, condition, addr),
        Return => unconditional_return(device),
        ReturnFromInterrupt => return_from_interrupt(device),
        ConditionalReturn(condition) => conditional_return(device, condition),
        Restart(handler) => restart(device, handler),

        //
        // Miscancellous
        //
        DisableInterrupts => disable_interrupts(device),
        EnableInterrupts => enable_interrupts(device),
        BCDCorrectA => bcd_correct_a(device),
        Halt => {
            device.halt();
            4
        }

        _ => panic!("Unimplemented instruction: {:?}", instruction)
    }
}
