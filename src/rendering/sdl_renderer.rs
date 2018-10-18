use emulation::constants::*;
use rendering::*;

use sdl2;
use sdl2::render::{Texture, Canvas};
use sdl2::rect::Rect;
use sdl2::pixels::*;
use sdl2::surface::*;

pub struct SdlRendererContext<'a> {
  pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
  pub texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>
}

impl<'a> SdlRendererContext<'a> {
  pub fn new(canvas: sdl2::render::Canvas<sdl2::video::Window>, texture_creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> SdlRendererContext<'a> {
    SdlRendererContext { canvas, texture_creator }
  }
}

type SdlColor = sdl2::pixels::Color;
type SdlPalette = sdl2::pixels::Palette;
type SdlSurface<'a> = sdl2::surface::Surface<'a>;

impl RendererColor for SdlColor {
  const TRANSPARENT: SdlColor =  Color { r: 0, g: 0, b: 0, a :0 };

  fn from_rgb(r: u8, g: u8, b: u8) -> SdlColor {
    Color::RGB(r, g, b)
  }
}

impl RendererPalette<SdlColor> for SdlPalette {
  fn from_colors(colors: [Color; 4]) -> SdlPalette {
    let mut palette_colors = [Color::RGB(255, 0, 255); 256];
    palette_colors[0] = colors[0];
    palette_colors[1] = colors[1];
    palette_colors[2] = colors[2];
    palette_colors[3] = colors[3];
    Palette::with_colors(&palette_colors).unwrap()
  }
}

pub struct SdlRendererState<'a> {
  pub background_buffer: SdlSurface<'a>,
  pub window_buffer: SdlSurface<'a>,
  pub background_palette: SdlPalette,
  pub tile_cache: SdlSurface<'a>,
  pub sprites: [SpriteAttributes; 40],
  pub tile_patterns: [TileData; 256],
  pub background_tiles: [u8; 32 * 32],
  pub window_tiles: [u8; 32 * 32],
  temp_tile_buffer: SdlSurface<'a>
}

impl<'a> SdlRendererState<'a> {
  pub fn new() -> SdlRendererState<'a> {
    let background_buffer = Surface::new(256, 256, PixelFormatEnum::RGB24).unwrap();
    let window_buffer = Surface::new(256, 256, PixelFormatEnum::RGB24).unwrap();
    let background_palette = Palette::from_colors([Color::RGB(255, 0, 255); 4]);
    let tile_cache = Surface::new(16 * 8, 16 * 8, PixelFormatEnum::RGB24).unwrap();
    let sprites = [SpriteAttributes::default(); 40];
    let tile_patterns = [TileData::default(); 256];
    let background_tiles = [0u8; 32 * 32];
    let window_tiles = [0u8; 32 * 32];
    let temp_tile_buffer = Surface::new(8, 8, PixelFormatEnum::Index8).unwrap();

    SdlRendererState {
      background_buffer,
      window_buffer,
      background_palette,
      tile_cache,
      sprites,
      tile_patterns,
      background_tiles,
      window_tiles,
      temp_tile_buffer
    }
  }

  pub fn get_window_tile(&self, x: u8, y: u8) -> u8 {
    self.window_tiles[(y as usize) * 32 + (x as usize)]
  }

  pub fn get_background_tile(&self, x: u8, y: u8) -> u8 {
    self.background_tiles[(y as usize) * 32 + (x as usize)]
  }

  pub fn get_tile_rect(&self, tile_id: u8) -> Rect {
    let row = (tile_id / 16) as i32;
    let column = (tile_id % 16) as i32;
    Rect::new(column * 8, row * 8, 8, 8)
  }

  fn refresh_tile_maps(&mut self, device: &Device) {
    CommonRenderer::read_background_tile_indices(&device.bus, &mut self.background_tiles);
    CommonRenderer::read_window_tile_indices(&device.bus, &mut self.window_tiles);
  }

  fn refresh_tile_patterns(&mut self, device: &Device) {
    CommonRenderer::read_tiles(&device.bus, &mut self.tile_patterns);
  }

  fn refresh_sprites(&mut self, device: &Device) {
    CommonRenderer::read_sprites(&device.bus, &mut self.sprites);
  }
  
  fn refresh_palette(&mut self, device: &Device) {
    let palette = device.bus.video.background_palette;
    let sdl_palette = Palette::from_gb_palette(&palette, false);
    self.temp_tile_buffer.set_palette(&sdl_palette).unwrap();
  }

  fn refresh_tile_cache(&mut self) {
    self.temp_tile_buffer.fill_rect(None, Color::RGB(0, 0, 0)).unwrap();
    for y in 0 .. 16 {
      for x in 0 .. 16 {
        let tile = self.tile_patterns[y * 16 + x];

        self.temp_tile_buffer.with_lock_mut(|tile_buffer| -> () {
          tile.unpack_to(tile_buffer);
        });

        self.temp_tile_buffer.blit(None, self.tile_cache.as_mut(), Some(Rect::new((x * 8) as i32, (y * 8) as i32, 8, 8))).unwrap();
      }
    }
  }

  pub fn refresh(&mut self, device: &Device) {
    self.refresh_tile_maps(device);
    self.refresh_sprites(device);
    self.refresh_tile_patterns(device);
    self.refresh_tile_cache();
  }

  pub fn refresh_scanline(&mut self, device: &Device) {
    self.refresh_palette(device);
  } 
}

pub struct SdlRenderer<'a> {
  context: SdlRendererContext<'a>,
  debug_context: SdlRendererContext<'a>,
  pub state: SdlRendererState<'a>,
  screen_buffer_cpu: Surface<'a>,
  screen_buffer_gpu: Texture<'a>,
  debug_buffer_cpu: Canvas<Surface<'a>>,
  debug_buffer_gpu: Texture<'a>,
  debug_data: DebugData  
}

struct DebugData {
  pub scroll: (u8, u8),
  pub window: (u8, u8)
}

impl DebugData {
  fn empty() -> Self {
    DebugData {
      window: (0, 0),
      scroll: (0, 0)
    }
  }

  fn from_device(device: &Device) -> Self {
    DebugData {
      window: (device.bus.video.window_x, device.bus.video.window_y),
      scroll: (device.bus.video.scroll_x, device.bus.video.scroll_y)
    }
  }
}

impl<'a> SdlRenderer<'a> {
  pub fn new(context: SdlRendererContext<'a>, debug_context: SdlRendererContext<'a>) -> SdlRenderer<'a> {
    let state = SdlRendererState::new();
    let screen_buffer_cpu = Surface::new(160, 144, PixelFormatEnum::RGB24).unwrap();
    let screen_buffer_gpu = context.texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 160, 144).unwrap();

    let debug_buffer_cpu = Surface::new(500, 500, PixelFormatEnum::RGBA8888).unwrap().into_canvas().unwrap();
    let debug_buffer_gpu = debug_context.texture_creator.create_texture_streaming(PixelFormatEnum::RGBA8888, 500, 500).unwrap();
    let debug_data = DebugData::empty();
    SdlRenderer { context, debug_context, state, screen_buffer_cpu, screen_buffer_gpu, debug_buffer_cpu, debug_buffer_gpu, debug_data }
  }

  fn draw_background_buffer(&mut self) {
    for y in 0 .. 32 {
      for x in 0 .. 32 {
        let tile = self.state.get_background_tile(x, y);
        let row = (tile / 16) as i32;
        let column = (tile % 16) as i32;
        let cache_rect = Rect::new(column * 8, row * 8, 8, 8);
        let target_rect = Rect::new((x as i32) * 8, (y as i32) * 8, 8, 8);
        self.state.tile_cache.blit(Some(cache_rect), self.state.background_buffer.as_mut(), Some(target_rect)).unwrap();
      }
    }
  }

  fn draw_window_buffer(&mut self) {
    for y in 0 .. 32 {
      for x in 0 .. 32 {
        let tile = self.state.get_window_tile(x, y);
        let row = (tile / 16) as i32;
        let column = (tile % 16) as i32;
        let cache_rect = Rect::new(column * 8, row * 8, 8, 8);
        let target_rect = Rect::new((x as i32) * 8, (y as i32) * 8, 8, 8);
        self.state.tile_cache.blit(Some(cache_rect), self.state.window_buffer.as_mut(), Some(target_rect)).unwrap();
      }
    }
  }
}

impl<'a> Renderer for SdlRenderer<'a> {
  fn present(&mut self) {
    self.state.tile_cache.blit(None, self.debug_buffer_cpu.surface_mut(), None).unwrap();
    self.state.background_buffer.blit(None, self.debug_buffer_cpu.surface_mut(), Rect::new(0, 140, 256, 256)).unwrap();
    self.debug_buffer_cpu.set_draw_color(Color::RGB(255, 0, 0));
    self.debug_buffer_cpu.draw_rect(Rect::new(self.debug_data.scroll.0 as i32, self.debug_data.scroll.1 as i32 + 140, 160, 140)).unwrap();

    let screen_buffer = self.screen_buffer_cpu.without_lock().unwrap();

    self.screen_buffer_gpu.with_lock(None, |gpu_bytes, _| {
      gpu_bytes.copy_from_slice(screen_buffer);
    }).unwrap();

    let debug_buffer = self.debug_buffer_cpu.surface().without_lock().unwrap();

    self.debug_buffer_gpu.with_lock(None, |gpu_bytes, _| {
      gpu_bytes.copy_from_slice(debug_buffer);
    }).unwrap();

    self.context.canvas.set_draw_color(Color::RGB(255, 255, 255));
    self.context.canvas.clear();
    self.context.canvas.copy(&self.screen_buffer_gpu, None, None).unwrap();
    self.context.canvas.present();

    self.debug_context.canvas.clear();
    self.debug_context.canvas.copy(&self.debug_buffer_gpu, None, None).unwrap();
    self.debug_context.canvas.present();
  }

  fn draw_scanline(&mut self, device: &Device, scanline: u8) {
    self.state.refresh_scanline(device);

    let scanline = scanline as i32;
    let scroll_x = device.bus.video.scroll_x as i32;
    let scroll_y = device.bus.video.scroll_y as i32;

    let window_x = device.bus.video.window_x as i32;
    let window_y = device.bus.video.window_y as i32;

    let scanline_rect = Rect::new(scroll_x, scanline as i32 + scroll_y, 256, 1);
    self.state.background_buffer.blit(scanline_rect, self.screen_buffer_cpu.as_mut(), Rect::new(0, scanline as i32, 160, 1)).unwrap();

    if window_y >= scanline && window_y < scanline + 256 {
      let window_line_rect = Rect::new(0, scanline - window_y as i32, 256, 256);
      self.state.window_buffer.blit(window_line_rect, self.screen_buffer_cpu.as_mut(), Rect::new(window_x, window_y, 256, 256)).unwrap();
    }

    for sprite in self.state.sprites.iter() {
      if sprite.x == 0 {
        continue;
      }

      let x = sprite.x as i32;
      let y = sprite.y as i32;
      let width = 8;
      let height = device.bus.video.get_sprite_height() as i32;

      if scanline >= y && scanline < y + height {
        let tile_rect = self.state.get_tile_rect(sprite.pattern);
        let displayed_line = scanline - y;
        let source_rect = Rect::new(tile_rect.x(), tile_rect.y() + displayed_line, width as u32, 1);
        let target_rect = Rect::new(x - width, scanline as i32, width as u32, 1);
        self.state.tile_cache.blit(source_rect, self.screen_buffer_cpu.as_mut(), target_rect).unwrap();
      }
    }
  }

  fn prepare_frame(&mut self, device: &Device) {
    self.state.refresh(device);
    self.debug_data = DebugData::from_device(device);

    self.state.background_buffer.fill_rect(None, Color::RGB(255, 255, 255)).unwrap();
    self.screen_buffer_cpu.fill_rect(None, Color::RGB(255, 255, 255)).unwrap();
    self.state.background_buffer.fill_rect(None, Color::RGB(255, 255, 255)).unwrap();
    self.state.window_buffer.fill_rect(None, Color::RGBA(255, 255, 255, 0)).unwrap();

    if device.bus.video.is_bg_enabled() {
      self.draw_background_buffer();
    }

    if device.bus.video.is_window_enabled() {
      self.draw_window_buffer();
    }

    /*for y in 0 .. 32 {
      for x in 0 .. 32 {
        let tile = self.state.get_background_tile(x, y);
        let row = (tile / 16) as i32;
        let column = (tile % 16) as i32;
        let cache_rect = Rect::new(column * 8, row * 8, 8, 8);
        let target_rect = Rect::new((x as i32) * 8, (y as i32) * 8, 8, 8);
        self.state.tile_cache.blit(Some(cache_rect), self.state.background_buffer.as_mut(), Some(target_rect)).unwrap();
      }
    }

    let scroll_rect = Rect::new(device.bus.video.scroll_x as i32, device.bus.video.scroll_y as i32, 160, 144);
    self.state.background_buffer.blit(None, self.screen_buffer_cpu.as_mut(), Some(scroll_rect)).unwrap();*/

    /*if device.bus.video.is_window_enabled() {
      for y in 0 .. 32 {
        for x in 0 .. 32 {
          let tile = self.state.get_window_tile(x, y);
          let row = (tile / 16) as i32;
          let column = (tile % 16) as i32;
          let cache_rect = Rect::new(column * 8, row * 8, 8, 8);
          let target_rect = Rect::new((x as i32) * 8, (y as i32) * 8, 8, 8);
          self.state.tile_cache.blit(Some(cache_rect), self.state.window_buffer.as_mut(), Some(target_rect)).unwrap();
        }
      }

      let window_rect = Rect::new(device.bus.video.window_x as i32, device.bus.video.window_y as i32, 160, 144);
      self.state.window_buffer.blit(None, self.screen_buffer_cpu.as_mut(), Some(window_rect)).unwrap();
    }

    for sprite in self.state.sprites.iter() {
      if sprite.x == 0 { continue; }
      let tile_rect = self.state.get_tile_rect(sprite.pattern);
      let target_rect = Rect::new(sprite.x as i32 - 8, sprite.y as i32 - 16, 8, 8);
      self.state.tile_cache.blit(Some(tile_rect), self.screen_buffer_cpu.as_mut(), Some(target_rect)).unwrap();
    }

    self.screen_buffer_gpu.update(None, self.screen_buffer_cpu.without_lock().unwrap(), 160 * 3).unwrap();*/
  }
}
