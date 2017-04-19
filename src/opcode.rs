use cpu::Cpu;
use register::*;

#[derive(Debug, Clone, Copy)]
pub enum Cond {
    NZ,
    Z,
    NC,
    C,
}

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
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
    LD_ADDR_SP(u16),
    LDHL_SP(i8),
    LD_SP_HL,
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
    RST(u16),
    SET(u8, Register),
    STOP,
    SUB(Register),
    SUB_d8(u8),
    SUB_C(Register, Register),
    SUB_C_d8(Register, u8),
    XOR(Register),
    XOR_d8(u8),
}

fn read_u8_arg(idx: usize, cpu: &mut Cpu) -> u8 {
    use cpu::*;
    read_address(idx + 1, cpu)
}

fn read_i8_arg(idx: usize, cpu: &mut Cpu) -> i8 {
    use cpu::*;
    read_address(idx + 1, cpu) as i8
}

fn read_u16_arg(idx: usize, cpu: &mut Cpu) -> u16 {
    use cpu::*;
    ((read_address(idx + 2, cpu) as u16) << 8) + (read_address(idx + 1, cpu) as u16)
}

fn get_cb(start: usize, cpu: &mut Cpu) -> (usize, OpCode, usize) {
    use self::OpCode::*;

    let b = read_u8_arg(start, cpu);
    let (mod_register, mod_mult) = (lookup_mod_register(b), lookup_mod_mult(b));

    (2, match b >> 3 {
        0 => RLC(mod_register),
        1 => RRC(mod_register),
        2 => RL(mod_register),
        3 => RR(mod_register),
        4 => SLA(mod_register),
        5 => SRA(mod_register),
        6 => SWAP(mod_register),
        7 => SRL(mod_register),
        0x08...0x0F => BIT((b - 0x40) / 8, mod_register),
        0x10...0x17 => RES((b - 0x80) / 8, mod_register),
        0x18...0x1F => SET((b - 0xC0) / 8, mod_register),
        _ => unreachable!(),
    }, 8 * mod_mult)
}

fn lookup_mod_register(b: u8) -> Register {
    use register::Register::*;
    let registers = [B, C, D, E, H, L, HL_ADDR, A];
    registers[(b % 8) as usize]
}

fn lookup_mod_mult(b: u8) -> usize {
    if (b % 8) == 6 { 2 } else { 1 }
}

pub fn lookup_op(start: usize, cpu: &mut Cpu) -> (usize, OpCode, usize) {
    use self::OpCode::*;
    use cpu::*;
    use register::Register::*;
    let op_byte = read_address(start, cpu);
    let res = match op_byte {
        0x00 => (1, NOP, 4),
        0x10 => (2, STOP, 4),
        0x20 => (2, JR_C(Cond::NZ, read_i8_arg(start, cpu)), 12), // 12/8 The first arg should be a signed byte
        0x30 => (2, JR_C(Cond::NC, read_i8_arg(start, cpu)), 12), //12/8 The first arg should be a signed byte
        0x01 => (3, LD_M(BC, read_u16_arg(start, cpu)), 12),
        0x11 => (3, LD_M(DE, read_u16_arg(start, cpu)), 12),
        0x21 => (3, LD_M(HL, read_u16_arg(start, cpu)), 12),
        0x31 => (3, LD_M(SP, read_u16_arg(start, cpu)), 12),
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
        0x34 => (1, INC_F(HL_ADDR), 12),
        0x05 => (1, DEC_F(B), 4),
        0x15 => (1, DEC_F(D), 4),
        0x25 => (1, DEC_F(H), 4),
        0x35 => (1, DEC_F(HL_ADDR), 12),
        0x06 => (2, LD(B, read_u8_arg(start, cpu)), 8),
        0x16 => (2, LD(D, read_u8_arg(start, cpu)), 8),
        0x26 => (2, LD(H, read_u8_arg(start, cpu)), 8),
        0x36 => (2, LD(HL_ADDR, read_u8_arg(start, cpu)), 12),
        0x07 => (1, RLCA, 4),
        0x17 => (1, RLA, 4),
        0x27 => (1, DAA, 4),
        0x37 => (1, SCF, 4),
        0x08 => (3, LD_ADDR_SP(read_u16_arg(start, cpu)), 20),
        0x18 => (2, JR(read_i8_arg(start, cpu)), 4),
        0x28 => (2, JR_C(Cond::Z, read_i8_arg(start, cpu)), 4), // 12/8
        0x38 => (2, JR_C(Cond::C, read_i8_arg(start, cpu)), 4),
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
        0x0E => (2, LD(C, read_u8_arg(start, cpu)), 8),
        0x1E => (2, LD(E, read_u8_arg(start, cpu)), 8),
        0x2E => (2, LD(L, read_u8_arg(start, cpu)), 8),
        0x3E => (2, LD(A, read_u8_arg(start, cpu)), 8),
        0x0F => (1, RRCA, 4),
        0x1F => (1, RRA, 4),
        0x2F => (1, CPL, 4),
        0x3F => (1, CCF, 4),
        0x0A => (1, LD_R(A, BC_ADDR), 8),
        0x1A => (1, LD_R(A, DE_ADDR), 8),
        0x2A => (1, LD_R(A, HLP), 8),
        0x3A => (1, LD_R(A, HLM), 8),
        0xE0 => (2, LD_R(ADDR(0xFF00 + (read_u8_arg(start, cpu) as u16)), A), 12),
        0xF0 => (2, LD_R(A, ADDR(0xFF00 + (read_u8_arg(start, cpu) as u16))), 12),
        0xC2 => (3, JP_C(Cond::NZ, read_u16_arg(start, cpu)), 12), // 16/12
        0xC3 => (3, JP(read_u16_arg(start, cpu)), 16),
        0xD2 => (3, JP_C(Cond::NC, read_u16_arg(start, cpu)), 12), // 16/12
        0xE2 => (1, LD_R(CH, A), 8),
        0xF2 => (1, LD_R(A, CH), 8),
        0xF3 => (1, DI, 4),
        0xC4 => (3, CALL_C(Cond::NZ, read_u16_arg(start, cpu)), 12), // 24/12
        0xD4 => (3, CALL_C(Cond::NC, read_u16_arg(start, cpu)), 12), // 24/12
        0xCB => get_cb(start, cpu),
        0xFB => (1, EI, 4),
        0xCC => (3, CALL_C(Cond::Z, read_u16_arg(start, cpu)), 12), // 24/12
        0xDC => (3, CALL_C(Cond::C, read_u16_arg(start, cpu)), 12), // 24/12
        0xCD => (3, CALL(read_u16_arg(start, cpu)), 24),
        0x76 => (0, HALT, 4),
        0x40 => (1, LD_R(B, B), 4),
        0x41 => (1, LD_R(B, C), 4),
        0x42 => (1, LD_R(B, D), 4),
        0x43 => (1, LD_R(B, E), 4),
        0x44 => (1, LD_R(B, H), 4),
        0x45 => (1, LD_R(B, L), 4),
        0x46 => (1, LD_R(B, HL_ADDR), 8),
        0x47 => (1, LD_R(B, A), 4),
        0x48 => (1, LD_R(C, B), 4),
        0x49 => (1, LD_R(C, C), 4),
        0x4A => (1, LD_R(C, D), 4),
        0x4B => (1, LD_R(C, E), 4),
        0x4C => (1, LD_R(C, H), 4),
        0x4D => (1, LD_R(C, L), 4),
        0x4E => (1, LD_R(C, HL_ADDR), 8),
        0x4F => (1, LD_R(C, A), 4),
        0x50 => (1, LD_R(D, B), 4),
        0x51 => (1, LD_R(D, C), 4),
        0x52 => (1, LD_R(D, D), 4),
        0x53 => (1, LD_R(D, E), 4),
        0x54 => (1, LD_R(D, H), 4),
        0x55 => (1, LD_R(D, L), 4),
        0x56 => (1, LD_R(D, HL_ADDR), 8),
        0x57 => (1, LD_R(D, A), 4),
        0x58 => (1, LD_R(E, B), 4),
        0x59 => (1, LD_R(E, C), 4),
        0x5A => (1, LD_R(E, D), 4),
        0x5B => (1, LD_R(E, E), 4),
        0x5C => (1, LD_R(E, H), 4),
        0x5D => (1, LD_R(E, L), 4),
        0x5E => (1, LD_R(E, HL_ADDR), 8),
        0x5F => (1, LD_R(E, A), 4),
        0x60 => (1, LD_R(H, B), 4),
        0x61 => (1, LD_R(H, C), 4),
        0x62 => (1, LD_R(H, D), 4),
        0x63 => (1, LD_R(H, E), 4),
        0x64 => (1, LD_R(H, H), 4),
        0x65 => (1, LD_R(H, L), 4),
        0x66 => (1, LD_R(H, HL_ADDR), 8),
        0x67 => (1, LD_R(H, A), 4),
        0x68 => (1, LD_R(L, B), 4),
        0x69 => (1, LD_R(L, C), 4),
        0x6A => (1, LD_R(L, D), 4),
        0x6B => (1, LD_R(L, E), 4),
        0x6C => (1, LD_R(L, H), 4),
        0x6D => (1, LD_R(L, L), 4),
        0x6E => (1, LD_R(L, HL_ADDR), 8),
        0x6F => (1, LD_R(L, A), 4),
        0x70 => (1, LD_R(HL_ADDR, B), 8),
        0x71 => (1, LD_R(HL_ADDR, C), 8),
        0x72 => (1, LD_R(HL_ADDR, D), 8),
        0x73 => (1, LD_R(HL_ADDR, E), 8),
        0x74 => (1, LD_R(HL_ADDR, H), 8),
        0x75 => (1, LD_R(HL_ADDR, L), 8),
        0x77 => (1, LD_R(HL_ADDR, A), 8),
        0x78 => (1, LD_R(A, B), 4),
        0x79 => (1, LD_R(A, C), 4),
        0x7A => (1, LD_R(A, D), 4),
        0x7B => (1, LD_R(A, E), 4),
        0x7C => (1, LD_R(A, H), 4),
        0x7D => (1, LD_R(A, L), 4),
        0x7E => (1, LD_R(A, HL_ADDR), 8),
        0x7F => (1, LD_R(A, A), 4),
        0x80 => (1, ADD(A, B), 4),
        0x81 => (1, ADD(A, C), 4),
        0x82 => (1, ADD(A, D), 4),
        0x83 => (1, ADD(A, E), 4),
        0x84 => (1, ADD(A, H), 4),
        0x85 => (1, ADD(A, L), 4),
        0x86 => (1, ADD(A, HL_ADDR), 8),
        0x87 => (1, ADD(A, A), 4),
        0x88 => (1, ADD_C(A, B), 4),
        0x89 => (1, ADD_C(A, C), 4),
        0x8A => (1, ADD_C(A, D), 4),
        0x8B => (1, ADD_C(A, E), 4),
        0x8C => (1, ADD_C(A, H), 4),
        0x8D => (1, ADD_C(A, L), 4),
        0x8E => (1, ADD_C(A, HL_ADDR), 8),
        0x8F => (1, ADD_C(A, A), 4),
        0x90 => (1, SUB(B), 4),
        0x91 => (1, SUB(C), 4),
        0x92 => (1, SUB(D), 4),
        0x93 => (1, SUB(E), 4),
        0x94 => (1, SUB(H), 4),
        0x95 => (1, SUB(L), 4),
        0x96 => (1, SUB(HL_ADDR), 8),
        0x97 => (1, SUB(A), 4),
        0x98 => (1, SUB_C(A, B), 4),
        0x99 => (1, SUB_C(A, C), 4),
        0x9A => (1, SUB_C(A, D), 4),
        0x9B => (1, SUB_C(A, E), 4),
        0x9C => (1, SUB_C(A, H), 4),
        0x9D => (1, SUB_C(A, L), 4),
        0x9E => (1, SUB_C(A, HL_ADDR), 8),
        0x9F => (1, SUB_C(A, A), 4),
        0xA0 => (1, AND(B), 4),
        0xA1 => (1, AND(C), 4),
        0xA2 => (1, AND(D), 4),
        0xA3 => (1, AND(E), 4),
        0xA4 => (1, AND(H), 4),
        0xA5 => (1, AND(L), 4),
        0xA6 => (1, AND(HL_ADDR), 8),
        0xA7 => (1, AND(A), 4),
        0xA8 => (1, XOR(B), 4),
        0xA9 => (1, XOR(C), 4),
        0xAA => (1, XOR(D), 4),
        0xAB => (1, XOR(E), 4),
        0xAC => (1, XOR(H), 4),
        0xAD => (1, XOR(L), 4),
        0xAE => (1, XOR(HL_ADDR), 8),
        0xAF => (1, XOR(A), 4),
        0xB0 => (1, OR(B), 4),
        0xB1 => (1, OR(C), 4),
        0xB2 => (1, OR(D), 4),
        0xB3 => (1, OR(E), 4),
        0xB4 => (1, OR(H), 4),
        0xB5 => (1, OR(L), 4),
        0xB6 => (1, OR(HL_ADDR), 8),
        0xB7 => (1, OR(A), 4),
        0xB8 => (1, CP(B), 4),
        0xB9 => (1, CP(C), 4),
        0xBA => (1, CP(D), 4),
        0xBB => (1, CP(E), 4),
        0xBC => (1, CP(H), 4),
        0xBD => (1, CP(L), 4),
        0xBE => (1, CP(HL_ADDR), 8),
        0xBF => (1, CP(A), 4),
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
        0xC6 => (2, ADD_d8(A, read_u8_arg(start, cpu)), 8),
        0xD6 => (2, SUB_d8(read_u8_arg(start, cpu)), 8),
        0xE6 => (2, AND_d8(read_u8_arg(start, cpu)), 8),
        0xF6 => (2, OR_d8(read_u8_arg(start, cpu)), 8),
        0xC7 => (1, RST(0x00), 16),
        0xD7 => (1, RST(0x10), 16),
        0xE7 => (1, RST(0x20), 16),
        0xF7 => (1, RST(0x30), 16),
        0xC8 => (1, RET_C(Cond::Z), 8), // actually 20/8
        0xD8 => (1, RET_C(Cond::C), 8), // actually 20/8
        0xE8 => (2, ADD_r8(SP, read_i8_arg(start, cpu)), 16),
        0xF8 => (2, LDHL_SP(read_i8_arg(start, cpu)), 12),
        0xC9 => (1, RET, 16),
        0xD9 => (1, RETI, 16),
        0xE9 => (1, JP_HL, 4),
        0xF9 => (1, LD_SP_HL, 8),
        0xCA => (3, JP_C(Cond::Z, read_u16_arg(start, cpu)), 16), // 16/12
        0xDA => (3, JP_C(Cond::C, read_u16_arg(start, cpu)), 16),
        0xEA => (3, LD_R(ADDR(read_u16_arg(start, cpu)), A), 16),
        0xFA => (3, LD_R(A, ADDR(read_u16_arg(start, cpu))), 16),
        0xCE => (2, ADD_C_d8(A, read_u8_arg(start, cpu)), 8),
        0xDE => (2, SUB_C_d8(A, read_u8_arg(start, cpu)), 8),
        0xEE => (2, XOR_d8(read_u8_arg(start, cpu)), 8),
        0xFE => (2, CP_d8(read_u8_arg(start, cpu)), 8),
        0xCF => (1, RST(0x08), 16),
        0xDF => (1, RST(0x18), 16),
        0xEF => (1, RST(0x28), 16),
        0xFF => (1, RST(0x38), 16),
        _ => {
            println!("Missing {:4>0X} op", op_byte);
            unreachable!()
        }
    };
    res
}

pub fn test_cond(cond: Cond, cpu: &mut Cpu) -> bool {
    use self::Cond::*;
    use flag;
    use flag::Flag;
    match cond {
        Z => flag::is_set(Flag::Z, cpu),
        NZ => !flag::is_set(Flag::Z, cpu),
        C => flag::is_set(Flag::C, cpu),
        NC => !flag::is_set(Flag::C, cpu),
    }
}
