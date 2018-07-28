
use emulation::constants::*;
use emulation::cartridge::{CartridgeMemory, CartridgeType};

pub trait Mapper {
  //const TYPE: MapperType;

  fn read_8(&self, cart: &CartridgeMemory, address: u16) -> u8;
  fn write_8(&mut self, cart: &mut CartridgeMemory, address: u16, value: u8);
}

#[derive(Debug, Clone, Copy)]
pub enum MapperType {
  RomOnly,
  MBC1,
  MBC2,
  MBC3
}

pub enum RomOnlyLocation {
  RomBank0(u16),
  RomBankN(u16),
  Ram(u16)
}

pub struct RomOnly { }

impl RomOnly {
  pub fn new() -> Box<RomOnly> {
    Box::new(RomOnly { })
  }

  fn resolve_address(&self, address: u16) -> RomOnlyLocation {
    use emulation::mappers::RomOnlyLocation::*;
    match address {
      ROM_BANK_0_START ... ROM_BANK_0_END => RomBank0(address),
      ROM_BANK_N_START ... ROM_BANK_N_END => RomBankN(address - ROM_BANK_N_START),
      CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => Ram(address - CARTRIDGE_RAM_START),
      _ => panic!("Tried to resolve an invalid address: ${:04x}", address)
    }
  }
}

impl Mapper for RomOnly {
  //const TYPE: MapperType = MapperType::RomOnly;

  fn read_8(&self, memory: &CartridgeMemory, address: u16) -> u8 {
    use emulation::mappers::RomOnlyLocation::*;
    let location = self.resolve_address(address);
    match location {
      RomBank0(offset) => memory.rom[offset as usize],
      RomBankN(offset) => memory.rom[ROM_BANK_SIZE + (offset as usize)],
      Ram(_) => {
        println!("Tried to read from cartridge RAM in a cart without a mapper! (0x{:04X})", address);
        0
      }
    }
  }

  fn write_8(&mut self, cart: &mut CartridgeMemory, address: u16, value: u8) {
    match address {
      _ => println!("Tried to write to a cartridge without a mapper! ({:?} <- {:X})", address, value)
    }
  }
}

impl Mapper {
  pub fn from_cartridge_type(cartridge_type: CartridgeType) -> Box<Mapper> {
    match cartridge_type.mapper {
      MapperType::RomOnly => RomOnly::new(),
      MapperType::MBC1 => MBC1::new(cartridge_type),
      _ => panic!("Tried to create create an unsupported mapper: {:?}", cartridge_type.mapper)
    }
  }
}

pub enum MBCMode {
  Ram,
  Rom
}

#[derive(Debug)]
pub struct MBC1 {
  cartridge_type: CartridgeType,
  ram_enabled: bool,
  rom_banking_mode: bool,
  rom_bank: u8,
  ram_bank_or_upper_rom_bits: u8
}

#[derive(Debug)]
enum MBC1Location {
  RomBank0(u32),
  RomBankN(u32),
  SelectedRamBank,
  RamEnable,
  Ram(u32)
}

impl MBC1 {
  pub fn new(cartridge_type: CartridgeType) -> Box<MBC1> {
    Box::new(MBC1 {
      cartridge_type,
      ram_enabled: false,
      rom_banking_mode: true,
      rom_bank: 1,
      ram_bank_or_upper_rom_bits: 0
    })
  }

  fn get_rom_bank(&self) -> u8 {
    if self.rom_banking_mode { 
      (self.ram_bank_or_upper_rom_bits & 0b11) << 5 | self.rom_bank
    } else {
      self.rom_bank
    }
  }

  fn get_ram_bank(&self) -> u8 {
    if self.rom_banking_mode { 0 } else { self.ram_bank_or_upper_rom_bits & 0b11 }
  }

  fn resolve_address(&self, address: u16) -> MBC1Location {
    use emulation::mappers::MBC1Location::*;

    let rom_bank = self.get_rom_bank();
    let ram_bank = self.get_ram_bank();

    match address {
      ROM_BANK_0_START ... ROM_BANK_0_END       => RomBank0((address - ROM_BANK_0_START) as u32),
      ROM_BANK_N_START ... ROM_BANK_N_END       => RomBankN(rom_bank as u32 * ROM_BANK_SIZE as u32 + (address - ROM_BANK_N_START) as u32),
      CARTRIDGE_RAM_START ... CARTRIDGE_RAM_END => Ram(ram_bank as u32 * RAM_BANK_SIZE as u32 + (address - CARTRIDGE_RAM_START) as u32),
      _ => panic!("Tried to read from an invalid address: ${:04x}", address)
    }
  }
}

impl Mapper for MBC1 {
  //const TYPE: MapperType = MapperType::MBC1;

  fn read_8(&self, memory: &CartridgeMemory, address: u16) -> u8 {
    use emulation::mappers::MBC1Location::*;
    let location = self.resolve_address(address);
    println!("Cart R: {:?}", location);
    match location {
      RomBank0(offs) => memory.rom[offs as usize],
      RomBankN(offs) => memory.rom[offs as usize],
      Ram(offs) if (offs as usize) < memory.ram.len() => memory.ram[offs as usize],
      Ram(_) => 0,
      _ => panic!("Tried to read from an invalid address: ${:04x}", address)
    }
  }

  fn write_8(&mut self, memory: &mut CartridgeMemory, address: u16, value: u8) {
    let ram_bank = self.get_ram_bank();

    println!("MBC W: 0x{:04X} <- 0x{:02X}", address, value);

    match address {
      0x0000 ... 0x1FFF => {
        self.ram_enabled = value & 0b1010 > 0;
      },
      0xA000 ... 0xBFFF => {
        let offset = ((address - 0xA000) * ram_bank as u16) as usize;

        if memory.ram.len() <= offset {
          println!("tried to write nonexistant RAM");
          return;
        }

        memory.ram[offset] = value;
      },
      0x2000 ... 0x3FFF => {
        let bank_bits = if value == 0 { 1 } else { value };
        self.rom_bank = bank_bits & 0b11111;
      },
      0x4000 ... 0x5FFF => {
        self.ram_bank_or_upper_rom_bits = value;
      },
      0x6000 ... 0x7FFF => {
        if value == 0 {
          self.rom_banking_mode = true;
        } else if value == 1 {
          self.rom_banking_mode = false;
        }
      }
      _ => panic!("unhandled location: {:?}", address)
    }

    println!("{:?}", self);
    //let location = self.resolve_address(address);
    //println!("Cart W: {:?} <- {}", location, value);
    /*match location {
      RomBank0(offs) => memory.rom[offs as usize] = value,
      RomBankN(offs) => memory.rom[offs as usize] = value,
      Ram(offs) if (offs as usize) < memory.ram.len() => memory.ram[offs as usize] = value,
      Ram(_) => println!("Tried to write to a non-existant RAM location: {:?}", location),
      _ => panic!("unhandled location: {:?}", location)
    }*/
  }
}