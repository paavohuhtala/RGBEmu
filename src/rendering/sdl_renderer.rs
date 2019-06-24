use crate::emulation::constants::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use crate::rendering::*;

use sdl2;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

pub struct SdlRendererContext {
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>
}

impl SdlRendererContext {
    pub fn new(
        canvas: Canvas<Window>,
        texture_creator: TextureCreator<WindowContext>
    ) -> SdlRendererContext {
        SdlRendererContext {
            canvas,
            texture_creator
        }
    }
}

type SdlColor = Color;
type SdlPalette = Palette;
type SdlSurface<'a> = Surface<'a>;

impl RendererColor for SdlColor {
    const TRANSPARENT: SdlColor = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0
    };

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
    pub background_palette: GbPalette,
    pub sprite_palette_0: GbPalette,
    pub sprite_palette_1: GbPalette,
    pub tile_cache: Box<[[u8; TILE_SIZE * TILE_SIZE]; 16 * 16]>,
    pub sprites: [SpriteAttributes; 40],
    pub tile_patterns: [TileData; 256],
    pub background_tiles: [u8; 32 * 32],
    pub window_tiles: [u8; 32 * 32]
}

impl<'a> SdlRendererState<'a> {
    pub fn new() -> SdlRendererState<'a> {
        let background_buffer = Surface::new(256, 256, PixelFormatEnum::RGB24).unwrap();
        let window_buffer = Surface::new(256, 256, PixelFormatEnum::RGB24).unwrap();
        let background_palette = GbPalette(0x00);
        let sprite_palette_0 = GbPalette(0x00);
        let sprite_palette_1 = GbPalette(0x00);
        let tile_cache = Box::new([[0; TILE_SIZE * TILE_SIZE]; 16 * 16]);
        let sprites = [SpriteAttributes::default(); 40];
        let tile_patterns = [TileData::default(); 256];
        let background_tiles = [0u8; 32 * 32];
        let window_tiles = [0u8; 32 * 32];

        SdlRendererState {
            background_buffer,
            window_buffer,
            background_palette,
            sprite_palette_0,
            sprite_palette_1,
            tile_cache,
            sprites,
            tile_patterns,
            background_tiles,
            window_tiles
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

    fn refresh_palettes(&mut self, device: &Device) {
        self.background_palette = device.bus.video.background_palette;
        self.sprite_palette_0 = device.bus.video.sprite_palette_0;
        self.sprite_palette_1 = device.bus.video.sprite_palette_1;
    }

    fn refresh_tile_cache(&mut self) {
        for y in 0..16 {
            for x in 0..16 {
                let tile = self.tile_patterns[y * 16 + x];
                let offset = (y * 16) + x;
                tile.unpack_to(&mut self.tile_cache[offset]);
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
        self.refresh_palettes(device);
    }
}

pub struct SdlRenderer<'a> {
    context: SdlRendererContext,
    debug_context: SdlRendererContext,
    pub state: SdlRendererState<'a>,
    screen_buffer_cpu: Surface<'a>,
    screen_buffer_gpu: Texture,
    debug_buffer_cpu: Canvas<Surface<'a>>,
    debug_buffer_gpu: Texture,
    debug_data: DebugData
}

#[derive(Default)]
struct DebugData {
    pub scroll: (u8, u8),
    pub window: (u8, u8)
}

impl DebugData {
    fn from_device(device: &Device) -> Self {
        DebugData {
            window: (device.bus.video.window_x, device.bus.video.window_y),
            scroll: (device.bus.video.scroll_x, device.bus.video.scroll_y)
        }
    }
}

impl<'a> SdlRenderer<'a> {
    pub fn new(context: SdlRendererContext, debug_context: SdlRendererContext) -> SdlRenderer<'a> {
        let state = SdlRendererState::new();
        let screen_buffer_cpu =
            Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT, PixelFormatEnum::RGB24).unwrap();
        let screen_buffer_gpu = context
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();

        let debug_buffer_cpu = Surface::new(500, 500, PixelFormatEnum::RGBA8888)
            .unwrap()
            .into_canvas()
            .unwrap();
        let debug_buffer_gpu = debug_context
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA8888, 500, 500)
            .unwrap();
        let debug_data = DebugData::default();

        SdlRenderer {
            context,
            debug_context,
            state,
            screen_buffer_cpu,
            screen_buffer_gpu,
            debug_buffer_cpu,
            debug_buffer_gpu,
            debug_data
        }
    }
}

impl<'a> Renderer for SdlRenderer<'a> {
    fn present(&mut self) {
        self.state
            .background_buffer
            .blit(
                None,
                self.debug_buffer_cpu.surface_mut(),
                Rect::new(0, 140, 256, 256)
            )
            .unwrap();

        self.debug_buffer_cpu
            .set_draw_color(Color::RGB(255, 255, 255));

        self.debug_buffer_cpu
            .draw_rect(Rect::new(
                self.debug_data.scroll.0 as i32,
                self.debug_data.scroll.1 as i32 + 140,
                SCREEN_WIDTH,
                SCREEN_HEIGHT
            ))
            .unwrap();

        let screen_buffer = self.screen_buffer_cpu.without_lock().unwrap();

        self.screen_buffer_gpu
            .with_lock(None, |gpu_bytes, _| {
                gpu_bytes.copy_from_slice(screen_buffer);
            })
            .unwrap();

        let debug_buffer = self.debug_buffer_cpu.surface().without_lock().unwrap();

        self.debug_buffer_gpu
            .with_lock(None, |gpu_bytes, _| {
                gpu_bytes.copy_from_slice(debug_buffer);
            })
            .unwrap();

        self.context.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.context.canvas.clear();
        self.context
            .canvas
            .copy(&self.screen_buffer_gpu, None, None)
            .unwrap();
        self.context.canvas.present();

        self.debug_context.canvas.clear();
        self.debug_context
            .canvas
            .copy(&self.debug_buffer_gpu, None, None)
            .unwrap();
        self.debug_context.canvas.present();
    }

    fn draw_scanline(&mut self, device: &Device, scanline: u8) {
        self.state.refresh_scanline(device);

        let scanline = scanline as i32;
        let scroll_x = device.bus.video.scroll_x as i32;
        let scroll_y = device.bus.video.scroll_y as i32;

        let window_x = device.bus.video.window_x as i32;
        let window_y = device.bus.video.window_y as i32;

        let bg_enabled = device.bus.video.is_bg_enabled();
        let sprites_enabled = device.bus.video.are_sprites_enabled();

        let base_offset = (SCREEN_WIDTH * 3) as usize * scanline as usize;
        let pixels = self.screen_buffer_cpu.as_mut().without_lock_mut().unwrap();

        let scanline = scanline as usize;

        for x in 0..SCREEN_WIDTH as usize {
            if bg_enabled {
                let tile_x = ((x + scroll_x as usize) / TILE_SIZE) as u8;
                let tile_y = ((scanline + scroll_y as usize) / TILE_SIZE) as u8;

                let start_of_tile_x = (tile_x as usize) * TILE_SIZE;
                let start_of_tile_y = (tile_y as usize) * TILE_SIZE;

                let tile_rel_x = x - start_of_tile_x;
                let tile_rel_y = scanline - start_of_tile_y;

                let tile_index = self.state.get_background_tile(tile_x, tile_y);
                let tile_data = &self.state.tile_cache[tile_index as usize];

                let tile_pixel_data = tile_data[tile_rel_y * TILE_SIZE + tile_rel_x];
                let (r, g, b, _) = device
                    .bus
                    .video
                    .background_palette
                    .get_color(tile_pixel_data, false);

                let screen_offset = base_offset + x * 3;
                pixels[screen_offset + 0] = r;
                pixels[screen_offset + 1] = g;
                pixels[screen_offset + 2] = b;
            }
        }

        // TODO: Render window

        if sprites_enabled {
            let width = TILE_SIZE as usize;
            let height = device.bus.video.get_sprite_height() as usize;

            let mut rendered_sprites = 0;

            for sprite in self.state.sprites.iter() {
                if sprite.x == 0 {
                    continue;
                }

                let sprite_y = sprite.y as usize - 16;

                if scanline >= sprite_y && scanline < sprite_y + height {
                    rendered_sprites += 1;

                    let sprite_x = sprite.x as usize - width;
                    let sprite_rel_y = scanline - sprite_y;

                    let tile_data = &self.state.tile_cache[sprite.pattern as usize];

                    let palette = if sprite.flags & 0b0001_0000 != 0 {
                        self.state.sprite_palette_0
                    } else {
                        self.state.sprite_palette_1
                    };

                    for x in 0..TILE_SIZE {
                        let tile_pixel_data = tile_data[sprite_rel_y * TILE_SIZE + x];

                        let (r, g, b, should_render) = palette.get_color(tile_pixel_data, true);

                        if !should_render {
                            continue;
                        }

                        let screen_offset = base_offset + (sprite_x + x) * 3;

                        pixels[screen_offset + 0] = r;
                        pixels[screen_offset + 1] = g;
                        pixels[screen_offset + 2] = b;
                    }
                }
            }
        }
    }

    fn prepare_frame(&mut self, device: &Device) {
        self.state.refresh(device);
        self.debug_data = DebugData::from_device(device);

        self.state
            .background_buffer
            .fill_rect(None, Color::RGB(0, 0, 0))
            .unwrap();

        self.screen_buffer_cpu
            .fill_rect(None, Color::RGB(0, 0, 0))
            .unwrap();

        self.state
            .window_buffer
            .fill_rect(None, Color::RGBA(0, 0, 0, 0))
            .unwrap();
    }
}
