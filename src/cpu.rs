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
    pub keys: u8,
    pub memory: [u8; 0x10000],
    pub boot_rom: Vec<u8>,
    pub cart_rom: Vec<u8>,
    pub has_booted: bool,
    pub interrupt_master_enabled: bool,
    pub curr_clocks: u32,
    pub curr_freq_clocks: u32,
    pub mbc_1: bool,
    pub rom_bank_selected: usize,
    pub dma_transfer_cycles_left: i32,
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
                } else {
                    write_address(0xFF05, tima + 1, self);
                }
            }
        } else {
            self.curr_freq_clocks = 0;
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
            keys: 0xFF,
            mbc_1: false,
            rom_bank_selected: 1,
            dma_transfer_cycles_left: 0,
        }
    }
}

pub fn safe_write_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    let safe_to_write = cpu.dma_transfer_cycles_left <= 0 &&
                        match address {
        0x0000...0x1FFF => false, // used for enabling ram bank
        0x2000...0x3FFF => false, //ROM bank number
        0x4000...0x5FFF => false, //RAM bank number or high bits of rom bank number
        0x6000...0x7FFF => false, //ROM/RAM select
        0x8000...0x9FFF => !LCDC::Power.is_set(cpu) || !ScreenMode::Transferring.is_set(cpu),
        0xA000...0xBFFF => false, // currently we have no ram, used for selecting raem
        0xFE00...0xFE9F => {
            // OAM
            !LCDC::Power.is_set(cpu) ||
            (ScreenMode::HBlank.is_set(cpu) || ScreenMode::VBlank.is_set(cpu))
        } 
        0xFEA0...0xFEFF => false, //Unused memory
        _ => true,
    };
    if safe_to_write {
        match address {
            0xC000...0xDDFF => {
                write_address(address, val, cpu);
                write_address(address + 0x2000, val, cpu);
            }
            0xE000...0xFDFF => {
                write_address(address, val, cpu);
                write_address(address - 0x2000, val, cpu)
            }
            0xFF00 => write_joypad(val, cpu),
            0xFF04 => write_address(address, 0, cpu),
            0xFF40 => write_lcdc_address(val, cpu),
            0xFF41 => write_stat_address(val, cpu),
            0xFF44 => write_address(address, 0, cpu),
            0xFF46 => dma_transfer(val, cpu), //this needs to be synced with clocks
            0xFF50 => {
                cpu.has_booted = true;
                println!("==================BOOTED==================");
                println!("0x0147: {:4>0X} (cartridge type)", read_address(0x147, cpu));
                write_address(0xFF00, 0xCF, cpu);
                if read_address(0x147, cpu) == 1 {
                    println!("MBC1 Detected");
                    cpu.mbc_1 = true;
                }
            }
            _ => write_address(address, val, cpu),
        }
    } else {
        if cpu.dma_transfer_cycles_left > 0 {
            match address {
                0xFF80...0xFFFE => write_address(address, val, cpu),
                _ => {}
            }
        } else {
            match address {
                0x2000...0x3FFF => select_rom_bank_lo(val, cpu),
                0x4000...0x5FFF => select_rom_or_ram_bank_hi(val, cpu),
                _ => {}
            }
        }
    }
}

fn select_rom_bank_lo(bank: u8, cpu: &mut Cpu) {
    let part_bank = (cpu.rom_bank_selected & 0xE0) | ((bank as usize) & 0x1F);
    let new_bank = match part_bank {
        0x00 => 0x01,
        0x20 => 0x21,
        0x40 => 0x41,
        0x60 => 0x61,
        _ => part_bank,
    };
    cpu.rom_bank_selected = new_bank;
}

fn select_rom_or_ram_bank_hi(bank: u8, cpu: &mut Cpu) {
    let part_bank = (cpu.rom_bank_selected & 0x1F) | (((bank as usize) & 0b11) << 5);
    let new_bank = match part_bank {
        0x00 => 0x01,
        0x20 => 0x21,
        0x40 => 0x41,
        0x60 => 0x61,
        _ => part_bank,
    };
    cpu.rom_bank_selected = new_bank;

}

fn write_joypad(new_val: u8, cpu: &mut Cpu) {
    let val = read_joypad(cpu);
    write_address(0xFF00, (new_val & 0xF0) | val & 0x0F, cpu);
}

pub fn write_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    match address {
        0x2000...0x3FFF => select_rom_bank_lo(val, cpu),
        0x4000...0x5FFF => select_rom_or_ram_bank_hi(val, cpu),
        _ => cpu.memory[address] = val,
    }
}

fn dma_transfer(val: u8, cpu: &mut Cpu) {
    let source_addr = (val as usize) << 8;
    cpu.dma_transfer_cycles_left = 162 * 4;
    for (i, addr) in (source_addr..(source_addr | 0xA0)).enumerate() {
        write_address(0xFE00 + i, read_address(addr, cpu), cpu);
    }
}

pub fn safe_read_address(address: usize, cpu: &mut Cpu) -> u8 {
    let safe_to_read = cpu.dma_transfer_cycles_left <= 0 &&
                       match address {
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
            0xE000...0xFDFF => read_address(address - 0x2000, cpu),
            0xFF41 => read_stat_address(cpu), 
            0xFF00 => read_joypad(cpu),
            _ => read_address(address, cpu),
        }
    } else {
        match address {
            0xFF80...0xFFFE => read_address(address, cpu),
            _ => 0xFF,
        }
    }
}

pub fn read_joypad(cpu: &mut Cpu) -> u8 {
    let val = read_address(0xFF00, cpu);
    if ((val >> 4) & 1) != 0 {
        (val & 0xF0) | (0xF & cpu.keys)
    } else if ((val >> 5) & 1) != 0 {
        (val & 0xF0) | ((0xF0 & cpu.keys) >> 4)
    } else {
        0x0F
    }
}

pub fn read_address(address: usize, cpu: &mut Cpu) -> u8 {
    match address {
        0...0x00FF => read_overlap_address(address, cpu),
        0x0100...0x3FFF => read_cart_address(address, cpu),
        0x4000...0x7FFF => read_bank_address(address, cpu),
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

pub fn read_bank_address(address: usize, cpu: &mut Cpu) -> u8 {
    if cpu.mbc_1 {
        cpu.cart_rom[address + 0x4000 * (cpu.rom_bank_selected - 1)]
    } else {
        cpu.cart_rom[address]
    }
}

pub fn stack_push(val: u16, cpu: &mut Cpu) -> () {
    let (l_byte, r_byte) = ((val >> 8) as u8, (0x00FF & val) as u8);
    cpu.sp = cpu.sp.wrapping_sub(2);
    safe_write_address(cpu.sp as usize, r_byte, cpu);
    safe_write_address(cpu.sp.wrapping_add(1) as usize, l_byte, cpu);
}

pub fn stack_pop(cpu: &mut Cpu) -> u16 {
    let r_byte = safe_read_address(cpu.sp as usize, cpu) as u16;
    let l_byte = safe_read_address(cpu.sp.wrapping_add(1) as usize, cpu) as u16;
    cpu.sp = cpu.sp.wrapping_add(2);
    let res = (l_byte << 8) + r_byte;
    res
}