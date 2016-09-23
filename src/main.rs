
#![allow(dead_code)]
#![feature(box_syntax)]
#![feature(associated_consts)]
#![feature(plugin)]
#![plugin(clippy)]

#![allow(wrong_self_convention)]

use std::io::prelude::*;
use std::fs::File;

extern crate sfml;

use sfml::graphics::{RenderWindow, RenderTarget, Color};
use sfml::window::{VideoMode, window_style};
use sfml::window::event::Event;
use sfml::system::{Vector2u};

mod emulation;

use emulation::device::Device;

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

  let mut device = Device::new_gbc(Some(rom_buffer));
  times!(255, {device.run_cycle()});

  //device.run_cycle();

  /*
  loop {
    for event in window.events() {
      match event {
        Event::Closed => return,
        _ => {}
      }
    }
    device.run_cycle()
  }*/
}