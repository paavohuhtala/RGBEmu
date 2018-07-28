
use emulation::bitutils::*;
use emulation::constants::*;
use emulation::device::Device;
use emulation::bus::Bus;
use emulation::video::controller::GbPalette;

pub trait RendererColor where Self: Sized {
  const TRANSPARENT: Self;

  fn from_rgb(r: u8, g: u8, b: u8) -> Self;

  fn from_rgb_tuple(colors: (u8, u8, u8)) -> Self {
    let (r, g, b) = colors;
    Self::from_rgb(r, g, b)
  }

  fn from_gb_color(color: u8) -> Self {
    match color {
      /*0 => Self::from_rgb_tuple(LIGHTEST_GREEN),
      1 => Self::from_rgb_tuple(LIGHT_GREEN),
      2 => Self::from_rgb_tuple(DARK_GREEN),
      3 => Self::from_rgb_tuple(DARKEST_GREEN),*/
      0 => Self::from_rgb(240, 240, 240),
      1 => Self::from_rgb(100, 100, 100),
      2 => Self::from_rgb(25, 25, 25),
      3 => Self::from_rgb(0, 0, 0),
      /*3 => Self::from_rgb(240, 240, 240),
      2 => Self::from_rgb(100, 100, 100),
      1 => Self::from_rgb(25, 25, 25),
      0 => Self::from_rgb(0, 0, 0),*/
      _ => panic!("Invalid color.")
    }
  }
}

pub trait RendererPalette<Color: RendererColor> where Self : Sized {
  fn from_colors(colors: [Color; 4]) -> Self;
  fn from_gb_palette(palette: &GbPalette, is_sprite: bool) -> Self {
    let color0 = if is_sprite {
      Color::TRANSPARENT
    } else {
      Color::from_gb_color(palette.get_color_0())
    };

    let colors = [
      color0,
      Color::from_gb_color(palette.get_color_1()),
      Color::from_gb_color(palette.get_color_2()),
      Color::from_gb_color(palette.get_color_3())
    ];

    Self::from_colors(colors)
  }
}

pub trait Renderer {
  fn present(&mut self);
  fn prepare_frame(&mut self, device: &Device);
  fn draw_scanline(&mut self, device: &Device, scanline: u8);
}

pub struct NullRenderer { }

impl Renderer for NullRenderer {
  fn present(&mut self) { }
  fn prepare_frame(&mut self, device: &Device) { }
  fn draw_scanline(&mut self, device: &Device, scanline: u8) { }
}

#[derive(Default, Clone, Copy)]
pub struct TileData(pub [u8; 16]);

impl TileData {
  fn unpack_row_to(&self, n: u8, buffer: &mut [u8], offset: usize) {
    let low = self.0[(n * 2) as usize];
    let high = self.0[(n * 2) as usize + 1];

    for i in 0 .. 8 {
      let low_bit = low.get_bit(7 - i);
      let high_bit = high.get_bit(7 - i);
      let index = (high_bit as u8) << 1 | (low_bit as u8);
      buffer[offset + i as usize] = index;
    }
  }

  pub fn unpack_to(&self, buffer: &mut [u8]) {
    for row in 0 .. 8 {
      self.unpack_row_to(row as u8, buffer, row * 8);
    }
  }
}

pub struct CommonRenderer { }

impl CommonRenderer {
  pub fn read_sprites(bus: &Bus, sprite_buffer: &mut [SpriteAttributes; 40]) {
    let mut sprite_attr_buffer = [0u8; 4];

    for i in 0 .. 40 {
      let addr = (0xFE00 + 4 * i) as u16;
      bus.read_to_buffer(&mut sprite_attr_buffer, addr, 4);
      sprite_buffer[i] = SpriteAttributes::from_bytes(sprite_attr_buffer);
    }
  }

  pub fn read_tiles(bus: &Bus, tiles_buffer: &mut [TileData; 256]) {
    let mut tile_buffer = [0u8; 16];

    let base_addr = bus.video.get_tile_pattern_table_addr() as usize;

    for i in 0 .. 256 {
      let addr = (base_addr + 16 * i) as u16;
      bus.read_to_buffer(&mut tile_buffer, addr, 16);
      tiles_buffer[i] = TileData(tile_buffer);
    }
  }

  pub fn read_background_tile_indices(bus: &Bus, bg_buffer: &mut [u8; 1024]) {
    bus.read_to_buffer(bg_buffer, bus.video.get_bg_tile_table_addr(), 1024);
  }

  pub fn read_window_tile_indices(bus: &Bus, bg_buffer: &mut [u8; 1024]) {
    bus.read_to_buffer(bg_buffer, bus.video.get_bg_tile_table_addr(), 1024);
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SpriteAttributes {
  pub y: u8,
  pub x: u8,
  pub pattern: u8,
  pub flags: u8
}

impl SpriteAttributes {
  pub fn from_bytes(bytes: [u8; 4]) -> SpriteAttributes {
    let y = bytes[0];
    let x = bytes[1];
    let pattern = bytes[2];
    let flags = bytes[3];

    SpriteAttributes { x, y, pattern, flags }
  }
}

pub mod sdl_renderer;