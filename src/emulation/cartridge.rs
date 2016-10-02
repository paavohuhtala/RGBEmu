
use std;
use std::str::from_utf8;

use emulation::constants::*;
use emulation::address_mapper::{Addressable, AddressMapper};
use emulation::memory_location::CartridgeMemoryLocation;

pub struct Cartridge {
  buffer: [u8]
}

impl Cartridge {
  pub fn from_bytes() -> Option<Box<Cartridge>> {
    None
  }

  pub fn read_8(&self, location: CartridgeMemoryLocation) -> u8 {
    unimplemented!()
  }

  pub fn write_8(&self, location: CartridgeMemoryLocation, value: u8) {
    unimplemented!()
  }
}

pub enum CartridgeType {
  RomOnly = 0
}

pub struct CartridgeHeader {
    logo: [u8; 0x30],
    pub title: Option<String>,
    supports_gbc: bool,
    supports_sgb: bool,
    cartridge_type: CartridgeType,
    pub rom_size: usize,
    ram_size: usize,
    is_japanese: bool
}

impl CartridgeHeader {
  pub fn parse(bytes: &[u8]) -> CartridgeHeader {
    let mut logo = [0 as u8; 0x30];
    for (i, b) in bytes[0x104 .. 0x134].iter().enumerate() {
      logo[i] = *b;
    }

    let title_bytes = &bytes[0x134 .. 0x144];
    let first_zero = title_bytes.iter().position(|b| *b == 0).unwrap_or(15);
    let title = std::str::from_utf8(&title_bytes[.. first_zero]).map(|s|s.to_string()).ok();

    // 32 kb << N
    let rom_size = match bytes[0x148] {
      b @ 0 ... 7 => 0x8000 << b,
      0x52 => 72 * ROM_BANK_SIZE,
      0x53 => 80 * ROM_BANK_SIZE,
      0x54 => 96 * ROM_BANK_SIZE,
      invalid => panic!("Invalid ROM size: {}", invalid)
    };

    CartridgeHeader {
      logo: logo,
      title: title,
      supports_gbc: true,
      supports_sgb: false,
      cartridge_type: CartridgeType::RomOnly,
      rom_size: rom_size,
      ram_size: 0,
      is_japanese: false
    }
  }
}

trait CatridgeMapper {

}
