extern crate rustgirl;
use std::io::prelude::*;
use std::fs::File;
use rustgirl::register::*;
use rustgirl::opcode::*;


fn get_arg(start:usize, num:u8, res:&Vec<u8>) -> u16 {
    match num {
        3 => ((res[start + 2] as u16) << 8) + (res[start + 1] as u16) ,
        _ => 0
    }
}

fn get_cb(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    let b = y[start + 1];
    match y[start + 1] {
        0x00...0x07 => (2, OpCode::RLC(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x08...0x0F => (2, OpCode::RRC(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x10...0x17 => (2, OpCode::RL(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x18...0x1F => (2, OpCode::RR(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x20...0x27 => (2, OpCode::SLA(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x28...0x2F => (2, OpCode::SRA(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x30...0x37 => (2, OpCode::SWAP(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x38...0x3F => (2, OpCode::SRL(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x40...0x7F => (2, OpCode::BIT((b - 0x40) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x80...0xBF => (2, OpCode::RES((b - 0x80) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0xC0...0xFF => (2, OpCode::SET((b - 0xC0) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        _ => (2, OpCode::ERR(format!("{:0>2X}", y[start])), 0)
    }
}

fn lookup_mod_register(b:u8) -> Register {
  use rustgirl::register::Register::*;
  let registers = [B, C, D, E, H, L, HL_ADDR, A];
  registers[(b % 8) as usize]
}

fn lookup_mod_cycles(b:u8) -> u8 {
  if (b % 8) == 6 { 8 } else { 4 }
}

fn lookup_mod_mult(b:u8) -> u8 {
  if (b % 8) == 6 { 2 } else { 1 }
}

fn lookup_ld_r(b:u8) -> (usize, OpCode, u8){
  let idx = b - 0x40;
  let (left, right) = (lookup_mod_register(idx/8) , lookup_mod_register(b));
  let cycles = if (idx / 8) == 6 || (idx % 8) == 6 { 8 } else { 4 };
  (1, OpCode::LD_R(left, right), cycles)
}

fn lookup_mod_op_a(op:fn(Register, Register) -> OpCode, b:u8) -> (usize, OpCode, u8) {
    (1, op(Register::A, lookup_mod_register(b)), lookup_mod_cycles(b))
}

fn lookup_mod_op(op:fn(Register) -> OpCode, b:u8) -> (usize, OpCode, u8) {
    (1, op(lookup_mod_register(b)), lookup_mod_cycles(b))
}

fn lookup_op(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    use rustgirl::opcode::OpCode::*;
    let res = match y[start] {
        0x00 => (1, NOP, 4),
        0x10 => (2, STOP, 4),
        0x20 => (2, JR_C(Cond::NZ, y[start + 1] as i8), 12), // 12/8 The first arg should be a signed byte
        0x30 => (2, JR_C(Cond::NC, y[start + 1] as i8), 12), //12/8 The first arg should be a signed byte
        0x01 => (3, LD_M(Register::BC, get_arg(start, 3, y)), 12),
        0x11 => (3, LD_M(Register::DE, get_arg(start, 3, y)), 12),
        0x21 => (3, LD_M(Register::HL, get_arg(start, 3, y)), 12),
        0x31 => (3, LD_M(Register::SP, get_arg(start, 3, y)), 12),
        0x02 => (1, LD_R(Register::BC_ADDR, Register::A), 8),
        0x12 => (1, LD_R(Register::DE_ADDR, Register::A), 8),
        0x22 => (1, LD_R(Register::HLP, Register::A), 8),
        0x32 => (1, LD_R(Register::HLM, Register::A), 8),
        0x03 => (1, INC(Register::BC), 8),
        0x13 => (1, INC(Register::DE), 8),
        0x23 => (1, INC(Register::HL), 8),
        0x33 => (1, INC(Register::SP), 8),
        0x04 => (1, INC_F(Register::B), 4),
        0x14 => (1, INC_F(Register::D), 4),
        0x24 => (1, INC_F(Register::H), 4),
        0x34 => (1, INC_F(Register::HL), 12),
        0x05 => (1, DEC_F(Register::B), 4),
        0x15 => (1, DEC_F(Register::D), 4),
        0x25 => (1, DEC_F(Register::H), 4),
        0x35 => (1, DEC_F(Register::HL), 12),
        0x06 => (2, LD(Register::B, y[start + 1]), 8),
        0x16 => (2, LD(Register::D, y[start + 1]), 8),
        0x26 => (2, LD(Register::H, y[start + 1]), 8),
        0x36 => (2, LD(Register::HL_ADDR, y[start + 1]), 12),
        0x07 => (1, RLCA, 4),
        0x17 => (1, RLA, 4),
        0x27 => (1, DAA, 4),
        0x37 => (1, SCF, 4),
        0x08 => (3, LD_R(Register::ADDR(get_arg(start, 3, y)), Register::SP), 20),
        0x18 => (2, JR(y[start + 1] as i8), 4),
        0x28 => (2, JR_C(Cond::Z, y[start + 1] as i8), 4), // 12/8
        0x38 => (2, JR_C(Cond::C, y[start + 1] as i8), 4),
        0x09 => (1, ADD(Register::HL, Register::BC), 8),
        0x19 => (1, ADD(Register::HL, Register::DE), 8),
        0x29 => (1, ADD(Register::HL, Register::HL), 8),
        0x39 => (1, ADD(Register::HL, Register::SP), 8),
        0x0B => (1, DEC(Register::BC), 8),
        0x1B => (1, DEC(Register::DE), 8),
        0x2B => (1, DEC(Register::HL), 8),
        0x3B => (1, DEC(Register::SP), 8),
        0x0C => (1, INC_F(Register::C), 4),
        0x1C => (1, INC_F(Register::E), 4),
        0x2C => (1, INC_F(Register::L), 4),
        0x3C => (1, INC_F(Register::A), 4),
        0x0D => (1, DEC_F(Register::C), 4),
        0x1D => (1, DEC_F(Register::E), 4),
        0x2D => (1, DEC_F(Register::L), 4),
        0x3D => (1, DEC_F(Register::A), 4),
        0x0E => (2, LD(Register::C, y[start + 1]), 8),
        0x1E => (2, LD(Register::E, y[start + 1]), 8),
        0x2E => (2, LD(Register::L, y[start + 1]), 8),
        0x3E => (2, LD(Register::A, y[start + 1]), 8),
        0x0F => (1, RRCA, 4),
        0x1F => (1, RRA, 4),
        0x2F => (1, CPL, 4),
        0x3F => (1, CCF, 4),
        0x0A => (1, LD_R(Register::A, Register::BC), 8),
        0x1A => (1, LD_R(Register::A, Register::DE), 8),
        0x2A => (1, LD_R(Register::A, Register::HLP), 8),
        0x3A => (1, LD_R(Register::A, Register::HLM), 8),
        0xE0 => (2, LD_R(Register::ADDR(0xFF00 + (y[start + 1] as u16)), Register::A), 12),
        0xF0 => (2, LD_R(Register::A, Register::ADDR(0xFF00 + (y[start + 1] as u16))), 12),
        0xC2 => (3, JP_C(Cond::NZ, get_arg(start, 3, y)), 16), // 16/12
        0xD2 => (3, JP_C(Cond::NC, get_arg(start, 3, y)), 16), // 16/12
        0xE2 => (1, LD_R(Register::CH, Register::A), 8),
        0xF2 => (1, LD_R(Register::A, Register::CH), 8),
        0xC3 => (3, JP(get_arg(start, 3, y)), 16),
        0xF3 => (1, DI, 4),
        0xC4 => (3, CALL_C(Cond::NZ, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xD4 => (3, CALL_C(Cond::NC, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xCB => get_cb(start, y),
        0xFB => (1, EI, 4),
        0xCC => (3, CALL_C(Cond::Z, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xDC => (3, CALL_C(Cond::C, Register::ADDR(get_arg(start, 3, y))), 24), // 24/12
        0xCD => (3, CALL(Register::ADDR(get_arg(start, 3, y))), 24),
        0x76 => (1, HALT, 4),
        b @ 0x40...0x7F => lookup_ld_r(b), //All the registers that use HL have the wrong cycle count
        b @ 0x80...0x88 => lookup_mod_op_a(ADD, b),
        b @ 0x88...0x8F => lookup_mod_op_a(ADD_C, b),
        b @ 0x90...0x98 => lookup_mod_op(SUB, b),
        b @ 0x98...0x9F => lookup_mod_op_a(SUB_C, b),
        b @ 0xA0...0xA8 => lookup_mod_op(AND, b),
        b @ 0xA8...0xAF => lookup_mod_op(XOR, b),
        b @ 0xB0...0xB8 => lookup_mod_op(OR, b),
        b @ 0xB8...0xBF => lookup_mod_op(CP, b),
        0xC0 => (1, RET_C(Cond::NZ), 8), // actually 20/8
        0xD0 => (1, RET_C(Cond::NC), 8), // actually 20/8
        0xC1 => (1, POP(Register::BC), 12),
        0xD1 => (1, POP(Register::DE), 12),
        0xE1 => (1, POP(Register::HL), 12),
        0xF1 => (1, POP(Register::AF), 12),
        0xC5 => (1, PUSH(Register::BC), 16),
        0xD5 => (1, PUSH(Register::DE), 16),
        0xE5 => (1, PUSH(Register::HL), 16),
        0xF5 => (1, PUSH(Register::AF), 16),
        0xC6 => (2, ADD_d8(Register::A, y[start + 1]), 8),
        0xD6 => (2, SUB_d8(y[start + 1]), 8),
        0xE6 => (2, AND_d8(y[start + 1]), 8),
        0xF6 => (2, OR_d8(y[start + 1]), 8),
        0xC7 => (1, RST(0x00), 8),
        0xD7 => (1, RST(0x10), 16),
        0xE7 => (1, RST(0x20), 16),
        0xF7 => (1, RST(0x30), 16),
        0xC8 => (1, RET_C(Cond::Z), 8), // actually 20/8
        0xD8 => (1, RET_C(Cond::C), 8), // actually 20/8
        0xE8 => (2, ADD_r8(Register::SP, y[start + 1] as i8), 16),
        0xF8 => (2, LD_R(Register::HL, Register::SP_OFF(y[start + 1] as i8)), 12),
        0xC9 => (1, RET, 16),
        0xD9 => (1, RETI, 16),
        0xE9 => (1, JP_HL, 4),
        0xF9 => (1, LD_R(Register::SP, Register::HL), 8),
        0xCA => (3, JP_C(Cond::Z, get_arg(start, 3, y)), 16), // 16/12
        0xDA => (3, JP_C(Cond::C, get_arg(start, 3, y)), 16),
        0xEA => (3, LD_R(Register::ADDR(get_arg(start, 3, y)), Register::A), 16),
        0xFA => (3, LD_R(Register::A, Register::ADDR(get_arg(start, 3, y))), 16),
        0xCE => (2, ADD_C_d8(Register::A, y[start + 1]), 8),
        0xDE => (2, SUB_C_d8(Register::A, y[start + 1]), 8),
        0xEE => (2, XOR_d8(y[start + 1]), 8),
        0xFE => (2, CP_d8(y[start + 1]), 8),
        0xCF => (1, RST(0x08), 16),
        0xDF => (1, RST(0x18), 16),
        0xEF => (1, RST(0x28), 16),
        0xFF => (1, RST(0x38), 16),
        _ => (1, OpCode::ERR(format!("{:0>2X}", y[start])), 0)
    };
    res
}

fn exec_ld_m(reg: Register, val: u16, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => { write_multi_register(reg, val, cpu); curr_addr }
    }
}

fn exec_ld(reg: Register, val: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => { write_register(reg, val, cpu); curr_addr }
    }
}

fn exec_xor(reg: Register, cpu: &mut Cpu) -> () {
    use rustgirl::register::Register::*;
    let reg_a_val = read_register(A, cpu);
    let reg_val = read_register(reg, cpu);
    let res = reg_a_val^reg_val;
    let res_f = (if res == 0 { 1 } else { 0 }) << 7;

    match reg {
        _ => { 
            write_register(A, res, cpu);
            write_register(F, res_f, cpu);
        }
    }
}


fn exec_instr(op: OpCode, curr_addr: usize, cpu: &mut Cpu) -> usize {
    use rustgirl::opcode::OpCode::*;
    match op {
        JP(addr) => addr as usize,
        JP_HL => read_multi_register(Register::HL, cpu) as usize,
        NOP => curr_addr,
        XOR(reg) => { exec_xor(reg, cpu); curr_addr },
        LD(reg, val) => exec_ld(reg, val, curr_addr, cpu),
        LD_M(reg, val) => exec_ld_m(reg, val, curr_addr, cpu),
        _ => unreachable!()
    }
}

fn write_multi_register(reg: Register, val: u16, cpu: &mut Cpu) -> () {
   use rustgirl::register::Register::*;
   let (l_byte, r_byte) = ((val >> 8) as u8, (0x0F & val) as u8);
   match reg {
       HL => { cpu.h = l_byte; cpu.l = r_byte; },
       AF => { cpu.a = l_byte; cpu.f = r_byte; },
       BC => { cpu.b = l_byte; cpu.c = r_byte; },
       DE => { cpu.d = l_byte; cpu.e = r_byte; },
       SP => cpu.sp = val,
       _ => unreachable!()
   };
}

fn read_register(reg: Register, cpu: &mut Cpu) -> u8 {
   use rustgirl::register::Register::*;
   match reg {
       A => cpu.a,
       B => cpu.b,
       C => cpu.c,
       D => cpu.d,
       E => cpu.e,
       F => cpu.f,
       H => cpu.h,
       L => cpu.l,
       _ => unreachable!()
   } 
}

fn write_register(reg: Register, val: u8, cpu: &mut Cpu) -> () {
   use rustgirl::register::Register::*;
   match reg {
       A => cpu.a = val,
       B => cpu.b = val,
       C => cpu.c = val,
       D => cpu.d = val,
       E => cpu.e = val,
       F => cpu.f = val,
       H => cpu.h = val,
       L => cpu.l = val,
       _ => unreachable!()
   } 
}

fn read_multi_register(reg: Register, cpu: &mut Cpu) -> u16 {
   use rustgirl::register::Register::*;
   match reg {
       HL => ((cpu.h as u16) << 8)  + (cpu.l as u16),
       AF => ((cpu.a as u16) << 8)  + (cpu.f as u16),
       BC => ((cpu.b as u16) << 8)  + (cpu.c as u16),
       DE => ((cpu.d as u16) << 8)  + (cpu.e as u16),
       SP => cpu.sp,
       PC => cpu.pc,
       _ => unreachable!()
   } 
}

struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16
}

fn main() {
    // Representing A, F, B, C, D, E, H, L in that order
    let mut cpu = Cpu { a: 0, b: 0, c:0, d:0, e:0, f:0, h:0, l:0, sp:0, pc: 0};
    let mut f = File::open("DMG_ROM.bin").unwrap();
//    let mut f = File::open("kirby.gb").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).ok();
    let mut next_addr = 0;
    for _ in 1..256 {
        let (op_length, instr, cycles) = lookup_op(next_addr, &buffer);
        println!("Address {:4>0X}: {:?} taking {:?} cycles", next_addr, instr, cycles);
        next_addr += op_length;
        next_addr = exec_instr(instr, next_addr, &mut cpu);
    }
}
