
#![allow(dead_code)]
#![feature(box_syntax)]
#![feature(associated_consts)]
#![feature(plugin)]
#![feature(field_init_shorthand)]

#[cfg(test)]
mod tests;

use std::io::prelude::*;
use std::fs::File;

extern crate sfml;

use sfml::graphics::{RenderWindow, RenderTarget, Color};
use sfml::window::{VideoMode, window_style};
use sfml::window::event::Event;
use sfml::system::{Vector2u};

mod emulation;
use emulation::device::Device;
use emulation::cartridge::{Cartridge};

macro_rules! times {
  ($n: expr, $_fn: block) => { for _ in 0..$n { $_fn } }
}

fn main() {
  /*let res = Vector2u::new(160, 144) * 4;
  let mut window = RenderWindow::new(VideoMode::new_init(res.x, res.y, 32), "RGBEmu", window_style::CLOSE, &Default::default()).unwrap();
  window.clear(&Color::black());
  window.display();
  window.set_framerate_limit(60);*/

  let mut rom_buffer : Vec<u8> = vec!();
  File::open("DMG_ROM.bin").unwrap().read_to_end(&mut rom_buffer).unwrap();

  let mut cartridge_data : Vec<u8> = vec!();
  File::open("./cpu_instrs/cpu_instrs.gb").unwrap().read_to_end(&mut cartridge_data).unwrap();

  let mut device = Device::new_gbc(Some(rom_buffer));
  let cartridge = Cartridge::from_bytes(&cartridge_data).unwrap();

  device.memory.cartridge = Some(cartridge);

  loop {
    device.run_cycle()
  }

  //device.run_cycle();

  
  /*loop {
    for event in window.events() {
      match event {
        Event::Closed => return,
        _ => {}
      }
    }
    device.run_cycle()*/
  //}
}
