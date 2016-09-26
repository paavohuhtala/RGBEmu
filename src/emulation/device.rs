
use emulation::constants::*;
use emulation::registers::{Registers};
use emulation::address_mapper::AddressMapper;
use emulation::mmu::{MMU};
use emulation::instruction::{Instruction};

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
    Device {
      regs: Registers::new(),
      memory: MMU::new(device, bootrom),
      execution_state: ExecutionState::Halted
    }
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

  // Partially based on:
  // http://www.classiccmp.org/dunfield/r/8080.txt
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
