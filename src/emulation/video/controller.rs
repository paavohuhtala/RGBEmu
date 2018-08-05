
use emulation::internal_message::{InternalMessage, RendererMessage};
use emulation::interrupt::Interrupt;
use emulation::device::DeviceType;

bitflags! {
  #[derive(Default)]
  pub struct LCDControlRegister: u8 {
    const BGDisplay        = 0b0000_0001;
    const ObjDisplay       = 0b0000_0010;
    const ObjSize          = 0b0000_0100;
    const BgTileTable      = 0b0000_1000;
    const TilePatternTable = 0b0001_0000;
    const WindowDisplay    = 0b0010_0000;
    const WindowTileTable  = 0b0100_0000;
    const LcdPower         = 0b1000_0000;
  }
}

bitflags! {
  #[derive(Default)]
  pub struct LCDStatusRegister: u8 {
    const LcdControllerMode            = 0b0000_0011;
    const ScanlineCoincidence          = 0b0000_0100;
    const InterruptOnMode0             = 0b0000_1000;
    const InterruptOnMode1             = 0b0001_0000;
    const InterruptOnMode2             = 0b0010_0000;
    const ScanlineCoincidenceInterrupt = 0b0100_0000;
  }
}

bitfield! {
  #[derive(Default, Clone, Copy)]
  pub struct GbPalette(u8);
  impl Debug;
  pub get_color_3, _: 7,6;
  pub get_color_2, _: 5,4;
  pub get_color_1, _: 3,2;
  pub get_color_0, _: 1,0;
}

#[derive(Debug, Clone, Copy)]
pub enum VideoMemoryLocation {
  Vram(u16),
  Oam(u8),
  LCDControlRegister,
  LCDStatusRegister,
  ScrollY,
  ScrollX,
  CurrentScanline,
  ComparisonScanline,
  BackgroundPalette,
  SpritePalette0,
  SpritePalette1,
  WindowY,
  WindowX,
  DMATransferControl,
  VramBank
}

use emulation::video::controller::VideoMemoryLocation::*;

#[derive(Debug)]
enum RenderingMode {
  OamRead,
  VramRead,
  Hblank,
  Vblank
}

#[derive(Debug)]
pub struct RenderingState {
  mode: RenderingMode,
  line: u8,  
  clock: u32
}

impl RenderingState {
  pub fn new() -> RenderingState {
    RenderingState {
      mode: RenderingMode::OamRead,
      line: 0,
      clock: 0
    }
  }
}

pub struct VideoController {
  device_type: DeviceType,
  vram: Vec<u8>,
  pub oam: [u8; 160],
  vram_bank: u8,
  lcd_control: LCDControlRegister,
  lcd_status: LCDStatusRegister,
  pub scroll_y: u8,
  pub scroll_x: u8,
  comparison_scanline: u8,
  pub background_palette: GbPalette,
  pub sprite_palette_0: GbPalette,
  pub sprite_palette_1: GbPalette,
  pub window_y: u8,
  pub window_x: u8,
  rendering_state: RenderingState
}

impl VideoController {
  pub fn new(device: DeviceType) -> VideoController {
    VideoController {
      device_type: device,
      vram: vec!(0; device.get_vram_size() as usize),
      oam: [0u8; 160],
      vram_bank: 0,
      lcd_status: LCDStatusRegister::default(),
      lcd_control: LCDControlRegister::default(),
      scroll_y: 0,
      scroll_x: 0,
      comparison_scanline: 0,
      background_palette: GbPalette(0xFC),
      sprite_palette_0: GbPalette(0xFC),
      sprite_palette_1: GbPalette(0xFC),
      window_y: 0,
      window_x: 0,
      rendering_state: RenderingState::new()
    }
  }

  pub fn resolve_address(&self, address: u16) -> VideoMemoryLocation {
    match address {
       0xFF40 => LCDControlRegister,
       0xFF41 => LCDStatusRegister,
       0xFF42 => ScrollY,
       0xFF43 => ScrollX,
       0xFF44 => CurrentScanline,
       0xFF45 => ComparisonScanline,
       0xFF46 => DMATransferControl,
       0xFF47 => BackgroundPalette,
       0xFF48 => SpritePalette0,
       0xFF49 => SpritePalette1,
       0xFF4A => WindowY,
       0xFF4B => WindowX,
       0xFF4F => VramBank,
       _ => panic!("Invalid GPU address: ${:04x}", address)
    }
  }

  pub fn read_8(&self, location: VideoMemoryLocation) -> u8 {
    match location {
      Vram(offs) => self.vram[offs as usize],
      Oam(offs) => self.oam[offs as usize],
      LCDControlRegister => self.lcd_control.bits(),
      LCDStatusRegister => self.lcd_status.bits(),
      ScrollY => self.scroll_y,
      ScrollX => self.scroll_x,
      CurrentScanline => self.rendering_state.line,
      ComparisonScanline => self.comparison_scanline,
      BackgroundPalette => self.background_palette.0,
      SpritePalette0 => self.sprite_palette_0.0,
      SpritePalette1 => self.sprite_palette_1.0,
      WindowY => self.window_y,
      WindowX => self.window_x,
      DMATransferControl => 0,
      VramBank => self.vram_bank
    }
  }

  pub fn write_8(&mut self, location: VideoMemoryLocation, value: u8) -> InternalMessage {
    match location {
      Vram(offs) => self.vram[offs as usize] = value,
      Oam(offs) => self.oam[offs as usize] = value,
      LCDControlRegister => self.lcd_control = LCDControlRegister::from_bits(value).unwrap(),
      LCDStatusRegister => self.lcd_status = LCDStatusRegister::from_bits(value).unwrap(),
      ScrollY => self.scroll_y = value,
      ScrollX => self.scroll_x = value,
      CurrentScanline => self.rendering_state.line = value,
      ComparisonScanline => self.comparison_scanline = value,
      BackgroundPalette => self.background_palette = GbPalette(value),
      SpritePalette0 => self.sprite_palette_0 = GbPalette(value),
      SpritePalette1 => self.sprite_palette_1 = GbPalette(value),
      WindowY => self.window_y = value,
      WindowX => self.window_x = value,
      DMATransferControl => (),
      // TODO: handle special behavior here, or in the VRAM case?
      VramBank => self.vram_bank = value
    }

    match location {
      DMATransferControl => InternalMessage::DMATransfer { from: (value as u16) << 8 },
      _ => InternalMessage::None
    }
  }

  pub fn is_lcd_on(&self) -> bool {
    self.lcd_control.contains(LCDControlRegister::LcdPower)
  }

  pub fn is_window_enabled(&self) -> bool {
    self.lcd_control.contains(LCDControlRegister::WindowDisplay)
  }

  pub fn is_bg_enabled(&self) -> bool {
    self.lcd_control.contains(LCDControlRegister::BGDisplay)
  }

  pub fn get_tile_pattern_table_addr(&self) -> u16 {
    if self.lcd_control.contains(LCDControlRegister::TilePatternTable) {
      0x8000
    } else {
      0x8800
    }
  }

  pub fn get_bg_tile_table_addr(&self) -> u16 {
    if self.lcd_control.contains(LCDControlRegister::BgTileTable) {
      0x9C00
    } else {
      0x9800
    }
  }

  pub fn get_window_tile_table_addr(&self) -> u16 {
    if self.lcd_control.contains(LCDControlRegister::WindowTileTable) {
      0x9C00
    } else {
      0x9800
    }
  }

  pub fn get_sprite_height(&self) -> u8 {
    if self.lcd_control.contains(LCDControlRegister::ObjSize) {
      16
    } else {
      8
    }
  }
 
  // Timings and basic principle from
  // http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings
  pub fn update(&mut self, elapsed_clocks: u32) -> InternalMessage {
    self.rendering_state.clock = self.rendering_state.clock.saturating_add(elapsed_clocks);
    let clock = self.rendering_state.clock;
    
    match self.rendering_state.mode {
      RenderingMode::OamRead if clock >= 80 => {
        self.rendering_state.clock = 0;
        self.rendering_state.mode = RenderingMode::VramRead;
        InternalMessage::None
      },
      RenderingMode::VramRead if clock >= 172 => {
        self.rendering_state.clock = 0;

        // Handle interrupt here?
        self.rendering_state.mode = RenderingMode::Hblank;
        InternalMessage::RendererMessage(RendererMessage::RenderScanline(self.rendering_state.line))
      },
      RenderingMode::Hblank if clock >= 204 => {
        self.rendering_state.clock = 0;
        self.rendering_state.line += 1;

        if self.rendering_state.line == 144 {
          self.rendering_state.mode = RenderingMode::Vblank;
          InternalMessage::TriggerInterrupt(Interrupt::LCDVBlank)
        } else {
          self.rendering_state.mode = RenderingMode::OamRead;
          InternalMessage::None
        }
      },
      RenderingMode::Vblank if clock >= 456 => {
        self.rendering_state.clock = 0;
        self.rendering_state.line += 1;

        if self.rendering_state.line > 153 {
          self.rendering_state.mode = RenderingMode::OamRead;
          self.rendering_state.line = 0;
          InternalMessage::RendererMessage(RendererMessage::PrepareNextFrame)
        } else {
          InternalMessage::None
        }
      },
      _ => InternalMessage::None
    }
  }
}