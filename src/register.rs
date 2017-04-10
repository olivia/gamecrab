use cpu::*;
use utility::*;

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
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
    PC, // Program Counter
}

pub fn read_register(reg: Register, cpu: &mut Cpu) -> u8 {
    use self::Register::*;
    match reg {
        A => cpu.a,
        B => cpu.b,
        C => cpu.c,
        CH => safe_read_address(0xFF00 + read_register(C, cpu) as usize, cpu),
        D => cpu.d,
        E => cpu.e,
        F => cpu.f,
        H => cpu.h,
        L => cpu.l,
        HL_ADDR => safe_read_address(read_multi_register(HL, cpu) as usize, cpu),
        BC_ADDR => safe_read_address(read_multi_register(BC, cpu) as usize, cpu),
        DE_ADDR => safe_read_address(read_multi_register(DE, cpu) as usize, cpu),
        ADDR(addr) => safe_read_address(addr as usize, cpu),
        SP_OFF(offset) => safe_read_address(wrapping_off_u16_i8(cpu.sp, offset) as usize, cpu),
        _ => {
            println!("Failed to read {:?}", reg);
            unreachable!()
        }
    }
}

pub fn read_multi_register(reg: Register, cpu: &mut Cpu) -> u16 {
    use self::Register::*;
    match reg {
        HL => ((cpu.h as u16) << 8) + (cpu.l as u16),
        AF => ((cpu.a as u16) << 8) + (cpu.f as u16),
        BC => ((cpu.b as u16) << 8) + (cpu.c as u16),
        DE => ((cpu.d as u16) << 8) + (cpu.e as u16),
        SP => cpu.sp,
        _ => unreachable!(),
    }
}

pub fn write_register(reg: Register, val: u8, cpu: &mut Cpu) -> () {
    use self::Register::*;
    match reg {
        A => cpu.a = val,
        B => cpu.b = val,
        C => cpu.c = val,
        D => cpu.d = val,
        E => cpu.e = val,
        F => cpu.f = val & 0xF0,
        H => cpu.h = val,
        L => cpu.l = val,
        CH => safe_write_address(0xFF00 + read_register(C, cpu) as usize, val, cpu),
        HL_ADDR => safe_write_address(read_multi_register(HL, cpu) as usize, val, cpu),
        BC_ADDR => safe_write_address(read_multi_register(BC, cpu) as usize, val, cpu),
        DE_ADDR => safe_write_address(read_multi_register(DE, cpu) as usize, val, cpu),
        ADDR(address) => safe_write_address(address as usize, val, cpu),
        _ => unreachable!(),
    }
}

pub fn write_multi_register(reg: Register, val: u16, cpu: &mut Cpu) -> () {
    use self::Register::*;
    let (l_byte, r_byte) = ((val >> 8) as u8, (0x00FF & val) as u8);
    match reg {
        HL => {
            cpu.h = l_byte;
            cpu.l = r_byte;
        }
        AF => {
            cpu.a = l_byte;
            cpu.f = r_byte & 0xF0;
        }
        BC => {
            cpu.b = l_byte;
            cpu.c = r_byte;
        }
        DE => {
            cpu.d = l_byte;
            cpu.e = r_byte;
        }
        SP => cpu.sp = val,
        _ => unreachable!(),
    };
}