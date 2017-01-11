
use emulation::constants::*;
use emulation::registers::{Registers};
use emulation::address_mapper::{AddressMapper};
use emulation::mmu::{MMU};
use emulation::instruction::{Instruction, Operand8, Operand16};
use emulation::instruction::Operand8::*;
use emulation::instruction::Operand16::*;
use emulation::instruction_decoder::*;
use emulation::interpreter;

pub enum DeviceType {
  GameBoy,
  GameBoyColor
}

pub struct DeviceInfo {
  pub ram_size: usize,
  pub vram_size: usize
}

impl DeviceType {
  pub fn get_device_info(&self) -> DeviceInfo {
    match *self {
      DeviceType::GameBoy => DeviceInfo {ram_size: GB_RAM_SIZE, vram_size: GB_VRAM_SIZE},
      DeviceType::GameBoyColor => DeviceInfo {ram_size: GBC_RAM_SIZE, vram_size: GBC_VRAM_SIZE}
    }
  }

  pub fn get_ram_size(&self) -> usize {
    self.get_device_info().ram_size
  }

  pub fn get_vram_size(&self) -> usize {
    self.get_device_info().vram_size
  }

  pub fn get_ram_bank_count(&self) -> usize {
    (self.get_device_info().ram_size / RAM_BANK_SIZE) as usize
  }

  pub fn get_vram_bank_count(&self) -> usize {
    (self.get_device_info().vram_size / VRAM_BANK_SIZE) as usize
  }
}

pub enum ExecutionState {
  Halted,
  Paused,
  Running
}

pub struct Device {
  pub regs: Registers,
  pub memory: MMU,
  pub execution_state: ExecutionState
}

impl Device {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> Device {
    let mut device = Device {
      regs: Registers::new(),
      memory: MMU::new(device, bootrom),
      execution_state: ExecutionState::Halted
    };

    if device.memory.bootrom.is_none() {
      device.simulate_bootrom();
    }

    device
  }

  pub fn simulate_bootrom(&mut self) {
    self.regs.pc = 0x100;
    self.regs.sp = 0xFFFE;
    self.regs.set_af(0x01B0);
    self.regs.set_bc(0x0013);
    self.regs.set_de(0x00D8);
    self.regs.set_hl(0x014d);
  }

  pub fn new_gbc(bootrom: Option<Vec<u8>>) -> Device { Device::new(DeviceType::GameBoyColor, bootrom) }

  pub fn read_next_byte(&mut self) -> u8 {
    let pc = self.regs.pc;
    self.regs.pc += 1;
    self.memory.read_8(self.memory.resolve_address(pc))
  }

  fn decode_next_instruction(&mut self) -> Instruction {
    decode_instruction(self)
  }

  pub fn run_cycle(&mut self) {
    interpreter::run_cycle(self);
  }
}

impl ReadOnlyByteStream for Device {
  fn read_next_byte(&mut self) -> u8 {
    self.read_next_byte()
  }

  fn get_stream_position(&self) -> u16 {
    self.regs.pc
  }
}

pub trait ReadWriteRegisters {
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