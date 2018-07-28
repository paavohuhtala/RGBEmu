
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![feature(trace_macros)] 
#![feature(log_syntax)]
#![feature(box_syntax)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bitfield;
extern crate time;
extern crate sdl2;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};
use std::io::prelude::*;
use std::io::Cursor;
use std::fs::File;
use std::thread::sleep;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;

mod rendering;
use rendering::*;
use rendering::sdl_renderer::*;

mod emulation;
use emulation::constants::*;
use emulation::internal_message::RendererMessage::*;
use emulation::device::Device;
use emulation::input::InputState;
use emulation::cartridge::{Cartridge};

use emulation::instruction_decoder::decode_instruction;

mod disassembler;

use time::{precise_time_ns};

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

fn main() {
  let context = sdl2::init().unwrap();
  let video = context.video().unwrap();
  let window = video.window("RGBEmu", 160 * 4, 144 * 4).position_centered().build().unwrap();
  let debug_window = video.window("RGBEmu video debugger", 600, 600).build().unwrap();

  let renderer_canvas = window.into_canvas().software().build().unwrap();
  let texture_creator = renderer_canvas.texture_creator(); 
  let renderer_context = SdlRendererContext::new(renderer_canvas, &texture_creator);

  let debug_canvas = debug_window.into_canvas().software().build().unwrap();
  let texture_creator = debug_canvas.texture_creator(); 
  let debug_window_context = SdlRendererContext::new(debug_canvas, &texture_creator);
  let mut sdl_renderer = SdlRenderer::new(renderer_context, debug_window_context);

  // let mut rom_buffer : Vec<u8> = vec!();
  // File::open("DMG_ROM.bin").unwrap().read_to_end(&mut rom_buffer).unwrap();

  let mut cartridge_data : Vec<u8> = vec!();
  File::open("./cpu_instrs/individual/04-op r,imm.gb").unwrap().read_to_end(&mut cartridge_data).unwrap();
  // File::open("./test_roms/Tetris (World).gb").unwrap().read_to_end(&mut cartridge_data).unwrap();

  // let mut device = Device::new_gb(Some(rom_buffer));
  let mut device = Device::new_gb(None);
  let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();
  println!("{:?}", cartridge.header);

  device.bus.cartridge = Some(cartridge);

  let mut event_pump = context.event_pump().unwrap();

  let mut total_cycles = 0u32;
  let mut last_frame = Instant::now();

  'main_loop: loop {
    total_cycles += device.run_tick();
    while let Some(msg) = device.next_renderer_message() {
      match msg {
        PrepareNextFrame => {
          sdl_renderer.prepare_frame(&device);
        }
        RenderScanline(n) => {
          sdl_renderer.draw_scanline(&device, n);
        }
        PresentFrame => {
          sdl_renderer.present();

          //last_update = precise_time_ns() as f64 / 10e9;
          //let spent = last_update - last_frame;
          //println!("Frametime: {}s (fps: {})", spent, 1f64 / spent);

          //last_frame = last_update;

          let new_input_state = get_input_state(&event_pump);
          device.update_input(new_input_state);

          for event in event_pump.poll_iter() {
            match event {
              Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main_loop,
              _ => ()
            }
          }

          let time_spent = Instant::now().duration_since(last_frame);
          let expected_time = Duration::new(0, ((1f64 / GB_CYCLES_PER_SEC as f64) * total_cycles as f64 * 1e9) as u32);

          let sleep_time = if time_spent > expected_time { Duration::new(0, 0) } else { expected_time - time_spent };

          last_frame = Instant::now();
          total_cycles = 0;
          
          //println!("FPS: {}", 1.0f64 / ((time_spent.subsec_nanos() as f64 + time_spent.as_secs() as f64 * 1e9) / 1e9));
          //sleep(sleep_time);
        }
      }
    }
  }
}
