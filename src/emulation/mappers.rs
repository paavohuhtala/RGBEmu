
use emulation::constants::*;
use emulation::cartridge::{CartridgeMemory};
use emulation::memory_location::*;

type Location = CartridgeMemoryLocation;

pub trait Mapper {
  const TYPE: MapperType;

  fn read_8(&self, cart: &CartridgeMemory, location: Location) -> u8;
  fn write_8(&mut self, cart: &mut CartridgeMemory, location: Location, value: u8);
}

#[derive(Debug, Clone, Copy)]
pub enum MapperType {
  RomOnly,
  MBC1,
  MBC2,
  MBC3
}

pub struct RomOnly { }

impl RomOnly {
  pub fn new() -> Box<RomOnly> {
    Box::new(RomOnly { })
  }
}

impl Mapper for RomOnly {
  const TYPE: MapperType = MapperType::RomOnly;

  fn read_8(&self, memory: &CartridgeMemory, location: Location) -> u8 {
    match location {
      CartridgeMemoryLocation::RomBank0(offset) => memory.rom[offset as usize],
      CartridgeMemoryLocation::RomBankN(offset) => memory.rom[ROM_BANK_SIZE + (offset as usize)],
      CartridgeMemoryLocation::CartridgeRam(offset) => {
        println!("Tried to read from cartridge RAM in a cart without a mapper! (0x{:04X})", offset);
        0
      }
    }
  }

  fn write_8(&mut self, cart: &mut CartridgeMemory, location: Location, value: u8) {
    match location {
      _ => println!("Tried to write to a cartridge without a mapper! ({:?} <- {:X})", location, value)
    }
  }
}

impl Mapper {
  pub fn from_type(mapper_type: MapperType) -> Box<Mapper> {
    match mapper_type {
      MapperType::RomOnly => RomOnly::new(),
      _ => panic!("Tried to create create an unsupported mapper: {:?}", mapper_type)
    }
  }
}