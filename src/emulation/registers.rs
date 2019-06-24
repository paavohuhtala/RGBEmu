#[derive(Debug, Clone, PartialEq)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16
}

macro_rules! get_16 {
    ($_self: ident, $high: ident, $low: ident) => {
        ($_self.$high as u16) << 8 | ($_self.$low as u16)
    };
}

macro_rules! set_16 {
    ($_self: ident, $high: ident, $low: ident, $value: expr) => {
        $_self.$high = (($value >> 8) & 0xFF) as u8;
        $_self.$low = ($value & 0xFF) as u8;
    };
}

#[repr(u8)]
pub enum StatusFlag {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0x0,
            pc: 0x0
        }
    }

    pub fn get_flag(&self, flag: StatusFlag) -> bool {
        self.f & flag as u8 != 0
    }

    pub fn set_flag(&mut self, flag: StatusFlag) {
        self.f |= flag as u8
    }

    pub fn clear_flag(&mut self, flag: StatusFlag) {
        self.f &= !(flag as u8)
    }

    pub fn set_flag_to(&mut self, flag: StatusFlag, to: bool) {
        if to {
            self.set_flag(flag)
        } else {
            self.clear_flag(flag)
        }
    }

    pub fn af(&self) -> u16 {
        get_16!(self, a, f)
    }

    pub fn bc(&self) -> u16 {
        get_16!(self, b, c)
    }

    pub fn de(&self) -> u16 {
        get_16!(self, d, e)
    }

    pub fn hl(&self) -> u16 {
        get_16!(self, h, l)
    }

    pub fn set_af(&mut self, value: u16) {
        set_16!(self, a, f, value);
    }

    pub fn set_bc(&mut self, value: u16) {
        set_16!(self, b, c, value);
    }

    pub fn set_de(&mut self, value: u16) {
        set_16!(self, d, e, value);
    }

    pub fn set_hl(&mut self, value: u16) {
        set_16!(self, h, l, value);
    }
}
