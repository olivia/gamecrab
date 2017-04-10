use std::fs::File;
use std::io::prelude::*;
use interrupt::*;
use lcd::*;

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
    pub has_booted: bool,
    pub interrupt_master_enabled: bool,
    pub curr_clocks: u32,
    pub curr_freq_clocks: u32,
    pub key_a: bool,
    pub key_b: bool,
    pub key_select: bool,
    pub key_start: bool,
    pub key_up: bool,
    pub key_down: bool,
    pub key_left: bool,
    pub key_right: bool,
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

    pub fn inc_clocks(&mut self, clocks: usize) {
        // increment div
        self.curr_clocks += clocks as u32;
        if self.curr_clocks > 256 {
            self.curr_clocks %= 256;
            let div = read_address(0xFF04, self);
            write_address(0xFF04, div.wrapping_add(1), self);
        }

        if TAC::Enabled.is_set(self) {
            println!("increment div");
            // increment
            self.curr_freq_clocks += clocks as u32;
            let freq = get_tac_freq(self);
            if self.curr_freq_clocks > freq {
                self.curr_freq_clocks %= freq;
                let tma = read_address(0xFF06, self);
                let tima = read_address(0xFF05, self);
                // overflow
                if tima == 0xFF {
                    write_address(0xFF05, tma, self);
                    Interrupt::Timer.request(self);
                    println!("Overflow");
                } else {
                    write_address(0xFF05, tima + 1, self);
                }
            }
        }
    }
}

pub enum TAC {
    Enabled,
    Freq1024,
    Freq16,
    Freq64,
    Freq256,
}

impl TAC {
    pub fn is_set(&self, cpu: &mut Cpu) -> bool {
        use self::TAC::*;
        let tac = read_address(0xFF07, cpu);
        let bit2 = tac & 0b100;
        let bit10 = tac & 0b11;
        match *self {
            Enabled => bit2 != 0,
            Freq1024 => bit10 == 0,
            Freq16 => bit10 == 1,
            Freq64 => bit10 == 2,
            Freq256 => bit10 == 3,
        }
    }
}

fn get_tac_freq(cpu: &mut Cpu) -> u32 {
    let tac = read_address(0xFF07, cpu);
    let bit10 = tac & 0b11;
    match bit10 {
        0 => 1024,
        1 => 16,
        2 => 64,
        3 => 256,
        _ => unreachable!(),
    }
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            has_booted: false,
            interrupt_master_enabled: false,
            memory: [0; 0x10000],
            boot_rom: Vec::new(),
            cart_rom: Vec::new(),
            curr_clocks: 0,
            curr_freq_clocks: 0,
            key_a: false,
            key_b: false,
            key_select: false,
            key_start: false,
            key_up: false,
            key_down: false,
            key_left: false,
            key_right: false,
        }
    }
}

pub fn safe_write_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    let safe_to_write = match address {
        0x8000...0x9FFF => !LCDC::Power.is_set(cpu) || !ScreenMode::Transferring.is_set(cpu),
        0xFE00...0xFE9F => {
            // OAM
            !LCDC::Power.is_set(cpu) ||
            (ScreenMode::HBlank.is_set(cpu) || ScreenMode::VBlank.is_set(cpu))
        } 
        0x0150...0x7FFF => false, //ROM, incorporate bank switching in the future
        0xFEA0...0xFEFF => false, //Unused memory
        _ => true,
    };
    if safe_to_write {
        match address {
            0xE000...0xFDFF => write_address(address - 0x1000, val, cpu),
            0xFF04 => cpu.memory[address] = 0,
            0xFF46 => dma_transfer(val, cpu), //this needs to be synced with clocks
            0xFF41 => write_stat_address(val, cpu),
            0xFF50 => {
                cpu.has_booted = true;
                println!("==================BOOTED==================");
            }
            0xFF44 => write_address(address, 0, cpu),
            0xFF00 => write_joypad(val, cpu),

            _ => write_address(address, val, cpu),
        }
    }
}

fn write_joypad(new_val: u8, cpu: &mut Cpu) {
    let val = read_joypad(cpu);
    write_address(0xFF00, (new_val & 0xF0) | val & 0x0F, cpu);
}

pub fn write_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    match address {
        _ => cpu.memory[address] = val,
    }
}

fn dma_transfer(val: u8, cpu: &mut Cpu) {
    let source_addr = (val as usize) << 8;
    for (i, addr) in (source_addr..(source_addr | 0xA0)).enumerate() {
        cpu.memory[0xFE00 + i] = cpu.memory[addr];
    }
}

pub fn safe_read_address(address: usize, cpu: &mut Cpu) -> u8 {
    let safe_to_read = match address {
        0x8000...0x9FFF => !LCDC::Power.is_set(cpu) || !ScreenMode::Transferring.is_set(cpu),
        0xFEA0...0xFEFF => false,
        0xFE00...0xFE9F => {
            !LCDC::Power.is_set(cpu) ||
            (ScreenMode::HBlank.is_set(cpu) || ScreenMode::VBlank.is_set(cpu))
        } 
        _ => true,
    };
    if safe_to_read {
        match address {
            0xE000...0xFDFF => read_address(address - 0x1000, cpu),
            0xFF41 => read_stat_address(cpu), 
            0xFF00 => read_joypad(cpu),
            _ => read_address(address, cpu),
        }
    } else {
        0xFF
    }
}

pub fn read_joypad(cpu: &mut Cpu) -> u8 {
    let val = read_address(0xFF00, cpu);
    if ((val >> 4) & 1) != 0 {
        ((val & 0xF0) | 0xF) - ((cpu.key_start as u8) << 3) - ((cpu.key_select as u8) << 2) -
        ((cpu.key_b as u8) << 1) - (cpu.key_a as u8)
    } else if ((val >> 5) & 1) != 0 {
        ((val & 0xF0) | 0xF) - ((cpu.key_down as u8) << 3) - ((cpu.key_up as u8) << 2) -
        ((cpu.key_left as u8) << 1) - (cpu.key_right as u8)
    } else {
        0x0F
    }
}

pub fn read_address(address: usize, cpu: &mut Cpu) -> u8 {
    match address {
        0...0x00FF => read_overlap_address(address, cpu),
        0x0100...0x7FFF => read_cart_address(address, cpu),
        _ => cpu.memory[address],
    }
}

//  This memory can either be the boot rom or the cartridge rom
//  depending on when it is accessed.
//
pub fn read_overlap_address(address: usize, cpu: &mut Cpu) -> u8 {
    if cpu.has_booted {
        read_cart_address(address, cpu)
    } else {
        cpu.boot_rom[address]
    }
}

pub fn read_cart_address(address: usize, cpu: &mut Cpu) -> u8 {
    cpu.cart_rom[address]
}

pub fn stack_push(val: u16, cpu: &mut Cpu) -> () {
    let (l_byte, r_byte) = ((val >> 8) as u8, (0x00FF & val) as u8);
    cpu.sp = cpu.sp.wrapping_sub(2);
    cpu.memory[cpu.sp as usize] = r_byte;
    cpu.memory[(cpu.sp.wrapping_add(1)) as usize] = l_byte;
}

pub fn stack_pop(cpu: &mut Cpu) -> u16 {
    let r_byte = cpu.memory[cpu.sp as usize] as u16;
    let l_byte = cpu.memory[(cpu.sp + 1) as usize] as u16;
    cpu.sp = cpu.sp.wrapping_add(2);
    let res = (l_byte << 8) + r_byte;
    res
}