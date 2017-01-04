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
    NOP(i32),
    STOP(i32),
    JP((), i32),
    JR_C(Cond, u8, i32),
    JR_NZ((u8), i32),
    CALL(B, i32),
    LD(Register, B, i32),
    LD_R(Register, Register, i32),
    LD_FF(Register, Register, i32),
    LD_FF_I(u8, Register, i32),
    LD_HLP_A(i32),
    LD_HLM_A(i32),
    AND(Register, i32),
    OR(Register, i32),
    CP(Register, i32),
    XOR(Register, i32),
    ADD(Register, Register, i32),
    ADD_C(Register, Register, i32),
    SUB(Register, i32),
    SUB_C(Register, Register, i32),
    HALT(i32),
    POP(Register, i32),
    PUSH(Register, i32),
    ERR(String),
    BIT(u8, Register),
    INC(Register, i32),
    DEC(Register, i32),
    DEC_F(Register, i32),
    INC_F(Register, i32),
    RET(i32),
    RETI(i32),
    RET_C(Cond, i32)
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
    SP, // Stack Pointer
    PC // Program Counter
}

fn get_arg(start:usize, num:u8, res:&Vec<u8>) -> u16 {
    match num {
        3 => ((res[start + 2] as u16) << 8) + (res[start + 1] as u16) ,
        _ => 0
    } 
}

fn get_cb(start:usize, y:&Vec<u8>) -> (usize, OpCode) {
    match y[start + 1] {
        0x7C => (start + 2, OpCode::BIT(7, Register::H)),
        _ => (start + 2, OpCode::ERR(format!("{:0>2X}", y[start])))
    }
}

fn lookup_LD_R(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0x40;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::LD_R(registers[(idx/8) as usize], registers[(idx%8) as usize], 4)) 
}

fn lookup_ADD(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0x80;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::ADD(Register::A, registers[(idx%8) as usize], 4)) 
}

fn lookup_ADD_C(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0x80;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::ADD_C(Register::A, registers[(idx%8) as usize], 4)) 
}

//TODO Make sure the HL operations for all the lookups are 8 instead of 4
fn lookup_SUB(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0x90;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::SUB(registers[(idx%8) as usize], 4)) 
}

fn lookup_SUB_C(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0x90;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::SUB_C(Register::A, registers[(idx%8) as usize], 4)) 
}

fn lookup_AND(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0xA0;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::AND(registers[(idx%8) as usize], 4)) 
}

fn lookup_XOR(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0xA0;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::XOR(registers[(idx%8) as usize], 4)) 
}
fn lookup_CP(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0xB0;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::CP(registers[(idx%8) as usize], 4)) 
}
fn lookup_OR(start:usize, b:u8) -> (usize, OpCode){
  let idx = b - 0xB0;
  let registers = [Register::B, Register::C, Register::D, Register::E, Register::H, Register::L, Register::HL, Register::A];
  (start + 1, OpCode::OR(registers[(idx%8) as usize], 4)) 
}

fn lookup_op(start:usize, y:&Vec<u8>) -> (usize, OpCode) {
    let res = match y[start] {
        0x00 => (start + 1, OpCode::NOP(4)),
        0x10 => (start + 2, OpCode::STOP(4)),
        0x20 => (2 + start, OpCode::JR_C(Cond::NZ, y[start + 1], 12)), // 12/8 The first arg should be a signed byte
        0x30 => (2 + start, OpCode::JR_C(Cond::NC, y[start + 1], 12)), //12/8 The first arg should be a signed byte
        0x01 => (3 + start, OpCode::LD(Register::BC, B::Two(get_arg(start, 3, y)), 12)),
        0x11 => (3 + start, OpCode::LD(Register::DE, B::Two(get_arg(start, 3, y)), 12)),
        0x21 => (3 + start, OpCode::LD(Register::HL, B::Two(get_arg(start, 3, y)), 12)),
        0x31 => (3 + start, OpCode::LD(Register::SP, B::Two(get_arg(start, 3, y)), 12)),
        0x02 => (1 + start, OpCode::LD_R(Register::BC, Register::A, 8)),
        0x12 => (1 + start, OpCode::LD_R(Register::DE, Register::A, 8)),
        0x22 => (1 + start, OpCode::LD_R(Register::HLP, Register::A, 8)),
        0x32 => (1 + start, OpCode::LD_R(Register::HLM, Register::A, 8)),
        0x03 => (1 + start, OpCode::INC(Register::BC, 8)),
        0x13 => (1 + start, OpCode::INC(Register::DE, 8)),
        0x23 => (1 + start, OpCode::INC(Register::HL, 8)),
        0x33 => (1 + start, OpCode::INC(Register::SP, 8)),
        0x04 => (1 + start, OpCode::INC_F(Register::B, 4)),
        0x14 => (1 + start, OpCode::INC_F(Register::D, 4)),
        0x24 => (1 + start, OpCode::INC_F(Register::H, 4)),
        0x34 => (1 + start, OpCode::INC_F(Register::HL, 12)),
        0x05 => (1 + start, OpCode::DEC_F(Register::B, 4)),
        0x15 => (1 + start, OpCode::DEC_F(Register::D, 4)),
        0x25 => (1 + start, OpCode::DEC_F(Register::H, 4)),
        0x35 => (1 + start, OpCode::DEC_F(Register::HL, 12)),
        0x06 => (2 + start, OpCode::LD(Register::B, B::One(y[start + 1]), 8)),
        0x16 => (2 + start, OpCode::LD(Register::D, B::One(y[start + 1]), 8)),
        0x26 => (2 + start, OpCode::LD(Register::H, B::One(y[start + 1]), 8)),
        0x36 => (2 + start, OpCode::LD(Register::HL, B::One(y[start + 1]), 12)),
        0x09 => (1 + start, OpCode::ADD(Register::HL, Register::BC, 8)),
        0x19 => (1 + start, OpCode::ADD(Register::HL, Register::DE, 8)),
        0x29 => (1 + start, OpCode::ADD(Register::HL, Register::HL, 8)),
        0x39 => (1 + start, OpCode::ADD(Register::HL, Register::SP, 8)),
        0x0B => (1 + start, OpCode::DEC(Register::BC, 8)),
        0x1B => (1 + start, OpCode::DEC(Register::DE, 8)),
        0x2B => (1 + start, OpCode::DEC(Register::HL, 8)),
        0x3B => (1 + start, OpCode::DEC(Register::SP, 8)),
        0x0C => (1 + start, OpCode::INC_F(Register::C, 4)),
        0x1C => (1 + start, OpCode::INC_F(Register::E, 4)),
        0x2C => (1 + start, OpCode::INC_F(Register::L, 4)),
        0x3C => (1 + start, OpCode::INC_F(Register::A, 4)),
        0x0D => (1 + start, OpCode::DEC_F(Register::C, 4)),
        0x1D => (1 + start, OpCode::DEC_F(Register::E, 4)),
        0x2D => (1 + start, OpCode::DEC_F(Register::L, 4)),
        0x3D => (1 + start, OpCode::DEC_F(Register::A, 4)),
        0x0E => (2 + start, OpCode::LD(Register::C, B::One(y[start + 1]), 8)),
        0x1E => (2 + start, OpCode::LD(Register::E, B::One(y[start + 1]), 8)),
        0x2E => (2 + start, OpCode::LD(Register::L, B::One(y[start + 1]), 8)),
        0x3E => (2 + start, OpCode::LD(Register::A, B::One(y[start + 1]), 8)),
        0x0A => (1 + start, OpCode::LD_R(Register::A, Register::BC, 8)),
        0x1A => (1 + start, OpCode::LD_R(Register::A, Register::DE, 8)),
        0x2A => (1 + start, OpCode::LD_R(Register::A, Register::HLP, 8)),
        0x3A => (1 + start, OpCode::LD_R(Register::A, Register::HLM, 8)),
        0xE0 => (2 + start, OpCode::LD_FF_I(y[start + 1], Register::A, 12)),
        0xE2 => (1 + start, OpCode::LD_FF(Register::C, Register::A, 8)),
        0xCB => get_cb(start, y),
        0xCD => (3 + start, OpCode::CALL(B::Two(get_arg(start, 3, y)), 24)),
        0x76 => (1 + start, OpCode::HALT(4)), 
        b @ 0x40...0x7F => lookup_LD_R(start, b),
        b @ 0x80...0x88 => lookup_ADD(start, b),
        b @ 0x88...0x8F => lookup_ADD_C(start, b),
        b @ 0x90...0x98 => lookup_SUB(start, b),
        b @ 0x98...0x9F => lookup_SUB_C(start, b),
        b @ 0xA0...0xA8 => lookup_AND(start, b),
        b @ 0xA8...0xAF => lookup_XOR(start, b),
        b @ 0xB0...0xB8 => lookup_OR(start, b),
        b @ 0xB8...0xBF => lookup_CP(start, b),
        0xC0 => (1 + start, OpCode::RET_C(Cond::NZ, 8)), // actually 20/8
        0xD0 => (1 + start, OpCode::RET_C(Cond::NC, 8)), // actually 20/8
        0xC1 => (1 + start, OpCode::POP(Register::BC, 12)),
        0xD1 => (1 + start, OpCode::POP(Register::DE, 12)),
        0xE1 => (1 + start, OpCode::POP(Register::HL, 12)),
        0xF1 => (1 + start, OpCode::POP(Register::AF, 12)),
        0xC5 => (1 + start, OpCode::PUSH(Register::BC, 16)),
        0xD5 => (1 + start, OpCode::PUSH(Register::DE, 16)),
        0xE5 => (1 + start, OpCode::PUSH(Register::HL, 16)),
        0xF5 => (1 + start, OpCode::PUSH(Register::AF, 16)),
        0xC8 => (1 + start, OpCode::RET_C(Cond::Z, 8)), // actually 20/8
        0xD8 => (1 + start, OpCode::RET_C(Cond::C, 8)), // actually 20/8
        _ => (start + 1, OpCode::ERR(format!("{:0>2X}", y[start])))
    };
    res
}

fn main() {
    let mut f = File::open("DMG_ROM.bin").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).ok();
    let start = 0x000;
    let res: Vec<String> = buffer.clone()
        .into_iter()
        .map(|x| format!("{:0>2X}", x))
        .skip(start)
        .take(500)
        .collect();
    
    let mut next_addr = 0;
    for v in 1..256 {
        let (new_addr, instr) = lookup_op(next_addr, &buffer);
        next_addr = new_addr;
        println!("{:?}", instr); 
    }
}