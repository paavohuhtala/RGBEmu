use crate::emulation::device::Device;
use crate::emulation::registers::StatusFlag;

pub fn enable_interrupts(device: &mut Device) -> u32 {
    device.interrupts_enabled = true;
    4
}

pub fn disable_interrupts(device: &mut Device) -> u32 {
    device.interrupts_enabled = false;
    4
}

// Based on https://ehaskins.com/2018-01-30%20Z80%20DAA/
pub fn bcd_correct_a(device: &mut Device) -> u32 {
    let mut correction = 0;

    let value = device.regs.a;
    let h = device.regs.get_flag(StatusFlag::H);
    let n = device.regs.get_flag(StatusFlag::N);
    let c = device.regs.get_flag(StatusFlag::C);

    if h || (!n && (value & 0xf) > 9) {
        correction += 0x6;
    }

    if c || (!n && value > 0x99) {
        correction += 0x60;
        device.regs.set_flag(StatusFlag::C);
    }

    let corrected_value_wide = (value as i16) + if n { -correction } else { correction };
    let corrected_value_byte = (corrected_value_wide & 0xFF) as u8;
    device.regs.a = corrected_value_byte;

    device.regs.clear_flag(StatusFlag::H);
    device
        .regs
        .set_flag_to(StatusFlag::Z, corrected_value_byte == 0);

    8
}
