use std::io::prelude::*;
use std::fs::File;

#[derive(Debug)]
enum B {
    One(u8),
    Two(u16)
}

#[derive(Debug)]
enum OpCode {
    NOP((), i32),
    JP((), i32),
    JR_NZ((u8), i32),
    CALL(B, i32),
    LD(Register, B, i32),
    LD_R(Register, Register, i32),
    LD_FF(Register, Register, i32),
    LD_FF_I(u8, Register, i32),
    LD_HLM_A((), i32),
    XOR_A((), i32),
    ADD,
    ERR(String),
    BIT(u8, Register),
    INC(Register, i32)
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

fn lookup_op(start:usize, y:&Vec<u8>) -> (usize, OpCode) {
    let res = match y[start] {
        0x00 => (start + 1, OpCode::NOP((), 4)),
        0x0E => (2 + start, OpCode::LD(Register::C, B::One(y[start + 1]), 8)),
        0x3E => (2 + start, OpCode::LD(Register::A, B::One(y[start + 1]), 8)),
        0x11 => (3 + start, OpCode::LD(Register::DE, B::Two(get_arg(start, 3, y)), 12)),
        0x31 => (3 + start, OpCode::LD(Register::SP, B::Two(get_arg(start, 3, y)), 12)),
        0x20 => (2 + start, OpCode::JR_NZ(y[start + 1], 8)), //The first arg should be a signed byte
        0x32 => (1 + start, OpCode::LD_HLM_A((), 8)),
        0xAF => (start + 1, OpCode::XOR_A((),4)),
        0x21 => (3 + start, OpCode::LD(Register::HL, B::Two(get_arg(start, 3, y)), 12)),
        0xE0 => (2 + start, OpCode::LD_FF_I(y[start + 1], Register::A, 12)),
        0xE2 => (1 + start, OpCode::LD_FF(Register::C, Register::A, 8)),
        0x0C => (1 + start, OpCode::INC(Register::C, 4)),
        0x1A => (1 + start, OpCode::LD_R(Register::A, Register::DE, 8)),
        0xCB => get_cb(start, y),
        0xCD => (3 + start, OpCode::CALL(B::Two(get_arg(start, 3, y)), 24)),
        b @ 0x40...0x7F => lookup_LD_R(start, b),
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
    for v in 1..100 {
        let (new_addr, instr) = lookup_op(next_addr, &buffer);
        next_addr = new_addr;
        println!("{:?}", instr); 
    }
}