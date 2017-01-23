use register::*;

#[derive(Debug)]
pub enum B {
    One(u8),
    Two(u16)
}

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
    CALL(Register),
    CALL_C(Cond, Register),
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
    LD(Register, B),
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