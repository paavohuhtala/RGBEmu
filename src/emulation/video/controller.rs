
use emulation::address_mapper::AddressMapper;

pub struct VideoController {
  ram: [u8; 0x2B]
}

impl VideoController {
  pub fn new() -> VideoController {
    VideoController {
      ram: [0; 0x2B]
    }
  }
}

type VideoRegisterOffset = u8;
pub type VideoRegisterLocation = Result<VideoRegisterOffset, u16>;

impl AddressMapper for VideoController {
  type T = VideoRegisterLocation;
  
  fn resolve_address(&self, address: u16) -> VideoRegisterLocation {
    if address < 0xFF40 || address > 0xFF6B {
      Err(address)
    } else {
      Ok((address - 0xFF40) as u8)
    }
  }

  fn read_8(&self, location: VideoRegisterLocation) -> u8 {
    match location {
      Ok(offset) => self.ram[offset as usize],
      Err(address) => panic!("Tried to read from invalid VRAM control register address: ${:04x}", address)
    }
  }

  fn write_8(&mut self, location: VideoRegisterLocation, value: u8) {
    match location {
      Ok(offset) => self.ram[offset as usize] = value,
      Err(address) => panic!("Tried to write to invalid VRAM control register address: ${:04x}", address)
    }
  }
}
