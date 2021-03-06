use std::panic::catch_unwind;

use crate::emulation::instruction::Instruction::*;
use crate::emulation::instruction::*;
use crate::emulation::instruction_decoder::*;

struct MockDevice {
    data: Vec<u8>,
    pc: u16
}

impl ReadOnlyByteStream for MockDevice {
    fn read_next_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.data[pc as usize]
    }

    fn get_stream_position(&self) -> u16 {
        self.pc
    }
}

impl MockDevice {
    pub fn from_bytes(bytes: Vec<u8>) -> MockDevice {
        MockDevice { data: bytes, pc: 0 }
    }
}

fn decode_bytes(bytes: &[u8]) -> Instruction {
    let mut device = MockDevice::from_bytes(bytes.to_vec());
    decode_instruction(&mut device)
}

fn verify_instruction(expected: Instruction, bytes: &[u8]) {
    let instruction = decode_bytes(bytes);
    assert_eq!(expected, instruction);
}

#[test]
pub fn random_instructions() {
    verify_instruction(Nop, &[0]);
    verify_instruction(OrOperandWithA(Operand8::B), &[0xB0]);
    verify_instruction(
        MoveOperand8 {
            to: Operand8::MemoryReference,
            from: Operand8::L
        },
        &[0x75]
    );
    verify_instruction(Instruction::SetBit(4, Operand8::D), &[0xCB, 0xE2]);
    verify_instruction(
        Instruction::SubtractOperandFromABorrow(Operand8::A),
        &[0x9F]
    );
}

#[test]
pub fn load_immediate_8() {
    verify_instruction(
        MoveOperand8 {
            to: Operand8::B,
            from: Operand8::Immediate(0xFF)
        },
        &[0x06, 0xFF]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::D,
            from: Operand8::Immediate(0x00)
        },
        &[0x16, 0x00]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::H,
            from: Operand8::Immediate(0x01)
        },
        &[0x26, 0x01]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::C,
            from: Operand8::Immediate(0x02)
        },
        &[0x0E, 0x02]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::E,
            from: Operand8::Immediate(0x03)
        },
        &[0x1E, 0x03]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::L,
            from: Operand8::Immediate(0x04)
        },
        &[0x2E, 0x04]
    );

    verify_instruction(
        MoveOperand8 {
            to: Operand8::A,
            from: Operand8::Immediate(0x05)
        },
        &[0x3E, 0x05]
    );
}

#[test]
pub fn special_stores_8() {
    verify_instruction(
        MoveOperand8 {
            to: Operand8::MemoryReference,
            from: Operand8::Immediate(0x00)
        },
        &[0x36, 0x00]
    );

    verify_instruction(StoreA(0x00FF), &[0xEA, 0xFF, 0x00]);
    verify_instruction(StoreSP(0x1234), &[0x08, 0x34, 0x12]);
    verify_instruction(StoreAIndirectHLIncrement, &[0x22]);
    verify_instruction(StoreAIndirectHLDecrement, &[0x32]);
    verify_instruction(StoreAHigh(0xFF), &[0xE0, 0xFF]);
    verify_instruction(StoreAHighC, &[0xE2]);
}

#[test]
pub fn removed_instructions_should_be_unknown_or_panic() {
    let removed_instructions = vec![
        vec![0xD3],
        vec![0xDB],
        vec![0xDD],
        vec![0xE3],
        vec![0xE4],
        vec![0xEB],
        vec![0xEC],
        vec![0xF2],
        vec![0xF4],
        vec![0xFC],
        vec![0xFD],
    ];

    for i in removed_instructions {
        let decoded = catch_unwind(|| decode_bytes(&i));
        match decoded {
            Err(_) | Ok(Unknown(_)) => {}
            _ => panic!("Expected Unknown, was {:?}", decoded)
        }
    }
}

#[test]
pub fn bitwise_swap() {
    verify_instruction(SwapNibbles(Operand8::A), &[0xCB, 0x37]);
    verify_instruction(SwapNibbles(Operand8::B), &[0xCB, 0x30]);
    verify_instruction(SwapNibbles(Operand8::C), &[0xCB, 0x31]);
    verify_instruction(SwapNibbles(Operand8::D), &[0xCB, 0x32]);
    verify_instruction(SwapNibbles(Operand8::E), &[0xCB, 0x33]);
    verify_instruction(SwapNibbles(Operand8::H), &[0xCB, 0x34]);
    verify_instruction(SwapNibbles(Operand8::L), &[0xCB, 0x35]);
    verify_instruction(SwapNibbles(Operand8::MemoryReference), &[0xCB, 0x36]);
}

#[test]
pub fn bitwise_complement() {
    // CPL
    verify_instruction(ComplementA, &[0x2F]);
    // CCF
    verify_instruction(ComplementCarry, &[0x3F]);
}
