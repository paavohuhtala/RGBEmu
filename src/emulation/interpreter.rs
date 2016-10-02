
use emulation::bitutils::*;
use emulation::device::Device;
use emulation::registers::StatusFlag;
use emulation::instruction_decoder::decode_instruction;
use emulation::instruction::{Operand8, Operand16, ConditionCode};
use emulation::instruction::Instruction::*;
use emulation::instruction::Operand8::*;
use emulation::instruction::Operand16::*;
use emulation::address_mapper::{AddressMapper, Addressable};

trait ReadWriteRegisters {
  fn get_operand_8(&self, operand: Operand8) -> u8;
  fn get_operand_16(&self, operand: Operand16) -> u16;
  fn set_operand_8(&mut self, operand: Operand8, value: u8);
  fn set_operand_16(&mut self, operand: Operand16, value: u16);
}

impl ReadWriteRegisters for Device {
  fn get_operand_8(&self, operand: Operand8) -> u8 {
    match operand {
      A => self.regs.a,
      B => self.regs.b,
      C => self.regs.c,
      D => self.regs.d,
      E => self.regs.e,
      H => self.regs.h,
      L => self.regs.l,
      MemoryReference => self.memory.read_8(self.memory.resolve_address(self.regs.hl())),
      Immediate(value) => value
    }
  }

  fn set_operand_8(&mut self, operand: Operand8, value: u8) {
    match operand {
      A => self.regs.a = value,
      B => self.regs.b = value,
      C => self.regs.c = value,
      D => self.regs.d = value,
      E => self.regs.e = value,
      H => self.regs.h = value,
      L => self.regs.l = value,
      MemoryReference => {
        let location = self.memory.resolve_address(self.regs.hl());
        self.memory.write_8(location, value)
      },
      Immediate(_) => panic!("Tried to set an immediate value")
    }
  }

  fn get_operand_16(&self, operand_16: Operand16) -> u16 {
    match operand_16 {
      BC => self.regs.bc(),
      DE => self.regs.de(),
      HL => self.regs.hl(),
      SP => self.regs.sp
    }
  }

  fn set_operand_16(&mut self, operand_16: Operand16, value: u16) {
    match operand_16 {
      BC => self.regs.set_bc(value),
      DE => self.regs.set_de(value),
      HL => self.regs.set_hl(value),
      SP => self.regs.sp = value
    }
  }
}

fn check_condition(device: &Device, condition: ConditionCode) -> bool {
  match condition {
    ConditionCode::Zero(status) => device.regs.get_flag(StatusFlag::Z) == status,
    ConditionCode::Carry(status) => device.regs.get_flag(StatusFlag::C) == status
  }
}

pub fn run_cycle(device: &mut Device) -> i32 {
  let instruction = decode_instruction(device);
  println!("{:?}", instruction);
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
    StoreAIndirectHLDecrement => {
      let a = device.get_operand_8(A);
      let hl = device.get_operand_16(HL);
      device.memory.write_addr_8(hl, a);
      device.set_operand_16(HL, hl.wrapping_sub(1));
      8
    },
    TestBit(n, operand) => {
      let op = device.get_operand_8(operand);
      device.regs.set_flag_to(StatusFlag::Z, get_nth_bit(op, n) == 0);
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
    StoreAHighC => {
      let c = device.get_operand_8(C);
      let a = device.get_operand_8(A);
      let address: u16 = 0xFF00 + c as u16;
      device.memory.write_addr_8(address, a);
      8
    },
    StoreAHigh(offset) => {
      let a = device.get_operand_8(A);
      let address: u16 = 0xFF00 + offset as u16;
      device.memory.write_addr_8(address, a);
      12
    }
    IncrementOperand8(operand) => {
      let inc = device.get_operand_8(operand).wrapping_add(1);
      device.set_operand_8(operand, inc);
      if operand.is_memref() {12} else {4}
    },
    LoadAIndirect(operand) => {
      let op = device.get_operand_16(operand);
      let value = device.memory.read_addr_8(op);
      device.set_operand_8(A, value);
      8
    },
    _ => panic!("Unimplemented instruction: {:?}", instruction)
  }
}
