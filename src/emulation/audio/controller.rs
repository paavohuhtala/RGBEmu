
use emulation::address_mapper::AddressMapper;

pub struct AudioController {
  ram: [u8; 0x2F]
}

impl AudioController {
  pub fn new() -> AudioController {
    AudioController {
      ram: [0; 0x2F]
    }
  }
}

type AudioRamOffset = u8;
pub type AudioRamLocation = Result<AudioRamOffset, u16>;

impl AddressMapper<AudioRamLocation> for AudioController {
  fn resolve_address(&self, address: u16) -> AudioRamLocation {
    if address < 0xFF10 || address > 0xFF3F {
      Err(address)
    } else {
      Ok((address - 0xFF10) as u8)
    }
  }

  fn read_8(&self, location: AudioRamLocation) -> u8 {
    match location {
      Ok(offset) => self.ram[offset as usize],
      Err(address) => panic!("Tried to read from invalid audio RAM address: ${:04x}", address)
    }
  }

  fn write_8(&mut self, location: AudioRamLocation, value: u8) {
    match location {
      Ok(offset) => self.ram[offset as usize] = value,
      Err(address) => panic!("Tried to write to invalid audio RAM address: ${:04x}", address)
    }
  }
}
