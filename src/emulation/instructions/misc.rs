
use emulation::device::{Device};

pub fn enable_interrupts(device: &mut Device) -> u32 {
  device.interrupts_enabled = true;
  4
}

pub fn disable_interrupts(device: &mut Device) -> u32 {
  device.interrupts_enabled = false;
  4
}
