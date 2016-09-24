
use emulation::constants::*;
use emulation::cartridge::{Cartridge};
use emulation::device::{DeviceType};

use emulation::memory_location::*;
use emulation::memory_location::MemoryLocation::*;
use emulation::memory_location::CartridgeMemoryLocation::*;

pub struct MMU {
  cartridge: Option<Box<Cartridge>>,
  bootrom: Option<Vec<u8>>,
  is_booting: bool,
  ram: Vec<u8>,
  vram: Vec<u8>,
  selected_ram_bank: usize,
  selected_vram_bank: usize
}

impl MMU {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> MMU {
    let ram = vec!(0; device.get_ram_size() as usize);
    let vram = vec!(0; device.get_vram_size() as usize);

    MMU {
      cartridge: None,
      bootrom: bootrom,
      is_booting: true,
      ram: ram,
      vram: vram,
      selected_ram_bank: 1,
      selected_vram_bank: 0
    }
  }

  pub fn resolve_address(&self, address: u16) -> MemoryLocation {
    match address {
      0 ... 255 if self.is_booting => Bootrom(address as u8),
      ROM_BANK_0_START ... ROM_BANK_0_END => CartridgeLocation(RomBank0(address - ROM_BANK_0_START)),
      ROM_BANK_N_START ... ROM_BANK_N_END => CartridgeLocation(RomBankN(address - ROM_BANK_N_START)),
      RAM_BANK_0_START ... RAM_BANK_0_END => RamBank0(address - RAM_BANK_0_START),
      RAM_BANK_N_START ... RAM_BANK_N_END => RamBankN(address - RAM_BANK_N_START),
      VRAM_START       ... VRAM_END       => Vram(address - VRAM_START),
      otherwise => MemoryLocation::Invalid(otherwise)
    }
  }

  pub fn read_8(&self, location: MemoryLocation) -> u8 {
    match location {
      MemoryLocation::Bootrom(offset) => if let Some(ref bootrom) = self.bootrom {bootrom[offset as usize]} else {0},
      MemoryLocation::RamBank0(offset) => self.ram[offset as usize],
      MemoryLocation::RamBankN(offset) => self.ram[self.selected_ram_bank * RAM_BANK_SIZE + (offset as usize)],
      MemoryLocation::Vram(offset) => self.vram[self.selected_vram_bank * VRAM_BANK_SIZE + (offset as usize)],
      MemoryLocation::CartridgeLocation(cartridge_location) => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.read_8(cartridge_location)
        } else {
          panic!("Tried to read from cartridge when one isn't inserted.")
        }
      }
      MemoryLocation::Invalid(e) => panic!("Tried to read an invalid memory location: 0x{:X}", e)
    }
  }

  pub fn read_addr_8(&self, address: u16) -> u8 {
    self.read_8(self.resolve_address(address))
  }

  pub fn read_to_buffer(&self, buffer: &mut [u8], address: u16, length: u16) {
    if buffer.len() < (length as usize) {
      panic!("Tried to read {} bytes to a buffer sized {}.", buffer.len(), length);
    }

    for i in 0 .. length {
      buffer[i as usize] = self.read_8(self.resolve_address(address + i));
    }
  } 

  pub fn write_8(&mut self, location: MemoryLocation, value: u8) {
    match location {
      MemoryLocation::Bootrom(_) => (),
      MemoryLocation::RamBank0(offset) => self.ram[offset as usize] = value,
      MemoryLocation::RamBankN(offset) => self.ram[self.selected_ram_bank * RAM_BANK_SIZE + (offset as usize)] = value,
      MemoryLocation::Vram(offset) => self.vram[self.selected_vram_bank * VRAM_BANK_SIZE + (offset as usize)] = value,
      MemoryLocation::CartridgeLocation(cartridge_location) => {
        match self.cartridge {
          Some(ref cartridge) => cartridge.write_8(cartridge_location, value),
          None => panic!("Tried to read from cartridge when one isn't inserted.")
        }
      }
      MemoryLocation::Invalid(e) => panic!("Tried to write to an invalid memory location: 0x{:X}", e)
    }
  }

  pub fn write_addr_8(&mut self, address: u16, value: u8) {
    let location = self.resolve_address(address);
    self.write_8(location, value);
  }
}
