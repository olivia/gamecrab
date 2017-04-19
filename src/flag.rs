use cpu::*;
use register::*;

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

pub trait FlagSetting {
    fn mod_flag(&self, reg: u8) -> u8;
}

impl FlagSetting for (Flag, bool) {
    fn mod_flag(&self, reg: u8) -> u8 {
        let (flag, cond) = *self;
        let b = flag_bit(flag);
        if cond {
            reg | b
        } else {
            reg & !b
        }
    }
}

impl<A: FlagSetting, B: FlagSetting, C: FlagSetting> FlagSetting
    for (A, B, C) {
    fn mod_flag(&self, reg: u8) -> u8 {
        let (ref a, ref b, ref c) = *self;
        c.mod_flag(b.mod_flag(a.mod_flag(reg)))
    }
}

impl<A: FlagSetting, B: FlagSetting, C: FlagSetting, D: FlagSetting> FlagSetting
    for (A, B, C, D) {
    fn mod_flag(&self, reg: u8) -> u8 {
        let (ref a, ref b, ref c, ref d) = *self;
        d.mod_flag(c.mod_flag(b.mod_flag(a.mod_flag(reg))))
    }
}

pub fn mod_flags<T: FlagSetting>(settings: T, cpu: &mut Cpu) {
    write_register(Register::F,
                   settings.mod_flag(read_register(Register::F, cpu)),
                   cpu)
}

// set if bool is true, reset if false
pub fn bool_set(flag: Flag, b: bool, cpu: &mut Cpu) {
    mod_flags((flag, b), cpu)
}

pub fn set(flag: Flag, cpu: &mut Cpu) -> () {
    mod_flags((flag, true), cpu)
}

pub fn reset(flag: Flag, cpu: &mut Cpu) -> () {
    mod_flags((flag, false), cpu)
}

pub fn is_set(flag: Flag, cpu: &mut Cpu) -> bool {
    read_register(Register::F, cpu) & flag_bit(flag) != 0
}

pub fn flag_bit(flag: Flag) -> u8 {
    use self::Flag::*;
    1 <<
    match flag {
        Z => 7, 
        N => 6,
        H => 5,
        C => 4,
    }
}
