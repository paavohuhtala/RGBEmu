
mod common;
use common::{run_program, read_address};

#[test]
fn store_sp() {
  let device = run_program(&[
    // LD SP, 0x1234
    0x31,
    0x34,
    0x12,
    // LD 0xCAAA, SP
    0x08,
    0xAA,
    0xCA,
    // HALT
    0x76
  ]);
  assert_eq!(0x34, read_address(&device, 0xCAAA));
  assert_eq!(0x12, read_address(&device, 0xCAAB));
}