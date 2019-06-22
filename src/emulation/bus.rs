use crate::emulation::constants::*;
use crate::emulation::cartridge::{Cartridge};
use crate::emulation::device::{DeviceType};
use crate::emulation::internal_message::{InternalMessage};
use crate::emulation::interrupt::{InterruptRegisters, Interrupt};
use crate::emulation::address_mapper::{AddressMapper};
use crate::emulation::input::InputRegister;
use crate::emulation::serial::SerialRegister;
use crate::emulation::timers::{TimerRegister, TimerRegisters};
use crate::emulation::audio::controller::{AudioController, AudioRamLocation};
use crate::emulation::video::controller::{VideoController, VideoMemoryLocation};

#[derive(Debug)]
pub enum MemoryLocation {
  Invalid(u16),
  Ignored(u16),
  Bootrom(u8),
  RamBank0(u16),
  RamBankN(u16),
  Cartridge(u16),
  Joypad,
  Serial(SerialRegister),
  Timer(TimerRegister),
  Audio(AudioRamLocation),
  Video(VideoMemoryLocation),
  BootromUnmap,
  HighRam(u8),
  InterruptEnable,
  InterruptRequest
}

use crate::emulation::bus::MemoryLocation::*;

pub struct Bus {
  pub cartridge: Option<Box<Cartridge>>,
  pub bootrom: Option<Vec<u8>>,
  is_booting: bool,
  pub ram: Vec<u8>,
  pub high_ram: Vec<u8>,
  selected_ram_bank: usize,
  pub input: InputRegister,
  pub timer: TimerRegisters,
  audio: AudioController,
  pub video: VideoController,
  pub interrupt: InterruptRegisters,
  serial_buffer: u8
}

impl Bus {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> Bus {
    let ram = vec!(0; device.get_ram_size() as usize);
    let high_ram = vec!(0u8; 128);

    Bus {
      cartridge: None,
      bootrom,
      is_booting: true,
      ram,
      high_ram,
      selected_ram_bank: 1,
      input: InputRegister::new(),
      timer: TimerRegisters::new(),
      audio: AudioController::new(),
      video: VideoController::new(device),
      interrupt: InterruptRegisters::new(),
      serial_buffer: 0
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

  pub fn oam_dma_transfer(&mut self, address: u16) {
    for i in 0 .. 160 {
      self.video.oam[i as usize] = self.read_8(self.resolve_address(address + i));
    }
  }

  pub fn read_to_buffer_mut(&self, buffer: &mut [u8], address: u16, length: u16) {
    self.read_to_buffer(buffer, address, length);
  }
}

impl AddressMapper for Bus {
  type T = MemoryLocation;
  
  fn resolve_address(&self, address: u16) -> MemoryLocation {
    match address {
      0 ..= 255 if self.is_booting => Bootrom(address as u8),
      ROM_BANK_0_START ..= ROM_BANK_N_END |
      0xA000           ..= 0xBFFF         => Cartridge(address),
      RAM_BANK_0_START ..= RAM_BANK_0_END |
      ECHO_RAM_START   ..= ECHO_RAM_END   => RamBank0(address - RAM_BANK_0_START),
      RAM_BANK_N_START ..= RAM_BANK_N_END => RamBankN(address - RAM_BANK_N_START),
      VRAM_START       ..= VRAM_END       => Video(VideoMemoryLocation::Vram(address - VRAM_START)),
      OAM_START        ..= OAM_END        => Video(VideoMemoryLocation::Oam((address - 0xFE00) as u8)),
      0xFEA0           ..= 0xFEFF         => Ignored(address),
      0xFF00                              => Joypad,
      0xFF01                              => Serial(SerialRegister::Data),
      0xFF02                              => Serial(SerialRegister::Control),
      TIMER_IO_START   ..= TIMER_IO_END   => Timer(self.timer.resolve_address(address)),
      INTERRUPT_REQUEST                   => InterruptRequest,
      AUDIO_IO_START   ..= AUDIO_IO_END   => Audio(self.audio.resolve_address(address)),
      VIDEO_IO_START   ..= VIDEO_IO_END   => Video(self.video.resolve_address(address)),
      BOOTROM_UNMAP                       => BootromUnmap,
      0xFF7F                              => Ignored(address),
      HIGH_RAM_START   ..= HIGH_RAM_END   => HighRam((address - HIGH_RAM_START) as u8),
      INTERRUPT_ENABLE                    => InterruptEnable,
      otherwise => Invalid(otherwise)
    }
  }

  fn read_8(&self, location: MemoryLocation) -> u8 {
    //println!("R: {:?}", location);
    match location {
      Bootrom(offset) => if let Some(ref bootrom) = self.bootrom {bootrom[offset as usize]} else { 0 },
      RamBank0(offset) => self.ram[offset as usize],
      RamBankN(offset) => self.ram[self.selected_ram_bank * RAM_BANK_SIZE + (offset as usize)],
      Cartridge(address) => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.read_8(address)
        } else {
          panic!("Tried to read from cartridge when one isn't inserted.")
        }
      },
      Joypad => self.input.read_8(),
      Serial(_) => 0,
      Timer(register) => self.timer.read_8(register),
      InterruptRequest => self.interrupt.get_request(),
      Audio(audio_location) => self.audio.read_8(audio_location),
      Video(video_location) => self.video.read_8(video_location),
      BootromUnmap => self.is_booting as u8,
      HighRam(offset) => self.high_ram[offset as usize],
      InterruptEnable => self.interrupt.get_enable(),
      Ignored(_) => 0,
      Invalid(e) => panic!("Tried to read an invalid memory location: 0x{:X}", e)
    }
  }

  fn write_8(&mut self, location: MemoryLocation, value: u8) -> InternalMessage {
    //println!("W: {:?} <- {}", location, value);
    match location {
      Audio(audio_location) => self.audio.write_8(audio_location, value),
      Video(video_location) => self.video.write_8(video_location, value),
      Serial(SerialRegister::Data) => {
        self.serial_buffer = value;
        InternalMessage::None
      },
      Serial(SerialRegister::Control) if value == 0x81 => {
        println!("Serial: {}", self.serial_buffer);
        // TODO: This should not be immediate.
        InternalMessage::TriggerInterrupt(Interrupt::EndOfSerialIO)
      },
      _ => {
        match location {
          Bootrom(_) => (),
          RamBank0(offset) => self.ram[offset as usize] = value,
          RamBankN(offset) => self.ram[self.selected_ram_bank * RAM_BANK_SIZE + (offset as usize)] = value,
          Cartridge(address) => {
            match self.cartridge {
              Some(ref mut cartridge) => cartridge.write_8(address, value),
              None => panic!("Tried to write to cartridge when one isn't inserted.")
            }
          },
          Joypad => self.input.write_8(value),
          Serial(SerialRegister::Control) => {
            println!("Unknown serial control command: {}", value);
          } 
          Timer(register) => self.timer.write_8(register, value),
          InterruptRequest => self.interrupt.set_request(value),
          BootromUnmap => self.is_booting = value != 1, 
          HighRam(offset) => {
            //println!("high ram: {} <- {}", offset, value);
            self.high_ram[offset as usize] = value
          },
          InterruptEnable => self.interrupt.set_enable(value),
          Ignored(_) => (),
          Invalid(e) => panic!("Tried to write to an invalid memory location: 0x{:X}", e),
          _ => unreachable!("This cannot happen.") 
        };
        InternalMessage::None
      }
    }
  }
}
