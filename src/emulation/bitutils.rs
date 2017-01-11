

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

pub struct BytePair { pub high: u8, pub low: u8 }

pub fn u16_to_pair(value: u16) -> BytePair {
  let high = (value >> 8) as u8;
  let low = (value & 0x00FF) as u8;
  let low = (value & 0x00FF) as u8;

  BytePair { high, low }
}

pub fn u16_from_bytes(high: u8, low: u8) -> u16 {
  ((high as u16) << 8) | (low as u16)
}

pub trait BitExtensions where Self : Sized {
  fn get_bit(self, n: u8) -> bool;
  fn set_bit(self, n: u8) -> Self;
  fn clear_bit(self, n: u8) -> Self;

  fn set_bit_to(self, n: u8, value: bool) -> Self {
    if value {
      self.set_bit(n)
    } else {
      self.clear_bit(n)
    }
  }
}

impl BitExtensions for u8 {
  fn get_bit(self, n: u8) -> bool {
    if self & (1 << n) != 0 { true } else { false }
  }

  fn set_bit(self, n: u8) -> u8 {
    self | (1 << n)
  }

  fn clear_bit(self, n: u8) -> u8 {
    self & !(1 << n)
  }
}

pub trait BoolExtensions {
  fn to_u8(self) -> u8;
}

impl BoolExtensions for bool {
  fn to_u8(self) -> u8 {
    if self { 1 } else { 0 }
  }
}