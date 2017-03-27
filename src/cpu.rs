use std::fs::File;
use std::io::prelude::*;

pub struct Cpu {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub memory: [u8; 0x10000],
    pub boot_rom: Vec<u8>,
    pub cart_rom: Vec<u8>,
    pub has_booted: bool
}

impl Cpu {
    pub fn load_bootrom(&mut self, path: &str) {
        let mut f = File::open(path).unwrap();
        f.read_to_end(&mut self.boot_rom).ok();
    }

    pub fn load_cart(&mut self, path: &str) {
        let mut f = File::open(path).unwrap();
        f.read_to_end(&mut self.cart_rom).ok();
    }
}

impl Default for Cpu {
    fn default() -> Cpu { 
        Cpu { 
            a: 0, b: 0, c:0, d:0, e:0, f:0, h:0, l:0, sp:0, pc: 0, has_booted: false,
            memory: [0; 0x10000], boot_rom: Vec::new(), cart_rom: Vec::new() 
        }
    }
}

pub fn write_address(address: usize, val: u8, cpu: &mut Cpu) -> () { 
    cpu.memory[address] = val;
}

pub fn read_address(address: usize, cpu: &mut Cpu) -> u8 { 
    match address {
        0...0x00FF => { if cpu.has_booted { read_cart_address(address, cpu) } else { cpu.boot_rom[address] } },
        0x0100...0x7FFF => read_cart_address(address, cpu),
        _ => cpu.memory[address]
    }
}

pub fn read_cart_address(address: usize, cpu: &mut Cpu) -> u8 {
    cpu.cart_rom[address]
}

pub fn stack_push(val: u16, cpu: &mut Cpu) -> () {
  let (l_byte, r_byte) = ((val >> 8) as u8, (0x00FF & val) as u8);
  cpu.sp = cpu.sp.wrapping_sub(2); 
  cpu.memory[cpu.sp as usize] = r_byte;
  cpu.memory[(cpu.sp + 1) as usize] = l_byte;
}

pub fn stack_pop(cpu: &mut Cpu) -> u16 {
  let r_byte = cpu.memory[cpu.sp as usize] as u16;
  let l_byte = cpu.memory[(cpu.sp + 1) as usize] as u16;
  cpu.sp = cpu.sp.wrapping_add(2); 
  let res = (l_byte << 8) + r_byte;
  res
}