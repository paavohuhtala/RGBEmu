#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate rgbemu;
extern crate sdl2;
extern crate time;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

use std::thread::sleep;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::EventPump;

use rgbemu::rendering::sdl_renderer::*;
use rgbemu::rendering::*;

use rgbemu::emulation::cartridge::Cartridge;
use rgbemu::emulation::constants::{GB_CYCLES_PER_SEC, SCREEN_HEIGHT, SCREEN_WIDTH};
use rgbemu::emulation::device::Device;
use rgbemu::emulation::input::InputState;
use rgbemu::emulation::internal_message::RendererMessage::*;

fn get_input_state(event_pump: &EventPump) -> InputState {
    let sdl_state = event_pump.keyboard_state();

    InputState {
        left: sdl_state.is_scancode_pressed(Scancode::Left),
        right: sdl_state.is_scancode_pressed(Scancode::Right),
        up: sdl_state.is_scancode_pressed(Scancode::Up),
        down: sdl_state.is_scancode_pressed(Scancode::Down),
        a: sdl_state.is_scancode_pressed(Scancode::Z),
        b: sdl_state.is_scancode_pressed(Scancode::X),
        select: sdl_state.is_scancode_pressed(Scancode::Space),
        start: sdl_state.is_scancode_pressed(Scancode::Return)
    }
}

#[allow(dead_code)]
fn wait_keypress(stdin: &mut impl Read) -> std::io::Result<()> {
    stdin.read_exact(&mut [0u8; 2])
}

fn load_bootrom() -> Vec<u8> {
    let mut rom_buffer: Vec<u8> = Vec::new();
    File::open("DMG_ROM.bin")
        .unwrap()
        .read_to_end(&mut rom_buffer)
        .unwrap();
    rom_buffer
}

fn load_game(path: &str) -> Cartridge {
    let mut cartridge_data: Vec<u8> = Vec::new();
    File::open(path)
        .unwrap()
        .read_to_end(&mut cartridge_data)
        .unwrap();

    let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();
    cartridge
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum UseBootromSetting {
    UseBootrom,
    EmulateBootrom
}

fn create_device(use_bootrom: UseBootromSetting) -> Device {
    let bootrom = if use_bootrom == UseBootromSetting::UseBootrom {
        Some(load_bootrom())
    } else {
        None
    };

    Device::new_gb(bootrom)
}

fn create_renderer(context: &mut sdl2::Sdl) -> Result<SdlRenderer, Box<dyn Error>> {
    let video = context.video()?;
    let window = video
        .window("RGBEmu", SCREEN_WIDTH * 4, SCREEN_HEIGHT * 4)
        .position_centered()
        .build()?;
    let debug_window = video.window("RGBEmu video debugger", 600, 600).build()?;

    let renderer_canvas = window.into_canvas().software().build()?;
    let texture_creator = renderer_canvas.texture_creator();
    let renderer_context = SdlRendererContext::new(renderer_canvas, texture_creator);

    let debug_canvas = debug_window.into_canvas().software().build()?;
    let texture_creator = debug_canvas.texture_creator();
    let debug_window_context = SdlRendererContext::new(debug_canvas, texture_creator);

    Ok(SdlRenderer::new(renderer_context, debug_window_context))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut context = sdl2::init()?;
    let mut event_pump = context.event_pump()?;
    let mut renderer = create_renderer(&mut context)?;

    let cartridge = load_game("./test_roms/Tetris (World).gb");
    // let cartridge = load_game("./test_roms/pong.gb");
    // let cartridge = load_game("./cpu_instrs/individual/09-op r,r.gb");
    let mut device = create_device(UseBootromSetting::EmulateBootrom);
    device.bus.cartridge = Some(cartridge);

    let mut total_cycles = 0u32;
    let mut last_frame = Instant::now();

    let mut stdin = std::io::stdin();
    let is_stepping = false;

    'main_loop: loop {
        total_cycles += device.run_tick();

        if is_stepping {
            wait_keypress(&mut stdin)?;
        }

        while let Some(msg) = device.next_renderer_message() {
            match msg {
                PrepareNextFrame => {
                    renderer.prepare_frame(&device);
                }
                RenderScanline(n) => {
                    renderer.draw_scanline(&device, n);
                }
                PresentFrame => {
                    renderer.present();

                    //last_update = precise_time_ns() as f64 / 10e9;
                    //let spent = last_update - last_frame;
                    //println!("Frametime: {}s (fps: {})", spent, 1f64 / spent);

                    //last_frame = last_update;

                    let new_input_state = get_input_state(&event_pump);

                    for event in event_pump.poll_iter() {
                        match event {
                            Event::Quit { .. }
                            | Event::KeyDown {
                                keycode: Some(Keycode::Escape),
                                ..
                            } => break 'main_loop,
                            _ => ()
                        }
                    }

                    device.update_input(new_input_state);

                    let time_spent = Instant::now().duration_since(last_frame);
                    let expected_time = Duration::new(
                        0,
                        ((1f64 / GB_CYCLES_PER_SEC as f64) * total_cycles as f64 * 1e9) as u32
                    );

                    let sleep_time = if time_spent > expected_time {
                        Duration::new(0, 0)
                    } else {
                        expected_time - time_spent
                    };

                    last_frame = Instant::now();
                    total_cycles = 0;

                    // println!("FPS: {}", 1.0f64 / ((time_spent.subsec_nanos() as f64 + time_spent.as_secs() as f64 * 1e9) / 1e9));
                    // sleep(sleep_time);
                }
            }
        }
    }

    Ok(())
}
