
use std;
use std::fmt;

use emulation::constants::*;
use emulation::memory_location::CartridgeMemoryLocation;
use emulation::mappers::{MapperType, Mapper, RomOnly};

pub struct Cartridge {
  pub memory: CartridgeMemory,
  pub mapper: Box<Mapper>,
  pub header: CartridgeHeader
}

pub struct CartridgeMemory {
  pub rom: Vec<u8>,
  pub ram: Vec<u8>
}

impl CartridgeMemory {
  pub fn new(rom_size: usize, ram_size: Option<usize>) -> CartridgeMemory {
    CartridgeMemory {
      rom: vec![0u8; rom_size],
      ram: vec![0u8; ram_size.unwrap_or(0 as usize)]
    }
  }
}

impl Cartridge {
  pub fn from_bytes(buffer: &[u8]) -> Option<Box<Cartridge>> {
    let mut header_bytes = vec![0u8; 0x14F];
    header_bytes.clone_from_slice(&buffer[ .. 0x14F]);

    let header = CartridgeHeader::parse(&header_bytes);
    let memory = CartridgeMemory::new(header.rom_size, header.ram_size);
    let mapper = Mapper::from_type(header.cartridge_type.mapper);
    let cartridge = Cartridge { header, memory, mapper };

    Some(Box::new(cartridge))
  }

  pub fn read_8(&self, location: CartridgeMemoryLocation) -> u8 {
    self.mapper.read_8(&self.memory, location)
  }

  pub fn write_8(&mut self, location: CartridgeMemoryLocation, value: u8) {
    self.mapper.write_8(&mut self.memory, location, value);
  }
}


#[derive(Debug, Clone, Copy)]
pub struct CartridgeType {
  mapper: MapperType,
  has_ram: bool,
  has_battery: bool
}

impl CartridgeType {
  pub fn default() -> CartridgeType {
    CartridgeType {
      mapper: MapperType::RomOnly,
      has_ram: false,
      has_battery: false
    }
  }
}

pub struct LogoWrapper([u8; 0x30]);

impl fmt::Debug for LogoWrapper {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
    }
}

#[derive(Debug)]
pub struct CartridgeHeader {
    logo: LogoWrapper,
    pub title: Option<String>,
    supports_gbc: bool,
    supports_sgb: bool,
    cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: Option<usize>,
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

    let ram_size = match bytes[0x149] {
      0x0 => None,
      0x1 => Some(CARTRIDGE_RAM_BANK_SIZE / 4),
      0x2 => Some(CARTRIDGE_RAM_BANK_SIZE),
      0x3 => Some(CARTRIDGE_RAM_BANK_SIZE * 4),
      0x4 => Some(CARTRIDGE_RAM_BANK_SIZE * 16),
      0x5 => Some(CARTRIDGE_RAM_BANK_SIZE * 8),
      invalid => panic!("Invalid RAM size: {}", invalid)
    };

    CartridgeHeader {
      logo: LogoWrapper(logo),
      title: title,
      supports_gbc: true,
      supports_sgb: false,
      cartridge_type: CartridgeType::default(),
      rom_size: rom_size,
      ram_size: ram_size,
      is_japanese: false
    }
  }
}

trait CatridgeMapper {

}
