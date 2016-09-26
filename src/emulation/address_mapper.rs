
pub trait AddressMapper<T> {
  fn resolve_address(&self, address: u16) -> T;
  fn read_8(&self, location: T) -> u8;
  fn write_8(&mut self, location: T, value: u8);

  fn read_addr_8(&self, address: u16) -> u8 {
    let location = self.resolve_address(address);
    self.read_8(location)
  }

  fn write_addr_8(&mut self, address: u16, value: u8) {
    let location = self.resolve_address(address);
    self.write_8(location, value);
  }
}
