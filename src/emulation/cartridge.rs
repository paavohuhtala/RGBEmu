use std::fmt;

use crate::emulation::constants::*;
use crate::emulation::mappers::{Mapper, MapperType};

pub struct Cartridge {
    pub memory: CartridgeMemory,
    pub mapper: Box<dyn Mapper>,
    pub header: CartridgeHeader
}

pub struct CartridgeMemory {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>
}

impl CartridgeMemory {
    pub fn new(rom_size: usize, ram_size: usize) -> CartridgeMemory {
        CartridgeMemory {
            rom: vec![0u8; rom_size],
            ram: vec![0u8; ram_size]
        }
    }
}

impl Cartridge {
    pub fn from_bytes(buffer: &[u8]) -> Option<Box<Cartridge>> {
        let mut header_bytes = vec![0u8; 0x14F];
        header_bytes.clone_from_slice(&buffer[..0x14F]);

        let header = CartridgeHeader::parse(&header_bytes);

        let mut memory = CartridgeMemory::new(
            header.cartridge_type.rom_size,
            header.cartridge_type.ram_size
        );
        memory.rom.copy_from_slice(buffer);
        let mapper = Mapper::from_cartridge_type(header.cartridge_type);
        let cartridge = Cartridge {
            header,
            memory,
            mapper
        };

        Some(Box::new(cartridge))
    }

    pub fn read_8(&self, address: u16) -> u8 {
        self.mapper.read_8(&self.memory, address)
    }

    pub fn write_8(&mut self, address: u16, value: u8) {
        self.mapper.write_8(&mut self.memory, address, value);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CartridgeType {
    pub mapper: MapperType,
    pub has_ram: bool,
    pub has_battery: bool,
    pub rom_size: usize,
    pub ram_size: usize
}

impl CartridgeType {
    pub fn new(id: u8, rom_size_id: u8, ram_size_id: u8) -> CartridgeType {
        // 32 kb << N
        let rom_size = match rom_size_id {
            b @ 0..=7 => 0x8000 << b,
            0x52 => 72 * ROM_BANK_SIZE,
            0x53 => 80 * ROM_BANK_SIZE,
            0x54 => 96 * ROM_BANK_SIZE,
            invalid => panic!("Invalid ROM size: {}", invalid)
        };

        let ram_size = match ram_size_id {
            0x0 => 0,
            0x1 => CARTRIDGE_RAM_BANK_SIZE / 4,
            0x2 => CARTRIDGE_RAM_BANK_SIZE,
            0x3 => CARTRIDGE_RAM_BANK_SIZE * 4,
            0x4 => CARTRIDGE_RAM_BANK_SIZE * 16,
            0x5 => CARTRIDGE_RAM_BANK_SIZE * 8,
            invalid => panic!("Invalid RAM size: {}", invalid)
        };

        let (mapper, has_ram, has_battery) = match id {
            0x00 => (MapperType::RomOnly, false, false),
            0x01 => (MapperType::MBC1, false, false),
            0x02 => (MapperType::MBC1, true, false),
            0x03 => (MapperType::MBC1, true, true),
            0x05 => (MapperType::MBC2, false, false),
            0x06 => (MapperType::MBC2, true, true),
            0x08 => (MapperType::RomOnly, true, false),
            0x09 => (MapperType::RomOnly, true, true),
            _ => panic!("Unsupported mapper: 0x{:02x}", id)
        };

        CartridgeType {
            mapper,
            has_ram,
            has_battery,
            rom_size,
            ram_size
        }
    }
}

pub struct LogoWrapper([u8; 0x30]);

impl fmt::Debug for LogoWrapper {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
    }
}

#[derive(Debug, Clone)]
pub struct CartridgeHeader {
    //logo: LogoWrapper,
    pub title: Option<String>,
    pub supports_gbc: bool,
    pub supports_sgb: bool,
    pub cartridge_type: CartridgeType,
    pub is_japanese: bool
}

impl CartridgeHeader {
    pub fn parse(bytes: &[u8]) -> CartridgeHeader {
        /*let mut logo = [0 as u8; 0x30];
        for (i, b) in bytes[0x104 .. 0x134].iter().enumerate() {
          logo[i] = *b;
        }*/

        let title_bytes = &bytes[0x134..0x144];
        let first_zero = title_bytes.iter().position(|b| *b == 0).unwrap_or(15);
        let title = std::str::from_utf8(&title_bytes[..first_zero])
            .map(|s| s.to_string())
            .ok();

        let cartridge_type_id = bytes[0x147];
        let rom_size_id = bytes[0x148];
        let ram_size_id = bytes[0x149];
        let cartridge_type = CartridgeType::new(cartridge_type_id, rom_size_id, ram_size_id);

        println!("{:?}", cartridge_type);

        CartridgeHeader {
            //logo: LogoWrapper(logo),
            title: title,
            supports_gbc: false,
            supports_sgb: false,
            cartridge_type,
            is_japanese: false
        }
    }
}
