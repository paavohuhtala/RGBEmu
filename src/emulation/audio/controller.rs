use crate::emulation::address_mapper::AddressMapper;
use crate::emulation::internal_message::InternalMessage;

pub struct AudioController {
    ram: [u8; 0x30]
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController { ram: [0; 0x30] }
    }
}

type AudioRamOffset = u8;
pub type AudioRamLocation = Result<AudioRamOffset, u16>;

impl AddressMapper for AudioController {
    type T = AudioRamLocation;

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
            Err(address) => panic!(
                "Tried to read from invalid audio RAM address: ${:04x}",
                address
            )
        }
    }

    fn write_8(&mut self, location: AudioRamLocation, value: u8) -> InternalMessage {
        match location {
            Ok(offset) => self.ram[offset as usize] = value,
            Err(address) => panic!(
                "Tried to write to invalid audio RAM address: ${:04x}",
                address
            )
        }

        InternalMessage::None
    }
}
