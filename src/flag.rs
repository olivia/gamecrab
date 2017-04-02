use cpu::*;
use register::*;

#[derive(Debug, Clone, Copy)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

// set if bool is true, reset if false
pub fn bool_set(flag: Flag, b: bool, cpu: &mut Cpu) {
    if b { set(flag, cpu) } else { reset(flag, cpu) }
}

pub fn set(flag: Flag, cpu: &mut Cpu) -> () {
    write_register(Register::F,
                   read_register(Register::F, cpu) | flag_bit(flag),
                   cpu);
}

pub fn reset(flag: Flag, cpu: &mut Cpu) -> () {
    write_register(Register::F,
                   read_register(Register::F, cpu) & (255 - flag_bit(flag)),
                   cpu);
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
