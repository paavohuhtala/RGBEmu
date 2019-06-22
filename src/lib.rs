#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![feature(trace_macros)]
#![feature(log_syntax)]
#![feature(box_syntax)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bitfield;
extern crate sdl2;
extern crate time;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod test_util;

pub mod disassembler;
pub mod emulation;
pub mod rendering;
