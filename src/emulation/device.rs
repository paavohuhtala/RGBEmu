
use std::io::{stdin, Read};
use std::collections::HashSet;
use std::thread::{sleep};
use std::sync::Arc;
use std::time::Duration;
use time::{precise_time_ns};

use emulation::bitutils::*;
use emulation::internal_message::{InternalMessage, RendererMessage};
use emulation::constants::*;
use emulation::registers::{Registers};
use emulation::address_mapper::{AddressMapper, Addressable};
use emulation::bus::{Bus};
use emulation::input::InputState;

use emulation::interrupt::{Interrupt};
use emulation::instruction::{Instruction, Operand8, Operand16};
use emulation::instruction::Operand8::*;
use emulation::instruction::Operand16::*;
use emulation::instruction_decoder::*;
use emulation::interpreter;

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
  GameBoy,
  GameBoyColor
}

pub struct DeviceInfo {
  pub ram_size: usize,
  pub vram_size: usize
}

impl DeviceType {
  pub fn get_device_info(&self) -> DeviceInfo {
    match *self {
      DeviceType::GameBoy => DeviceInfo {ram_size: GB_RAM_SIZE, vram_size: GB_VRAM_SIZE},
      DeviceType::GameBoyColor => DeviceInfo {ram_size: GBC_RAM_SIZE, vram_size: GBC_VRAM_SIZE}
    }
  }

  pub fn get_ram_size(&self) -> usize {
    self.get_device_info().ram_size
  }

  pub fn get_vram_size(&self) -> usize {
    self.get_device_info().vram_size
  }

  pub fn get_ram_bank_count(&self) -> usize {
    (self.get_device_info().ram_size / RAM_BANK_SIZE) as usize
  }

  pub fn get_vram_bank_count(&self) -> usize {
    (self.get_device_info().vram_size / VRAM_BANK_SIZE) as usize
  }
}

#[derive(PartialEq, Eq)]
pub enum ExecutionState {
  Halted,
  Paused,
  Running
}

pub enum DebugState {
  Default,
  HandlingBreakpoint
}

pub struct Device {
  pub regs: Registers,
  pub bus: Bus,
  pub execution_state: ExecutionState,
  pub interrupts_enabled: bool,
  pub debug_state: DebugState,

  breakpoints: HashSet<u16>,
  renderer_messages: Vec<RendererMessage>
}

impl Device {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> Device {
    let mut device = Device {
      regs: Registers::new(),
      bus: Bus::new(device, bootrom),
      execution_state: ExecutionState::Running,
      interrupts_enabled: true,
      debug_state: DebugState::Default,
      breakpoints: HashSet::new(),
      renderer_messages: Vec::with_capacity(128)
    };

    if device.bus.bootrom.is_none() {
      device.simulate_bootrom();
    }

    device
  }

  pub fn simulate_bootrom(&mut self) {
    self.regs.pc = 0x100;
    self.regs.sp = 0xFFFE;
    self.regs.set_af(0x01B0);
    self.regs.set_bc(0x0013);
    self.regs.set_de(0x00D8);
    self.regs.set_hl(0x014d);
  }

  pub fn new_gb(bootrom: Option<Vec<u8>>) -> Device {
    Device::new(DeviceType::GameBoy, bootrom)
  }

  pub fn read_next_byte(&mut self) -> u8 {
    let pc = self.regs.pc;
    self.regs.pc += 1;
    self.bus.read_8(self.bus.resolve_address(pc))
  }

  fn decode_next_instruction(&mut self) -> Instruction {
    decode_instruction(self)
  }

  pub fn run_instruction(&mut self) -> u32 {
    interpreter::run_instruction(self)
  }

  pub fn write_addr_8(&mut self, addr: u16, value: u8) {
    let msg = self.bus.write_addr_8(addr, value);
    self.handle_message(msg);
  }

  pub fn push_16(&mut self, value: u16) {
    let BytePair { high, low } = u16_to_pair(value);
    let sp = self.regs.sp;
    // POTENTIAL BUG: these writes don't send internal messages
    self.write_addr_8(sp, low);
    self.write_addr_8(sp - 1, high);
    self.regs.sp -= 2;
  }

  pub fn pop_16(&mut self) -> u16 {
    let high = self.bus.read_addr_8(self.regs.sp + 1);
    let low = self.bus.read_addr_8(self.regs.sp + 2);
    self.regs.sp += 2;
    u16_from_bytes(high, low)
  }

  fn handle_message(&mut self, message: InternalMessage) {
    match message {
      InternalMessage::TriggerInterrupt(interrupt) => {
        if let Interrupt::LCDVBlank = interrupt {
          self.renderer_messages.push(RendererMessage::PresentFrame);
        }
        self.bus.interrupt.request_interrupt(interrupt);
      }
      InternalMessage::RendererMessage(msg) => self.renderer_messages.push(msg),
      InternalMessage::DMATransfer { from } => {
        let mut buffer = [0u8; 160];
        self.bus.read_to_buffer(&mut buffer, from, 160);
        self.bus.video.oam = buffer;
      }
      InternalMessage::None => (),
      _ => panic!("Unsupported internal message: {:?}", message)
    }
  }

  fn check_interrupts(&mut self) {
    if let Some(interrupt) = self.bus.interrupt.handle_next_interrupt() {
      if self.execution_state == ExecutionState::Halted {
        self.execution_state = ExecutionState::Running;
      }

      self.interrupts_enabled = false;
      let pc = self.regs.pc;
      self.push_16(pc);
      self.regs.pc = interrupt.get_handler_address();
    }
  }

  pub fn run_tick(&mut self) -> u32 {
    if self.breakpoints.contains(&self.regs.pc) {
      self.debug_state = DebugState::HandlingBreakpoint;
    } else {
      self.debug_state = DebugState::Default;
    }

    let mut elapsed_cycles = 4;

    if self.execution_state != ExecutionState::Halted {
      elapsed_cycles = self.run_instruction();
    }

    if self.bus.video.is_lcd_on() {
      let gpu_message = self.bus.video.update(elapsed_cycles);
      self.handle_message(gpu_message);
    }

    for _ in 0 .. elapsed_cycles {
      let timer_message = self.bus.timer.update();
      self.handle_message(timer_message);
    }

    if self.interrupts_enabled {
      self.check_interrupts();
    }

    elapsed_cycles
  }

  pub fn next_renderer_message(&mut self) -> Option<RendererMessage> {
    self.renderer_messages.pop()
  }

  pub fn pause(&mut self) {
    let _ = stdin().read(&mut [0u8]).unwrap();
  }

  pub fn set_breakpoint(&mut self, addr: u16) {
    self.breakpoints.insert(addr);
  }

  pub fn update_input(&mut self, state: InputState) {
    self.bus.input.update(state)
  }

  pub fn halt(&mut self) {
    self.execution_state = ExecutionState::Halted;
  } 
}

impl ReadOnlyByteStream for Device {
  fn read_next_byte(&mut self) -> u8 {
    self.read_next_byte()
  }

  fn get_stream_position(&self) -> u16 {
    self.regs.pc
  }
}

pub trait ReadWriteRegisters {
  fn get_operand_8(&self, operand: Operand8) -> u8;
  fn get_operand_16(&self, operand: Operand16) -> u16;
  fn set_operand_8(&mut self, operand: Operand8, value: u8);
  fn set_operand_16(&mut self, operand: Operand16, value: u16);
}

impl ReadWriteRegisters for Device {
  fn get_operand_8(&self, operand: Operand8) -> u8 {
    match operand {
      A => self.regs.a,
      B => self.regs.b,
      C => self.regs.c,
      D => self.regs.d,
      E => self.regs.e,
      H => self.regs.h,
      L => self.regs.l,
      MemoryReference => self.bus.read_addr_8(self.regs.hl()),
      Immediate(value) => value
    }
  }

  fn set_operand_8(&mut self, operand: Operand8, value: u8) {
    match operand {
      A => self.regs.a = value,
      B => self.regs.b = value,
      C => self.regs.c = value,
      D => self.regs.d = value,
      E => self.regs.e = value,
      H => self.regs.h = value,
      L => self.regs.l = value,
      MemoryReference => {
        let addr = self.regs.hl();
        self.write_addr_8(addr, value);
      },
      Immediate(_) => panic!("Tried to set an immediate value"),
    }
  }

  fn get_operand_16(&self, operand_16: Operand16) -> u16 {
    match operand_16 {
      BC => self.regs.bc(),
      DE => self.regs.de(),
      HL => self.regs.hl(),
      SP => self.regs.sp
    }
  }

  fn set_operand_16(&mut self, operand_16: Operand16, value: u16) {
    match operand_16 {
      BC => self.regs.set_bc(value),
      DE => self.regs.set_de(value),
      HL => self.regs.set_hl(value),
      SP => self.regs.sp = value
    };
  }
}