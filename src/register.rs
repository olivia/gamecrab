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
        DE,
        HL,
        HLP,
        HLM,
        ADDR(u16),
        SP, // Stack Pointer
        SP_OFF(i8), // stack pointer + offset
        PC // Program Counter
    }