
use emulation::constants::*;
use emulation::registers::{Registers};
use emulation::mmu::{MMU};
use emulation::instruction::{Instruction, Operand, RegisterPair, BCOrDE, ConditionCode};
use emulation::instruction::Instruction::*;

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

pub enum ExecutionState {
  Halted,
  Paused,
  Running
}

pub struct Device {
  pub regs: Registers,
  pub memory: MMU,
  pub execution_state: ExecutionState
}

mod bitutils {
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
}

impl Device {
  pub fn new(device: DeviceType, bootrom: Option<Vec<u8>>) -> Device {
    Device {
      regs: Registers::new(),
      memory: MMU::new(device, bootrom),
      execution_state: ExecutionState::Halted
    }
  }

  pub fn simulate_bootrom(&mut self) {
    self.regs
    .set_pc(0x100)
    .set_sp(0xFFFE)
    .set_af(0x01B0)
    .set_bc(0x0013)
    .set_de(0x00D8)
    .set_hl(0x014d);
  }

  pub fn new_gbc(bootrom: Option<Vec<u8>>) -> Device { Device::new(DeviceType::GameBoyColor, bootrom) }

  fn read_next_byte(&mut self) -> u8 {
    let pc = self.regs.pc;
    self.regs.pc += 1;
    self.memory.read8(self.memory.resolve_address(pc))
  }

  #[inline]
  fn read_next_16(&mut self) -> u16 {
    let low = self.read_next_byte();
    let high = self.read_next_byte();

    ((high as u16) << 8) | (low as u16)
  }

  fn as_register_pair(b1: u8, b0: u8) -> RegisterPair {
    RegisterPair::decode(bitutils::to_byte_2(b1, b0)).unwrap()
  }

  fn as_bc_or_de(b1: u8, b0: u8) -> BCOrDE {
    RegisterPair::decode(bitutils::to_byte_2(b1, b0)).and_then(RegisterPair::as_bc_or_de).unwrap()
  }

  fn to_operand(b2: u8, b1: u8, b0: u8) -> Operand {
    Operand::decode(bitutils::to_byte_3(b2, b1, b0)).unwrap()
  }

  fn to_condition_code(b2: u8, b1: u8, b0: u8) -> ConditionCode {
    ConditionCode::decode(bitutils::to_byte_3(b2, b1, b0)).unwrap()
  }

  // Partially based on:
  // http://www.classiccmp.org/dunfield/r/8080.txt
  fn decode_next_instruction(&mut self) -> Instruction {
    print!("${:04x} ", self.regs.pc);
    let first_byte = self.read_next_byte();
    println!("0x{:X}", first_byte);
    let first_bits = bitutils::to_bit_tuple(first_byte);

    match first_bits {
      (0, 0, 0, 0, 0, 0, 0, 0) => Nop,
      (0, 1, 1, 1, 0, 1, 1, 0) => Halt,
      (0, 0, 0, 1, 0, 0, 0, 0) => Stop,
      (0, 1, d2, d1, d0, s2, s1, s0) => {
        let to =  Device::to_operand(d2, d1, d0);
        let from = Device::to_operand(s2, s1, s0);
        MoveOperand8 {to: to, from: from}
      },
      (0, 0, d2, d1, d0, 1, 1, 0) => {
        let to = Device::to_operand(d2, d1, d0);
        let immediate = self.read_next_byte();
        MoveImmediate8 {to: to, value: immediate}
      },
      (0, 0, r1, r0, 0, 0, 0, 1) => {
        let to = Device::as_register_pair(r1, r0);
        let immediate = self.read_next_16();
        MoveImmediate16 {to: to, value: immediate}
      },
      (1, 1, 1, 1, 1, 0, 1, 0) => LoadA(self.read_next_16()),
      (1, 1, 1, 0, 1, 0, 1, 0) => StoreA(self.read_next_16()),
      (0, 0, 1, 0, 1, 0, 1, 0) => LoadAIndirectHLIncrement,
      (0, 0, 1, 0, 0, 0, 1, 0) => StoreAIndirectHLIncrement,
      (0, 0, 1, 1, 1, 0, 1, 0) => LoadAIndirectHLDecrement,
      (0, 0, 1, 1, 0, 0, 1, 0) => StoreAIndirectHLDecrement,
      (0, 0, r1, r0, 1, 0, 1, 0) => LoadAIndirect(Device::as_bc_or_de(r1, r0)),
      (0, 0, r1, r0, 0, 0, 1, 0) => StoreAIndirect(Device::as_bc_or_de(r1, r0)),
      (1, 1, 1, 1, 0, 0, 0, 0) => LoadAHigh(self.read_next_byte()),
      (1, 1, 1, 0, 0, 0, 0, 0) => StoreAHigh(self.read_next_byte()),
      (1, 1, 1, 0, 0, 0, 1, 0) => StoreAHighC,
      (1, 0, 0, 0, carry, s2, s1, s0) => {
        let register = Device::to_operand(s2, s1, s0);
        if carry == 1 {
          AddOperandToACarry(register)
        } else {
          AddOperandToA(register)
        }
      },
      (1, 1, 0, 0, carry, 1, 1, 0) => {
        let immediate = self.read_next_byte();
        if carry == 1 {
          AddImmediateToACarry(immediate)
        } else {
          AddImmediateToA(immediate)
        }
      },
      (1, 0, 0, 1, carry, s2, s1, s0) => {
        let register = Device::to_operand(s2, s1, s0);
        if carry == 1 {
          SubtractOperandFromABorrow(register)
        } else {
          SubtractOperandFromA(register)
        }
      },
      (1, 1, 0, 1, carry, 1, 1, 0) => {
        let immediate = self.read_next_byte();
        if carry == 1 {
          SubtractImmediateFromABorrow(immediate)
        } else {
          SubtractImmediateFromA(immediate)
        }
      },
      (0, 0, d2, d1, d0, 1, 0, decr) => {
        let register = Device::to_operand(d2, d1, d0);
        if decr == 1 {
          DecrementOperand8(register)
        } else {
          IncrementOperand8(register)
        }
      },
      (0, 0, r1, r0, decr, 0, 1, 1) => {
        let register_pair = Device::as_register_pair(r1, r0);
        if decr == 1 {
          DecrementOperand16(register_pair)
        } else {
          IncrementOperand16(register_pair)
        }
      },
      (0, 0, r1, r0, 1, 0, 0, 1) => AddOperandToHL(Device::as_register_pair(r1, r0)),
      (0, 0, 1, 0, 0, 1, 1, 1) => BCDCorrectA,
      (1, 0, 1, 0, 0, s2, s1, s0) => AndOperandWithA(Device::to_operand(s2, s1, s0)),
      (1, 1, 1, 0, 0, 1, 1, 0) => AndImmediateWithA(self.read_next_byte()),
      (1, 0, 1, 1, 0, s2, s1, s0) => OrOperandWithA(Device::to_operand(s2, s1, s0)),
      (1, 1, 1, 1, 0, 1, 1, 0) => OrImmediateWithA(self.read_next_byte()),
      (1, 0, 1, 0, 1, s2, s1, s0) => XorOperandWithA(Device::to_operand(s2, s1, s0)),
      (1, 1, 1, 0, 1, 1, 1, 0) => XorImmediateWithA(self.read_next_byte()),
      (1, 0, 1, 1, 1, s2, s1, s0) => CompareOperandWithA(Device::to_operand(s2, s1, s0)),
      (1, 1, 1, 1, 1, 1, 1, 0) => CompareImmediateWithA(self.read_next_byte()),
      (0, 0, 0, carry, right, 1, 1, 1) => {
        match (carry == 1, right == 1) {
          (false, false) => RotateALeft,
          (true, false) => RotateALeftCarry,
          (false, true) => RotateARight,
          (true, true) => RotateARightCarry
        }
      },
      (0, 0, 1, 0, 1, 1, 1, 1) => ComplementA,
      (0, 0, 1, 1, 1, 1, 1, 1) => ComplementCarry,
      (0, 0, 1, 1, 0, 1, 1, 1) => SetCarry,
      (1, 1, 0, 0, 0, 0, 1, 1) => Jump(self.read_next_16()),
      (1, 1, c2, c1, c0, 0, 1, 0) => ConditionalJump(Device::to_condition_code(c2, c1, c0), self.read_next_16()),
      (0, 0, 0, 1, 1, 0, 0, 0) => RelativeJump(self.read_next_byte() as i8),
      (0, 0, 1, 0, 0, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Zero(false), self.read_next_byte() as i8),
      (0, 0, 1, 1, 0, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Carry(false), self.read_next_byte() as i8),
      (0, 0, 1, 0, 1, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Zero(true), self.read_next_byte() as i8),
      (0, 0, 1, 1, 1, 0, 0, 0) => ConditionalRelativeJump(ConditionCode::Carry(true), self.read_next_byte() as i8),
      (1, 1, 0, 0, 1, 1, 0, 1) => Call(self.read_next_16()),
      (1, 1, c2, c1, c0, 1, 0, 0) => ConditionalCall(Device::to_condition_code(c2, c1, c0), self.read_next_16()),
      (1, 1, 0, 0, 1, 0, 0, 1) => Return,
      (1, 1, c2, c1, c0, 0, 0, 0) => ConditionalReturn(Device::to_condition_code(c2, c1, c0)),
      (1, 1, n2, n1, n0, 1, 1, 1) => Restart(bitutils::to_byte_3(n2, n1, n0)),
      (1, 1, 1, 0, 1, 0, 0, 1) => JumpToHL,
      (1, 1, r1, r0, 0, 1, 0, 1) => Push(Device::as_register_pair(r1, r0)),
      (1, 1, r1, r0, 0, 0, 0, 1) => Pop(Device::as_register_pair(r1, r0)),
      (1, 1, 1, 1, enable, 0, 1, 1) => {
        if enable == 1 {
          EnableInterrupts
        } else {
          DisableInterrupts
        }
      },
      // Bit instructions
      _ if first_byte == 0xCB => {
        println!("bit instructions");
        let next_byte = self.read_next_byte();
        println!("0x{:X}", next_byte);
        let next_bits = bitutils::to_bit_tuple(next_byte);
        match next_bits {
          (0, 0, 0, no_carry, 0, d2, d1, d0) => {
            let register = Device::to_operand(d2, d1, d0);
            if no_carry == 1 {
              RotateLeft(register)
            } else {
              RotateLeftCarry(register)
            }
          },
          (0, 0, 0, no_carry, 1, d2, d1, d0) => {
            let register = Device::to_operand(d2, d1, d0);
            if no_carry == 1 {
              RotateRight(register)
            } else {
              RotateRightCarry(register)
            }
          },
          (0, 0, 1, logical, 0, d2, d1, d0) => {
            let register = Device::to_operand(d2, d1, d0);
            if logical == 1 {
              // Technically an undocumented instruction
              ShiftLeftLogical(register)
            } else {
              ShiftLeftArithmetic(register)
            }
          },
          (0, 0, 1, logical, 1, d2, d1, d0) => {
            let register = Device::to_operand(d2, d1, d0);
            if logical == 1 {
              ShiftRightLogical(register)
            } else {
              ShiftRightArithmetic(register)
            }
          },
          (0, 1, b2, b1, b0, d2, d1, d0) => TestBit(bitutils::to_byte_3(b2, b1, b0), Device::to_operand(d2, d1, d0)),
          (1, 0, b2, b1, b0, d2, d1, d0) => ClearBi(bitutils::to_byte_3(b2, b1, b0), Device::to_operand(d2, d1, d0)),
          (1, 1, b2, b1, b0, d2, d1, d0) => SetBit(bitutils::to_byte_3(b2, b1, b0), Device::to_operand(d2, d1, d0)),
          _ => panic!("Unimplemented bit instruction: 0b{:b}", first_byte)
        }
      }
      _ => panic!("Unimplemented instruction: 0b{:b}", first_byte)
    }
  }

  pub fn run_cycle(&mut self) {
    let instr = self.decode_next_instruction();
    println!("{:?}", instr);
  }
}
