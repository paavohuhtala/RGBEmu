
use crate::emulation::device::{Device, DeviceType};

pub fn get_device() -> Device {
  let mut device = Device::new(DeviceType::GameBoy, None);

  device.regs.set_af(0);
  device.regs.set_bc(0);
  device.regs.set_de(0);
  device.regs.set_hl(0);

  device
}