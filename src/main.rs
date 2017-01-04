use std::io::prelude::*;
use std::fs::File;

#[derive(Debug)]
enum B {
    One(u8),
    Two(u16)
}

#[derive(Debug)]
enum Cond {
    NZ,
    Z,
    NC,
    C
}

#[derive(Debug)]
enum OpCode {
    ADD(Register, Register),
    ADD_C(Register, Register),
    AND(Register),
    BIT(u8, Register),
    CALL(Register),
    CALL_C(Cond, Register),
    CP(Register),
    DEC(Register),
    DEC_F(Register),
    DI,
    ERR(String),
    HALT,
    INC(Register),
    INC_F(Register),
    JP(u16),
    JP_C(Cond, u16),
    JR_C(Cond, i8),
    LD(Register, B),
    LD_R(Register, Register),
    NOP,
    OR(Register),
    POP(Register),
    PUSH(Register),
    RET,
    RETI,
    RET_C(Cond),
    RST(u8),
    STOP,
    SUB(Register),
    SUB_C(Register, Register),
    XOR(Register)
}

//struct ROpCode<A> {
//    nop: fn(i32) -> A,
//    jp: fn(i32) -> A,
//    jr_nz: fn(u8, i32) -> A
//}
//
//fn interp_op<A>(start: usize, y: &Vec<u8>, interp: ROpCode<A>) -> (usize, A) {
//    match y[start] {
//        0x00 => (start + 1, interp.nop(4))
//        0x20 => (2 + start, interp.jr_nz(y[start+1], 8))
//    }
//}
//
//fn runner_thingy() {
//    let interp: ROpCode<str> = ROpCode {
//        nop: {|c| "NOP"},
//        jp: {|c| "JP"},
//        jr_nz: {|u, c| "JRNZ"}
//    }
//}
//
#[derive(Debug, Copy, Clone)]
enum Register {
    A,
    B,
    C,
    CH, //$FF00+C
    D,
    E,
    F,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    HLP,
    HLM,
    ADDR(u16), 
    SP, // Stack Pointer
    PC // Program Counter
}

fn get_arg(start:usize, num:u8, res:&Vec<u8>) -> u16 {
    match num {
        3 => ((res[start + 2] as u16) << 8) + (res[start + 1] as u16) ,
        _ => 0
    } 
}

fn get_cb(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    match y[start + 1] {
        0x7C => (2, OpCode::BIT(7, Register::H), 8),
        _ => (2, OpCode::ERR(format!("{:0>2X}", y[start])), 0)
    }
}

fn lookup_mod_register(b:u8) -> Register {
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  registers[(b % 8) as usize]
}

fn lookup_mod_cycles(b:u8) -> u8 {
  if (b % 8) == 6 { 8 } else { 4 } 
}

fn lookup_LD_R(start:usize, b:u8) -> (usize, OpCode, u8){
  let idx = b - 0x40;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  let (left, right) = (registers[(idx/8) as usize], lookup_mod_register(b));
  let cycles = if (idx / 8) == 6 || (idx % 8) == 6 { 8 } else { 4 }; 
  (1, OpCode::LD_R(registers[(idx/8) as usize], lookup_mod_register(b)), cycles) 
}

fn lookup_op(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    let res = match y[start] {
        0x00 => (1, OpCode::NOP, 4),
        0x10 => (2, OpCode::STOP, 4),
        0x20 => (2, OpCode::JR_C(Cond::NZ, y[start + 1] as i8), 12), // 12/8 The first arg should be a signed byte
        0x30 => (2, OpCode::JR_C(Cond::NC, y[start + 1] as i8), 12), //12/8 The first arg should be a signed byte
        0x01 => (3, OpCode::LD(Register::BC, B::Two(get_arg(start, 3, y))), 12),
        0x11 => (3, OpCode::LD(Register::DE, B::Two(get_arg(start, 3, y))), 12),
        0x21 => (3, OpCode::LD(Register::HL, B::Two(get_arg(start, 3, y))), 12),
        0x31 => (3, OpCode::LD(Register::SP, B::Two(get_arg(start, 3, y))), 12),
        0x02 => (1, OpCode::LD_R(Register::BC, Register::A), 8),
        0x12 => (1, OpCode::LD_R(Register::DE, Register::A), 8),
        0x22 => (1, OpCode::LD_R(Register::HLP, Register::A), 8),
        0x32 => (1, OpCode::LD_R(Register::HLM, Register::A), 8),
        0x03 => (1, OpCode::INC(Register::BC), 8),
        0x13 => (1, OpCode::INC(Register::DE), 8),
        0x23 => (1, OpCode::INC(Register::HL), 8),
        0x33 => (1, OpCode::INC(Register::SP), 8),
        0x04 => (1, OpCode::INC_F(Register::B), 4),
        0x14 => (1, OpCode::INC_F(Register::D), 4),
        0x24 => (1, OpCode::INC_F(Register::H), 4),
        0x34 => (1, OpCode::INC_F(Register::HL), 12),
        0x05 => (1, OpCode::DEC_F(Register::B), 4),
        0x15 => (1, OpCode::DEC_F(Register::D), 4),
        0x25 => (1, OpCode::DEC_F(Register::H), 4),
        0x35 => (1, OpCode::DEC_F(Register::HL), 12),
        0x06 => (2, OpCode::LD(Register::B, B::One(y[start + 1])), 8),
        0x16 => (2, OpCode::LD(Register::D, B::One(y[start + 1])), 8),
        0x26 => (2, OpCode::LD(Register::H, B::One(y[start + 1])), 8),
        0x36 => (2, OpCode::LD(Register::HL, B::One(y[start + 1])), 12),
        0x09 => (1, OpCode::ADD(Register::HL, Register::BC), 8),
        0x19 => (1, OpCode::ADD(Register::HL, Register::DE), 8),
        0x29 => (1, OpCode::ADD(Register::HL, Register::HL), 8),
        0x39 => (1, OpCode::ADD(Register::HL, Register::SP), 8),
        0x0B => (1, OpCode::DEC(Register::BC), 8),
        0x1B => (1, OpCode::DEC(Register::DE), 8),
        0x2B => (1, OpCode::DEC(Register::HL), 8),
        0x3B => (1, OpCode::DEC(Register::SP), 8),
        0x0C => (1, OpCode::INC_F(Register::C), 4),
        0x1C => (1, OpCode::INC_F(Register::E), 4),
        0x2C => (1, OpCode::INC_F(Register::L), 4),
        0x3C => (1, OpCode::INC_F(Register::A), 4),
        0x0D => (1, OpCode::DEC_F(Register::C), 4),
        0x1D => (1, OpCode::DEC_F(Register::E), 4),
        0x2D => (1, OpCode::DEC_F(Register::L), 4),
        0x3D => (1, OpCode::DEC_F(Register::A), 4),
        0x0E => (2, OpCode::LD(Register::C, B::One(y[start + 1])), 8),
        0x1E => (2, OpCode::LD(Register::E, B::One(y[start + 1])), 8),
        0x2E => (2, OpCode::LD(Register::L, B::One(y[start + 1])), 8),
        0x3E => (2, OpCode::LD(Register::A, B::One(y[start + 1])), 8),
        0x0A => (1, OpCode::LD_R(Register::A, Register::BC), 8),
        0x1A => (1, OpCode::LD_R(Register::A, Register::DE), 8),
        0x2A => (1, OpCode::LD_R(Register::A, Register::HLP), 8),
        0x3A => (1, OpCode::LD_R(Register::A, Register::HLM), 8),
        0xE0 => (2, OpCode::LD_R(Register::ADDR(0xFF00 + (y[start + 1] as u16)), Register::A), 12),
        0xF0 => (2, OpCode::LD_R(Register::A, Register::ADDR(0xFF00 + (y[start + 1] as u16))), 12),
        0xC2 => (3, OpCode::JP_C(Cond::NZ, get_arg(start, 3, y)), 16), // 16/12
        0xD2 => (3, OpCode::JP_C(Cond::NC, get_arg(start, 3, y)), 16), // 16/12
        0xE2 => (2, OpCode::LD_R(Register::CH, Register::A), 8),
        0xF2 => (2, OpCode::LD_R(Register::A, Register::CH), 8),
        0xC3 => (3, OpCode::JP(get_arg(start, 3, y)), 16), 
        0xF3 => (1, OpCode::DI, 4),
        0xC4 => (3, OpCode::CALL_C(Cond::NZ, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xD4 => (3, OpCode::CALL_C(Cond::NC, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xCB => get_cb(start, y),
        0xCC => (3, OpCode::CALL_C(Cond::Z, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xDC => (3, OpCode::CALL_C(Cond::C, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xCD => (3, OpCode::CALL(Register::ADDR(get_arg(start, 3, y))), 24),
        0x76 => (1, OpCode::HALT, 4), 
        b @ 0x40...0x7F => lookup_LD_R(start, b), //All the registers that use HL have the wrong cycle count
        b @ 0x80...0x88 => (1, OpCode::ADD(Register::A, lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0x88...0x8F => (1, OpCode::ADD_C(Register::A, lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0x90...0x98 => (1, OpCode::SUB(lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0x98...0x9F => (1, OpCode::SUB_C(Register::A, lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0xA0...0xA8 => (1, OpCode::AND(lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0xA8...0xAF => (1, OpCode::XOR(lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0xB0...0xB8 => (1, OpCode::OR(lookup_mod_register(b)), lookup_mod_cycles(b)),
        b @ 0xB8...0xBF => (1, OpCode::CP(lookup_mod_register(b)), lookup_mod_cycles(b)),
        0xC0 => (1, OpCode::RET_C(Cond::NZ), 8), // actually 20/8
        0xD0 => (1, OpCode::RET_C(Cond::NC), 8), // actually 20/8
        0xC1 => (1, OpCode::POP(Register::BC), 12),
        0xD1 => (1, OpCode::POP(Register::DE), 12),
        0xE1 => (1, OpCode::POP(Register::HL), 12),
        0xF1 => (1, OpCode::POP(Register::AF), 12),
        0xC5 => (1, OpCode::PUSH(Register::BC), 16),
        0xD5 => (1, OpCode::PUSH(Register::DE), 16),
        0xE5 => (1, OpCode::PUSH(Register::HL), 16),
        0xF5 => (1, OpCode::PUSH(Register::AF), 16),
        0xC7 => (1, OpCode::RST(0x00), 16),
        0xD7 => (1, OpCode::RST(0x10), 16),
        0xE7 => (1, OpCode::RST(0x20), 16),
        0xF7 => (1, OpCode::RST(0x30), 16),
        0xC8 => (1, OpCode::RET_C(Cond::Z), 8), // actually 20/8
        0xD8 => (1, OpCode::RET_C(Cond::C), 8), // actually 20/8
        0xCF => (1, OpCode::RST(0x08), 16),
        0xDF => (1, OpCode::RST(0x18), 16),
        0xEF => (1, OpCode::RST(0x28), 16),
        0xFF => (1, OpCode::RST(0x38), 16),
        _ => (1, OpCode::ERR(format!("{:0>2X}", y[start])), 0)
    };
    res
}

// Representing A, F, B, C, D, E, H, L in that order
static mut byte_registers: [u8; 8] = [0;8];
static mut sp: u16 = 0;
static mut pc: u16 = 0;

fn main() {
    let mut f = File::open("DMG_ROM.bin").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).ok();
    let mut next_addr = 0;
    for _ in 1..256 {
        let (op_length, instr, cycles) = lookup_op(next_addr, &buffer);
        println!("Address {:4>0X}: {:?}", next_addr, instr); 
        next_addr += op_length;
    }
}