
pub const RAM_BANK_SIZE: usize = 4096;
pub const VRAM_BANK_SIZE: usize = 8192;
pub const CARTRIDGE_RAM_BANK_SIZE: usize = 8192;
pub const ROM_BANK_SIZE: usize = 16384;

pub const GB_RAM_SIZE: usize = 8192;
pub const GB_VRAM_SIZE: usize = 8192;
pub const GBC_RAM_SIZE: usize = 32768;
pub const GBC_VRAM_SIZE: usize = 16384;

pub const ROM_BANK_0_START: u16    = 0x0000;
pub const ROM_BANK_0_END: u16      = ROM_BANK_0_START + ROM_BANK_SIZE as u16 - 1;
pub const ROM_BANK_N_START: u16    = 0x4000;
pub const ROM_BANK_N_END: u16      = ROM_BANK_N_START + ROM_BANK_SIZE as u16 - 1;

pub const VRAM_START: u16          = 0x8000;
pub const CARTRIDGE_RAM_START: u16 = 0xA000;

pub const RAM_BANK_0_START: u16    = 0xC000;
pub const RAM_BANK_0_END: u16      = RAM_BANK_0_START + RAM_BANK_SIZE as u16 - 1;

pub const RAM_BANK_N_START: u16    = 0xD000;
pub const RAM_BANK_N_END: u16      = RAM_BANK_N_START + RAM_BANK_SIZE as u16 - 1;

pub const ECHO_RAM_START: u16      = 0xE000;
pub const ECHO_RAM_END: u16        = 0xFDFF;
pub const OAM_START: u16           = 0xFE00;
pub const OAM_END: u16             = 0xFE9F;
pub const IO_START: u16            = 0xFF00;
pub const IO_END: u16              = 0xFF7F;