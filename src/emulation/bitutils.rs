

// Based on https://www.reddit.com/r/rust/comments/2d7rrj/bit_level_pattern_matching/cjmxjtn
#[inline]
pub fn to_bit_tuple(byte: u8) -> (u8, u8, u8, u8, u8, u8, u8, u8) {
  (byte >> 7 & 1,
    byte >> 6 & 1,
    byte >> 5 & 1,
    byte >> 4 & 1,
    byte >> 3 & 1,
    byte >> 2 & 1,
    byte >> 1 & 1,
    byte & 1)
}

#[inline]
pub fn to_byte_2(b1: u8, b0: u8) -> u8 {
  b1 << 1 | b0
}

#[inline]
pub fn to_byte_3(b2: u8, b1: u8, b0: u8) -> u8 {
  b2 << 2 | b1 << 1 | b0
}

pub fn get_nth_bit(byte: u8, n: u8) -> u8 {
  if byte & (1 << n) != 0 {1} else {0}
}

pub fn set_nth_bit(byte: u8, n: u8) -> u8 {
  byte | (1 << n)
}
