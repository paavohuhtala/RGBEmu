
use emulation::constants::*;
use emulation::cartridge::{Cartridge};
use emulation::device::{DeviceType};

use emulation::address_mapper::{AddressMapper};
use emulation::memory_location::*;
use emulation::memory_location::MemoryLocation::*;
use emulation::memory_location::CartridgeMemoryLocation::*;

use emulation::audio::controller::AudioController;
use emulation::video::controller::VideoController;

pub struct MMU {
  pub cartridge: Option<Box<Cartridge>>,
  pub bootrom: Option<Vec<u8>>,
  is_booting: bool,
  pub ram: Vec<u8>,
  pub high_ram: Vec<u8>,
  vram: Vec<u8>,
  selected_ram_bank: usize,
  selected_vram_bank: usize,
  audio: AudioController,
  video: VideoController
}

impl MMU {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> MMU {
    let ram = vec!(0; device.get_ram_size() as usize);
    let high_ram = vec!(0u8; 128);
    let vram = vec!(0; device.get_vram_size() as usize);

    MMU {
      cartridge: None,
      bootrom,
      is_booting: true,
      ram,
      high_ram,
      vram,
      selected_ram_bank: 1,
      selected_vram_bank: 0,
      audio: AudioController::new(),
      video: VideoController::new()
    }
  }

  pub fn read_to_buffer(&self, buffer: &mut [u8], address: u16, length: u16) {
    if buffer.len() < (length as usize) {
      panic!("Tried to read {} bytes to a buffer sized {}.", buffer.len(), length);
    }

    for i in 0 .. length {
      buffer[i as usize] = self.read_8(self.resolve_address(address + i));
    }
  }
}

impl AddressMapper for MMU {
  type T = MemoryLocation;
  
  fn resolve_address(&self, address: u16) -> MemoryLocation {
    match address {
      0 ... 255 if self.is_booting => Bootrom(address as u8),
      ROM_BANK_0_START ... ROM_BANK_0_END => CartridgeLocation(RomBank0(address - ROM_BANK_0_START)),
      ROM_BANK_N_START ... ROM_BANK_N_END => CartridgeLocation(RomBankN(address - ROM_BANK_N_START)),
      RAM_BANK_0_START ... RAM_BANK_0_END => RamBank0(address - RAM_BANK_0_START),
      RAM_BANK_N_START ... RAM_BANK_N_END => RamBankN(address - RAM_BANK_N_START),
      VRAM_START       ... VRAM_END       => Vram(address - VRAM_START),
      AUDIO_IO_START   ... AUDIO_IO_END   => Audio(self.audio.resolve_address(address)),
      VIDEO_IO_START   ... VIDEO_IO_END   => Video(self.video.resolve_address(address)),
      HIGH_RAM_START   ... HIGH_RAM_END   => HighRam((address - HIGH_RAM_START) as u8),
      otherwise => MemoryLocation::Invalid(otherwise)
    }
  }

  fn read_8(&self, location: MemoryLocation) -> u8 {
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
      },
      MemoryLocation::Audio(audio_location) => self.audio.read_8(audio_location),
      MemoryLocation::Video(video_location) => self.video.read_8(video_location),
      MemoryLocation::HighRam(offset) => self.high_ram[offset as usize],
      MemoryLocation::Invalid(e) => panic!("Tried to read an invalid memory location: 0x{:X}", e)
    }
  }

  fn write_8(&mut self, location: MemoryLocation, value: u8) {
    println!("${:?} = 0x{:x}", location, value);
    match location {
      MemoryLocation::Bootrom(_) => (),
      MemoryLocation::RamBank0(offset) => self.ram[offset as usize] = value,
      MemoryLocation::RamBankN(offset) => self.ram[self.selected_ram_bank * RAM_BANK_SIZE + (offset as usize)] = value,
      MemoryLocation::Vram(offset) => self.vram[self.selected_vram_bank * VRAM_BANK_SIZE + (offset as usize)] = value,
      MemoryLocation::CartridgeLocation(cartridge_location) => {
        match self.cartridge {
          Some(ref mut cartridge) => cartridge.write_8(cartridge_location, value),
          None => panic!("Tried to read from cartridge when one isn't inserted.")
        }
      },
      MemoryLocation::Audio(audio_location) => self.audio.write_8(audio_location, value),
      MemoryLocation::Video(video_location) => self.audio.write_8(video_location, value),
      MemoryLocation::HighRam(offset) => self.high_ram[offset as usize] = value,
      MemoryLocation::Invalid(e) => panic!("Tried to write to an invalid memory location: 0x{:X}", e)
    }
  }
}
