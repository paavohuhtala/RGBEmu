
use emulation::audio::controller::AudioRamLocation;
use emulation::video::controller::VideoRegisterLocation;

#[derive(Debug)]
pub enum CartridgeMemoryLocation {
  RomBank0(u16),
  RomBankN(u16),
  CartridgeRam(u16)
}

#[derive(Debug)]
pub enum MemoryLocation {
  Invalid(u16),
  RamBank0(u16),
  RamBankN(u16),
  Vram(u16),
  CartridgeLocation(CartridgeMemoryLocation),
  Bootrom(u8),
  Audio(AudioRamLocation),
  Video(VideoRegisterLocation)
}
