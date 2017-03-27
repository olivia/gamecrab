use cpu::Cpu;
use register::*;

#[derive(Debug)]
pub enum Cond {
    NZ,
    Z,
    NC,
    C
}

#[derive(Debug)]
pub enum OpCode {
    ADD(Register, Register),
    ADD_d8(Register, u8),
    ADD_r8(Register, i8),
    ADD_C_d8(Register, u8),
    ADD_C(Register, Register),
    AND(Register),
    AND_d8(u8),
    BIT(u8, Register),
    CALL(u16),
    CALL_C(Cond, u16),
    CP(Register),
    CP_d8(u8),
    CPL,
    CCF,
    DEC(Register),
    DEC_F(Register),
    DI,
    EI,
    ERR(String),
    HALT,
    INC(Register),
    INC_F(Register),
    JP(u16),
    JP_HL,
    JP_C(Cond, u16),
    JR(i8),
    JR_C(Cond, i8),
    LD(Register, u8),
    LD_M(Register, u16),
    LD_R(Register, Register),
    NOP,
    OR(Register),
    OR_d8(u8),
    POP(Register),
    PUSH(Register),
    RET,
    RETI,
    RET_C(Cond),
    RES(u8, Register),
    RLC(Register),
    RLCA,
    RRC(Register),
    RRCA,
    RL(Register),
    RLA,
    RR(Register),
    RRA,
    DAA,
    SLA(Register),
    SRA(Register),
    SWAP(Register),
    SRL(Register),
    SCF,
    RST(u8),
    SET(u8, Register),
    STOP,
    SUB(Register),
    SUB_d8(u8),
    SUB_C(Register, Register),
    SUB_C_d8(Register, u8),
    XOR(Register),
    XOR_d8(u8)
}

fn read_u16(idx:usize, res:&Vec<u8>) -> u16 {
    ((res[idx + 2] as u16) << 8) + (res[idx + 1] as u16)
}

fn get_cb(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    use self::OpCode::*;

    let b = y[start + 1];
    match y[start + 1] {
        0x00...0x07 => (2, RLC(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x08...0x0F => (2, RRC(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x10...0x17 => (2, RL(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x18...0x1F => (2, RR(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x20...0x27 => (2, SLA(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x28...0x2F => (2, SRA(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x30...0x37 => (2, SWAP(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x38...0x3F => (2, SRL(lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x40...0x7F => (2, BIT((b - 0x40) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0x80...0xBF => (2, RES((b - 0x80) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        0xC0...0xFF => (2, SET((b - 0xC0) / 8, lookup_mod_register(b)), 8 * lookup_mod_mult(b)),
        _ => (2, OpCode::ERR(format!("{:0>2X}", y[start])), 0)
    }
}

fn lookup_mod_register(b:u8) -> Register {
  use register::Register::*;
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

pub fn lookup_op(start:usize, y:&Vec<u8>) -> (usize, OpCode, u8) {
    use self::OpCode::*;
    use register::Register::*;
    let res = match y[start] {
        0x00 => (1, NOP, 4),
        0x10 => (2, STOP, 4),
        0x20 => (2, JR_C(Cond::NZ, y[start + 1] as i8), 12), // 12/8 The first arg should be a signed byte
        0x30 => (2, JR_C(Cond::NC, y[start + 1] as i8), 12), //12/8 The first arg should be a signed byte
        0x01 => (3, LD_M(BC, read_u16(start, y)), 12),
        0x11 => (3, LD_M(DE, read_u16(start, y)), 12),
        0x21 => (3, LD_M(HL, read_u16(start, y)), 12),
        0x31 => (3, LD_M(SP, read_u16(start, y)), 12),
        0x02 => (1, LD_R(BC_ADDR, A), 8),
        0x12 => (1, LD_R(DE_ADDR, A), 8),
        0x22 => (1, LD_R(HLP, Register::A), 8),
        0x32 => (1, LD_R(HLM, Register::A), 8),
        0x03 => (1, INC(BC), 8),
        0x13 => (1, INC(DE), 8),
        0x23 => (1, INC(HL), 8),
        0x33 => (1, INC(SP), 8),
        0x04 => (1, INC_F(B), 4),
        0x14 => (1, INC_F(D), 4),
        0x24 => (1, INC_F(H), 4),
        0x34 => (1, INC_F(HL), 12),
        0x05 => (1, DEC_F(B), 4),
        0x15 => (1, DEC_F(D), 4),
        0x25 => (1, DEC_F(H), 4),
        0x35 => (1, DEC_F(HL), 12),
        0x06 => (2, LD(B, y[start + 1]), 8),
        0x16 => (2, LD(D, y[start + 1]), 8),
        0x26 => (2, LD(H, y[start + 1]), 8),
        0x36 => (2, LD(HL_ADDR, y[start + 1]), 12),
        0x07 => (1, RLCA, 4),
        0x17 => (1, RLA, 4),
        0x27 => (1, DAA, 4),
        0x37 => (1, SCF, 4),
        0x08 => (3, LD_R(ADDR(read_u16(start, y)), SP), 20),
        0x18 => (2, JR(y[start + 1] as i8), 4),
        0x28 => (2, JR_C(Cond::Z, y[start + 1] as i8), 4), // 12/8
        0x38 => (2, JR_C(Cond::C, y[start + 1] as i8), 4),
        0x09 => (1, ADD(HL, BC), 8),
        0x19 => (1, ADD(HL, DE), 8),
        0x29 => (1, ADD(HL, HL), 8),
        0x39 => (1, ADD(HL, SP), 8),
        0x0B => (1, DEC(BC), 8),
        0x1B => (1, DEC(DE), 8),
        0x2B => (1, DEC(HL), 8),
        0x3B => (1, DEC(SP), 8),
        0x0C => (1, INC_F(C), 4),
        0x1C => (1, INC_F(E), 4),
        0x2C => (1, INC_F(L), 4),
        0x3C => (1, INC_F(A), 4),
        0x0D => (1, DEC_F(C), 4),
        0x1D => (1, DEC_F(E), 4),
        0x2D => (1, DEC_F(L), 4),
        0x3D => (1, DEC_F(A), 4),
        0x0E => (2, LD(C, y[start + 1]), 8),
        0x1E => (2, LD(E, y[start + 1]), 8),
        0x2E => (2, LD(L, y[start + 1]), 8),
        0x3E => (2, LD(A, y[start + 1]), 8),
        0x0F => (1, RRCA, 4),
        0x1F => (1, RRA, 4),
        0x2F => (1, CPL, 4),
        0x3F => (1, CCF, 4),
        0x0A => (1, LD_R(A, BC_ADDR), 8),
        0x1A => (1, LD_R(A, DE_ADDR), 8),
        0x2A => (1, LD_R(A, HLP), 8),
        0x3A => (1, LD_R(A, HLM), 8),
        0xE0 => (2, LD_R(ADDR(0xFF00 + (y[start + 1] as u16)), A), 12),
        0xF0 => (2, LD_R(A, ADDR(0xFF00 + (y[start + 1] as u16))), 12),
        0xC2 => (3, JP_C(Cond::NZ, read_u16(start, y)), 16), // 16/12
        0xC3 => (3, JP(read_u16(start, y)), 16),
        0xD2 => (3, JP_C(Cond::NC, read_u16(start, y)), 16), // 16/12
        0xE2 => (1, LD_R(CH, A), 8),
        0xF2 => (1, LD_R(A, CH), 8),
        0xF3 => (1, DI, 4),
        0xD4 => (3, CALL_C(Cond::NC, read_u16(start, y)), 24), // 24/12
        0xCB => get_cb(start, y),
        0xFB => (1, EI, 4),
        0xCC => (3, CALL_C(Cond::Z, read_u16(start, y)), 24), // 24/12
        0xDC => (3, CALL_C(Cond::C, read_u16(start, y)), 24), // 24/12
        0xCD => (3, CALL(read_u16(start, y)), 24),
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
        0xC1 => (1, POP(BC), 12),
        0xD1 => (1, POP(DE), 12),
        0xE1 => (1, POP(HL), 12),
        0xF1 => (1, POP(AF), 12),
        0xC5 => (1, PUSH(BC), 16),
        0xD5 => (1, PUSH(DE), 16),
        0xE5 => (1, PUSH(HL), 16),
        0xF5 => (1, PUSH(AF), 16),
        0xC6 => (2, ADD_d8(A, y[start + 1]), 8),
        0xD6 => (2, SUB_d8(y[start + 1]), 8),
        0xE6 => (2, AND_d8(y[start + 1]), 8),
        0xF6 => (2, OR_d8(y[start + 1]), 8),
        0xC7 => (1, RST(0x00), 8),
        0xD7 => (1, RST(0x10), 16),
        0xE7 => (1, RST(0x20), 16),
        0xF7 => (1, RST(0x30), 16),
        0xC8 => (1, RET_C(Cond::Z), 8), // actually 20/8
        0xD8 => (1, RET_C(Cond::C), 8), // actually 20/8
        0xE8 => (2, ADD_r8(SP, y[start + 1] as i8), 16),
        0xF8 => (2, LD_R(HL, SP_OFF(y[start + 1] as i8)), 12),
        0xC9 => (1, RET, 16),
        0xD9 => (1, RETI, 16),
        0xE9 => (1, JP_HL, 4),
        0xF9 => (1, LD_R(SP, HL), 8),
        0xCA => (3, JP_C(Cond::Z, read_u16(start, y)), 16), // 16/12
        0xDA => (3, JP_C(Cond::C, read_u16(start, y)), 16),
        0xEA => (3, LD_R(ADDR(read_u16(start, y)), A), 16),
        0xFA => (3, LD_R(A, ADDR(read_u16(start, y))), 16),
        0xCE => (2, ADD_C_d8(A, y[start + 1]), 8),
        0xDE => (2, SUB_C_d8(A, y[start + 1]), 8),
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

pub fn test_cond(cond: Cond, cpu: &mut Cpu) -> bool{
   use self::Cond::*;
   use flag;
   use flag::Flag;
   match cond {
       Z => flag::is_set(Flag::Z, cpu),
       NZ => !flag::is_set(Flag::Z, cpu),
       C => flag::is_set(Flag::C, cpu),
       NC => !flag::is_set(Flag::C, cpu)
   } 
}
