
use crate::emulation::internal_message::{InternalMessage};

pub trait AddressMapper {
  type T;
  fn resolve_address(&self, address: u16) -> Self::T;
  fn read_8(&self, location: Self::T) -> u8;
  fn write_8(&mut self, location: Self::T, value: u8) -> InternalMessage;
}

pub trait Addressable {
  fn read_addr_8(&self, address: u16) -> u8;
  fn write_addr_8(&mut self, address: u16, value: u8) -> InternalMessage;
}

impl<T : AddressMapper> Addressable for T {
  fn read_addr_8(&self, address: u16) -> u8 {
    let location = self.resolve_address(address);
    self.read_8(location)
  }

  fn write_addr_8(&mut self, address: u16, value: u8) -> InternalMessage {
    let location = self.resolve_address(address);
    self.write_8(location, value)
  }
}
