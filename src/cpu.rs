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
    pub memory: [u8; 0x10000]
}

impl Default for Cpu {
    fn default() -> Cpu { 
        Cpu { a: 0, b: 0, c:0, d:0, e:0, f:0, h:0, l:0, sp:0, pc: 0, memory: [0; 0x10000] }
    }
}

pub fn write_address(address: usize, val: u8, cpu: &mut Cpu) -> () { 
    cpu.memory[address] = val;
}

pub fn read_address(address: usize, cpu: &mut Cpu) -> u8 { 
    cpu.memory[address]
}

pub fn stack_push(val: u16, cpu: &mut Cpu) -> () {
  let (l_byte, r_byte) = ((val >> 8) as u8, (0x00FF & val) as u8);
  cpu.sp -= 2; 
  cpu.memory[cpu.sp as usize] = r_byte;
  cpu.memory[(cpu.sp + 1) as usize] = l_byte;
}

pub fn stack_pop(cpu: &mut Cpu) -> u16 {
  let r_byte = cpu.memory[cpu.sp as usize] as u16;
  let l_byte = cpu.memory[(cpu.sp + 1) as usize] as u16;
  cpu.sp += 2; 
  l_byte + r_byte
}