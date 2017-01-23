#[derive(Debug, Copy, Clone)]
pub enum Register {
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
    BC_ADDR,
    DE_ADDR,
    DE,
    HL,
    HL_ADDR,
    HLP,
    HLM,
    ADDR(u16),
    SP, // Stack Pointer
    SP_OFF(i8), // stack pointer + offset
    PC // Program Counter
}