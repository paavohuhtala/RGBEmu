
pub const GB_CYCLES_PER_SEC: u32 = 4194000;
pub const GB_FRAME_RATE: f64 = 59.7;

pub const RAM_BANK_SIZE: usize = 4096;
pub const VRAM_BANK_SIZE: usize = 8192;
pub const CARTRIDGE_RAM_BANK_SIZE: usize = 8192;
pub const ROM_BANK_SIZE: usize = 16384;

pub const GB_RAM_SIZE: usize = 8192;
pub const GB_VRAM_SIZE: usize = 8192;
pub const GBC_RAM_SIZE: usize = 32768;
pub const GBC_VRAM_SIZE: usize = 16384;

pub const ROM_BANK_0_START: u16    = 0x0000;
pub const ROM_BANK_0_END: u16      = 0x3FFF;
pub const ROM_BANK_N_START: u16    = 0x4000;
pub const ROM_BANK_N_END: u16      = 0x7FFF;

pub const VRAM_START: u16          = 0x8000;
pub const VRAM_END: u16            = 0x9FFF;

pub const CARTRIDGE_RAM_START: u16 = 0xA000;
pub const CARTRIDGE_RAM_END: u16   = 0xBFFF; 

pub const RAM_BANK_0_START: u16    = 0xC000;
pub const RAM_BANK_0_END: u16      = RAM_BANK_0_START + RAM_BANK_SIZE as u16 - 1;

pub const RAM_BANK_N_START: u16    = 0xD000;
pub const RAM_BANK_N_END: u16      = RAM_BANK_N_START + RAM_BANK_SIZE as u16 - 1;

pub const ECHO_RAM_START: u16      = 0xE000;
pub const ECHO_RAM_END: u16        = 0xFDFF;
pub const OAM_START: u16           = 0xFE00;
pub const OAM_END: u16             = 0xFE9F;
pub const IO_START: u16            = 0xFF00;
pub const TIMER_IO_START: u16      = 0xFF04;
pub const TIMER_IO_END: u16        = 0xFF07;
pub const INTERRUPT_REQUEST: u16   = 0xFF0F;
pub const AUDIO_IO_START: u16      = 0xFF10;
pub const AUDIO_IO_END: u16        = 0xFF3F;
pub const VIDEO_IO_START: u16      = 0xFF40;
pub const VIDEO_IO_END: u16        = 0xFF4F;
pub const BOOTROM_UNMAP: u16       = 0xFF50;
pub const IO_END: u16              = 0xFF7F;
pub const HIGH_RAM_START: u16      = 0xFF80;
pub const HIGH_RAM_END: u16        = 0xFFFE;
pub const INTERRUPT_ENABLE: u16    = 0xFFFF;


pub const INTERRUPT_HANDLER_VBLANK: u16 = 0x0040;
pub const INTERRUPT_HANDLER_LCD: u16    = 0x0048;
pub const INTERRUPT_HANDLER_TIMER: u16  = 0x0050;
pub const INTERRUPT_HANDLER_SERIAL: u16 = 0x0058;
pub const INTERRUPT_HANDLER_JOYPAD: u16 = 0x0060;

pub const DARKEST_GREEN:  (u8, u8, u8) = (15, 56, 15);
pub const DARK_GREEN:     (u8, u8, u8) = (48, 98, 48);
pub const LIGHT_GREEN:    (u8, u8, u8) = (139, 172, 15);
pub const LIGHTEST_GREEN: (u8, u8, u8) = (155, 188, 15);
