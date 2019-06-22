
use crate::emulation::interrupt::Interrupt;
use crate::emulation::internal_message::{InternalMessage};

#[derive(Debug, Clone, Copy)]
pub enum TimerRegister {
  Divider,
  Counter,
  Modulo,
  Control
}

#[derive(Debug)]
pub struct TimerRegisters {
  divider_tick: u16,
  counter_tick: u16,
  modulo: u8,
  control: TimerControlRegister,
  divider: u8,
  counter: u8
}

bitfield! {
  #[derive(Default, Clone, Copy)]
  pub struct TimerControlRegister(u8);
  impl Debug;

  get_input_clock, _: 1, 0;
  get_timer_enabled, _: 2;
}

impl TimerControlRegister {
  pub fn get_counter_tick_target(self) -> u16 {
    match self.get_input_clock() {
      0b00 => 1024,
      0b01 => 16,
      0b10 => 64,
      0b11 => 256,
      _    => panic!("This can't happen.") 
    }
  }
}

impl TimerRegisters {
  pub fn new() -> TimerRegisters {
    TimerRegisters {
      divider_tick: 0,
      counter_tick: 0,
      modulo: 0,
      control: TimerControlRegister::default(),
      divider: 0,
      counter: 0
    }
  }

  pub fn update(&mut self) -> InternalMessage {
    if !self.control.get_timer_enabled() {
      return InternalMessage::None;
    }

    self.divider_tick += 1;

    if self.divider_tick >= 255 {
      self.divider_tick = 0;
      self.divider = self.divider.wrapping_add(1);
    }

    self.counter_tick += 1;

    if self.counter_tick >= self.control.get_counter_tick_target() - 1 {
      self.counter_tick = 0;

      if self.counter == 255 {
        self.counter = self.modulo;
        InternalMessage::TriggerInterrupt(Interrupt::TimerOverflow)
      } else {
        self.counter += 1;
        InternalMessage::None
      }
    } else {
      InternalMessage::None
    }
  }

  pub fn resolve_address(&self, addr: u16) -> TimerRegister {
    println!("Timer register: {:04x}", addr);
    match addr {
      0xFF04 => TimerRegister::Divider,
      0xFF05 => TimerRegister::Counter,
      0xFF06 => TimerRegister::Modulo,
      0xFF07 => TimerRegister::Control,
      _ => panic!("Invalid timer address: ${:04x}", addr)
    }
  }

  pub fn write_8(&mut self, register: TimerRegister, value: u8) {
    match register {
      TimerRegister::Divider | TimerRegister::Counter => {
        self.divider = 0;
        self.counter = 0;
      },
      TimerRegister::Modulo => self.modulo = value,
      TimerRegister::Control => {
        self.control = TimerControlRegister(value);
        println!("Timer control: {:b}", value);
      }
    }
  }

  pub fn read_8(&self, register: TimerRegister) -> u8 {
    match register {
      TimerRegister::Divider => self.divider,
      TimerRegister::Counter => self.counter,
      TimerRegister::Modulo => self.modulo,
      TimerRegister::Control => self.control.0
    }
  }
}
