
use emulation::memory_location::CartridgeMemoryLocation;

pub struct Cartridge {
  buffer: [u8]
}

impl Cartridge {
  pub fn from_bytes() -> Option<Box<Cartridge>> {
    None
  }

  pub fn read8(&self, location: CartridgeMemoryLocation) -> u8 {
    unimplemented!()
  }

  pub fn write8(&self, location: CartridgeMemoryLocation, value: u8) {
    unimplemented!()
  }
}

struct CatridgeHeader {
    
}

trait CatridgeMapper {

}
