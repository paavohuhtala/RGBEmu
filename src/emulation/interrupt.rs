use crate::emulation::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    LCDVBlank,
    LCDController,
    TimerOverflow,
    EndOfSerialIO,
    JoyPad
}

impl Interrupt {
    pub const ALL_INTERRUPTS: [(Interrupt, InterruptRegisterFlags); 5] = [
        (
            Interrupt::LCDVBlank,
            InterruptRegisterFlags::LCDVBlankInterrupt
        ),
        (
            Interrupt::LCDController,
            InterruptRegisterFlags::LCDControllerInterrupt
        ),
        (
            Interrupt::TimerOverflow,
            InterruptRegisterFlags::TimerOverflowInterrupt
        ),
        (
            Interrupt::EndOfSerialIO,
            InterruptRegisterFlags::EndOfSerialIOInterrupt
        ),
        (Interrupt::JoyPad, InterruptRegisterFlags::JoyPadInterrupt)
    ];

    pub fn to_mask_bit(self) -> InterruptRegisterFlags {
        match self {
            Interrupt::LCDVBlank => InterruptRegisterFlags::LCDVBlankInterrupt,
            Interrupt::LCDController => InterruptRegisterFlags::LCDControllerInterrupt,
            Interrupt::TimerOverflow => InterruptRegisterFlags::TimerOverflowInterrupt,
            Interrupt::EndOfSerialIO => InterruptRegisterFlags::EndOfSerialIOInterrupt,
            Interrupt::JoyPad => InterruptRegisterFlags::JoyPadInterrupt
        }
    }

    pub fn get_handler_address(self) -> u16 {
        match self {
            Interrupt::LCDVBlank => INTERRUPT_HANDLER_VBLANK,
            Interrupt::LCDController => INTERRUPT_HANDLER_LCD,
            Interrupt::TimerOverflow => INTERRUPT_HANDLER_TIMER,
            Interrupt::EndOfSerialIO => INTERRUPT_HANDLER_SERIAL,
            Interrupt::JoyPad => INTERRUPT_HANDLER_JOYPAD
        }
    }
}

bitflags! {
  pub struct InterruptRegisterFlags: u8 {
    const LCDVBlankInterrupt     = 0b0000_0001;
    const LCDControllerInterrupt = 0b0000_0010;
    const TimerOverflowInterrupt = 0b0000_0100;
    const EndOfSerialIOInterrupt = 0b0000_1000;
    const JoyPadInterrupt        = 0b0001_0000;
  }
}

pub struct InterruptRegisters {
    enabled: InterruptRegisterFlags,
    requested: InterruptRegisterFlags
}

impl InterruptRegisters {
    pub fn new() -> InterruptRegisters {
        InterruptRegisters {
            enabled: InterruptRegisterFlags::empty(),
            requested: InterruptRegisterFlags::empty()
        }
    }

    pub fn get_enable(&self) -> u8 {
        self.enabled.bits()
    }

    pub fn set_enable(&mut self, value: u8) {
        self.enabled = InterruptRegisterFlags::from_bits(value).unwrap();
    }

    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        self.enabled.insert(interrupt.to_mask_bit());
    }

    pub fn disable_interrupt(&mut self, interrupt: Interrupt) {
        self.enabled.remove(interrupt.to_mask_bit());
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.requested.insert(interrupt.to_mask_bit());
    }

    pub fn unrequest_interrupt(&mut self, interrupt: Interrupt) {
        self.requested.remove(interrupt.to_mask_bit());
    }

    pub fn get_request(&self) -> u8 {
        self.requested.bits()
    }

    pub fn set_request(&mut self, value: u8) {
        self.requested = InterruptRegisterFlags::from_bits(value).unwrap();
    }

    pub fn handle_next_interrupt(&mut self) -> Option<Interrupt> {
        let enabled_requested = self.enabled & self.requested;

        for &(interrupt, bit) in &Interrupt::ALL_INTERRUPTS {
            if enabled_requested.contains(bit) {
                self.requested.remove(bit);
                return Some(interrupt);
            }
        }

        None
    }
}
