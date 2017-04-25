extern crate nfd;
use std::fs::File;
use std::io::prelude::*;
use interrupt::*;
use lcd::*;
use apu::*;
use std::path::Path;
use self::nfd::Response;

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
    pub ram_memory: [u8; 0x8000],
    pub boot_rom: Vec<u8>,
    pub cart_rom: Vec<u8>,
    pub cart_loaded: bool,
    pub has_booted: bool,
    pub interrupt_master_enabled: bool,
    pub curr_clocks: u32,
    pub curr_freq_clocks: u32,
    pub mbc_1: bool,
    pub mbc_1_ram: bool,
    pub mbc_1_battery: bool,
    pub ram_banking_mode: bool,
    pub ram_enabled: bool,
    pub ram_or_rom_bank: usize,
    pub lo_rom_bank: usize,
    pub dma_transfer_cycles_left: i32,
    pub background_mode: u8,
    pub window_mode: u8,
    pub sprite_mode: u8,
    pub halted: bool,
    pub apu: Apu,
}

impl Cpu {
    pub fn load_bootrom(&mut self, path: &str) {
        let mut f = File::open(path).unwrap();
        f.read_to_end(&mut self.boot_rom).ok();
    }

    pub fn load_cart(&mut self, default_rom_path: &str) {
        let file_path = if Path::new(default_rom_path).exists() {
            default_rom_path.to_string()
        } else {
            let result = nfd::dialog()
                .default_path("./")
                .filter("gb")
                .open()
                .unwrap();
            match result {
                Response::Okay(path) => path,
                _ => return,
            }
        };

        let mut f = File::open(file_path).unwrap();
        f.read_to_end(&mut self.cart_rom).ok();
        self.cart_loaded = true;
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
            cart_loaded: false,
            has_booted: false,
            interrupt_master_enabled: false,
            memory: [0; 0x10000],
            ram_memory: [0; 0x8000],
            boot_rom: Vec::new(),
            cart_rom: Vec::new(),
            curr_clocks: 0,
            curr_freq_clocks: 0,
            keys: 0xFF,
            mbc_1: false,
            mbc_1_ram: false,
            mbc_1_battery: false,
            ram_or_rom_bank: 0,
            lo_rom_bank: 0,
            dma_transfer_cycles_left: 0,
            ram_enabled: false,
            ram_banking_mode: false,
            halted: false,
            background_mode: 0,
            sprite_mode: 0,
            window_mode: 0,
            apu: Default::default(),
        }
    }
}

pub fn write_nx4_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    // Check if should trigger
    if (val & 0x80) != 0 {
        match address {
            0xFF14 => {
                let nr12 = read_address(0xFF12, cpu);
                cpu.apu.channel_1.enabled = true;
                cpu.apu.channel_1.counter = 64;
                cpu.apu.channel_1.envelope_pos = nr12 & 7;
                cpu.apu.channel_1_pos = 0;
                cpu.apu.channel_1.volume = (nr12 & 0xF0) >> 4;
                cpu.apu.channel_1.enabled = nr12 & 0xF8 != 0;
            }
            0xFF19 => {
                let nr22 = read_address(0xFF17, cpu);
                cpu.apu.channel_2.enabled = true;
                cpu.apu.channel_2.counter = 64;
                cpu.apu.channel_2.envelope_pos = nr22 & 7;
                cpu.apu.channel_2_pos = 0;
                cpu.apu.channel_2.volume = (nr22 & 0xF0) >> 4;
                cpu.apu.channel_2.enabled = nr22 & 0xF8 != 0;
            }
            0xFF1E => {
                cpu.apu.channel_3.enabled = true;
                cpu.apu.channel_3.counter = 256;
                cpu.apu.channel_3_wave_pos = 0;
                let nr32 = read_address(0xFF1C, cpu);
                cpu.apu.channel_3_pos = 2 *
                                        (2048 -
                                         (read_address(0xFF1D, cpu) as u32 |
                                          (read_address(0xFF1E, cpu) as u32) << 8) &
                                         0x7F);
                cpu.apu.channel_3.volume = (nr32 & 0x60) >> 5;
                cpu.apu.channel_3.enabled = (read_address(0xFF1A, cpu) & 0x80) != 0;
            }
            0xFF23 => {
                let nr42 = read_address(0xFF21, cpu);
                let nr43 = read_address(0xFF22, cpu);
                let divisors = [8, 16, 32, 48, 64, 80, 96, 112];
                let dividing_ratio = divisors[(nr43 & 0x7) as usize];
                let shift_clock_freq = (nr43 >> 4) as u32;
                let timer_freq = (dividing_ratio << shift_clock_freq);
                cpu.apu.channel_4.enabled = true;
                cpu.apu.channel_4.counter = 64;
                cpu.apu.channel_4.lfsr = 0x7FFF;
                cpu.apu.channel_4.freq_pos = timer_freq;
                cpu.apu.channel_4.envelope_pos = nr42 & 7;
                cpu.apu.channel_4.volume = nr42 >> 4;
                cpu.apu.channel_4.enabled = nr42 & 0xF8 != 0;
            }
            _ => {}
        };
    }
    write_address(address, val, cpu);
}

pub fn safe_write_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    let safe_to_write = cpu.dma_transfer_cycles_left <= 0 &&
                        match address {
        0x0000...0x1FFF => false, // used for enabling ram bank
        0x2000...0x3FFF => false, //ROM bank number
        0x4000...0x5FFF => false, //RAM bank number or high bits of rom bank number
        0x6000...0x7FFF => false, //ROM/RAM select
        0x8000...0x9FFF => !LCDC::Power.is_set(cpu) || !ScreenMode::Transferring.is_set(cpu),
        0xA000...0xBFFF => cpu.ram_enabled, // currently we have no ram, used for selecting raem
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
            0xA000...0xBFFF => write_ram_address(address, val, cpu),
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
            0xFF10 => write_address(address, val, cpu), //NR 10 Sound Mode 1 Sweep Register
            0xFF11 => {
                cpu.apu.channel_1.counter = 64 - (val & 0x3F) as u16;
                write_address(address, val, cpu);
            } //NR 11 Sound Mode 1 Duty/Sound length
            0xFF12 => {
                let (vol_init, vol_add, vol_period) = (val >> 4, val & 8 != 0, val & 7);
                cpu.apu.channel_1.volume = vol_init;
                cpu.apu.channel_1.incr_vol = vol_add;
                cpu.apu.channel_1.envelope_period = vol_period;
                write_address(address, val, cpu)
            } //NR 12 Sound Mode 1 Envelope
            0xFF13 => write_address(address, val, cpu), //NR 13 Sound Mode 1 Frequency lo
            0xFF14 => write_nx4_address(address, val, cpu), //NR 14 Sound Mode 1 Frequency hi
            0xFF16 => {
                cpu.apu.channel_2.counter = 64 - (val & 0x3F) as u16;
                write_address(address, val, cpu);
            } //NR 21 Sound Mode 2 Duty/Sound length
            0xFF17 => {

                let (vol_init, vol_add, vol_period) = (val >> 4, val & 8 != 0, val & 7);
                cpu.apu.channel_2.volume = vol_init;
                cpu.apu.channel_2.incr_vol = vol_add;
                cpu.apu.channel_2.envelope_period = vol_period;
                write_address(address, val, cpu)
            } //NR 22 Sound Mode 2 Envelope
            0xFF18 => write_address(address, val, cpu), //NR 23 Sound Mode 2 Frequency lo
            0xFF19 => write_nx4_address(address, val, cpu), //NR 24 Sound Mode 2 Frequency hi
            0xFF1A => write_address(address, val, cpu), //NR 30 Sound Mode 3 On/Off
            0xFF1B => {
                cpu.apu.channel_3.counter = 256 - val as u16;
                write_address(address, val, cpu);
            } //NR 31 Sound Mode 3 Sound length
            0xFF1C => write_address(address, val, cpu), //NR 32 Sound Mode 3 Select Output Level
            0xFF1D => write_address(address, val, cpu), //NR 33 Sound Mode 3 Frequency lo
            0xFF1E => write_nx4_address(address, val, cpu), //NR 34 Sound Mode 3 Frequency hi
            0xFF20 => {
                cpu.apu.channel_4.counter = 64 - (val & 0x3F) as u16;
                write_address(address, val, cpu);
            } //NR 41 Sound Mode 4 Sound length
            0xFF21 => {
                let (vol_init, vol_add, vol_period) = (val >> 4, val & 8 != 0, val & 7);
                cpu.apu.channel_4.volume = vol_init;
                cpu.apu.channel_4.incr_vol = vol_add;
                cpu.apu.channel_4.envelope_period = vol_period;
                write_address(address, val, cpu);
            } //NR 42 Sound Mode 4 Envelope
            0xFF22 => write_address(address, val, cpu), //NR 43 Sound Mode 4 Polynomial Counter
            0xFF23 => write_nx4_address(address, val, cpu), //NR 44 Sound Mode 4 Counter
            0xFF24 => write_address(address, val, cpu), //NR 50 Channel Control
            0xFF25 => write_address(address, val, cpu), //NR 51 Sound Output Terminal
            0xFF26 => write_address(address, val, cpu), //NR 52 Sound On/Off
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
                if read_address(0x147, cpu) == 2 {
                    println!("MBC1+RAM Detected");
                    cpu.mbc_1 = true;
                    cpu.mbc_1_ram = true;
                }
                if read_address(0x147, cpu) == 3 {
                    println!("MBC1+RAM+BATTERY Detected");
                    cpu.mbc_1 = true;
                    cpu.mbc_1_ram = true;
                    cpu.mbc_1_battery = true;
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
                0x0000...0x1FFF => cpu.ram_enabled = cpu.mbc_1_ram && ((0xA & val) == 0xA),
                0x2000...0x3FFF => select_rom_bank_lo(val, cpu),
                0x4000...0x5FFF => select_rom_or_ram_bank_hi(val, cpu),
                0x6000...0x7FFF => cpu.ram_banking_mode = val == 1,
                _ => {}
            }
        }
    }
}

fn select_rom_bank_lo(bank: u8, cpu: &mut Cpu) {
    cpu.lo_rom_bank = (bank as usize) & 0x1F;
}

fn select_rom_or_ram_bank_hi(bank: u8, cpu: &mut Cpu) {
    cpu.ram_or_rom_bank = (bank & 0b11) as usize;
}

fn write_joypad(new_val: u8, cpu: &mut Cpu) {
    let val = read_joypad(cpu);
    write_address(0xFF00, (new_val & 0xF0) | val & 0x0F, cpu);
}

pub fn write_ram_address(address: usize, val: u8, cpu: &mut Cpu) -> () {
    let ram_bank = if cpu.ram_banking_mode {
        cpu.ram_or_rom_bank
    } else {
        0
    };
    cpu.ram_memory[address - 0xA000 + ram_bank * 0x2000] = val;
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

pub fn read_channel_1_addresses(cpu: &mut Cpu) -> (u8, u8, u8, u8, u8) {
    (read_address(0xFF10, cpu),
     read_address(0xFF11, cpu),
     read_address(0xFF12, cpu),
     read_address(0xFF13, cpu),
     read_address(0xFF14, cpu))
}

pub fn read_channel_2_addresses(cpu: &mut Cpu) -> (u8, u8, u8, u8) {
    (read_address(0xFF16, cpu),
     read_address(0xFF17, cpu),
     read_address(0xFF18, cpu),
     read_address(0xFF19, cpu))
}

pub fn read_channel_3_addresses(cpu: &mut Cpu) -> (u8, u8, u8, u8, u8) {
    (read_address(0xFF1A, cpu),
     read_address(0xFF1B, cpu),
     read_address(0xFF1C, cpu),
     read_address(0xFF1D, cpu),
     read_address(0xFF1E, cpu))
}

pub fn read_channel_4_addresses(cpu: &mut Cpu) -> (u8, u8, u8, u8) {
    (read_address(0xFF20, cpu),
     read_address(0xFF21, cpu),
     read_address(0xFF22, cpu),
     read_address(0xFF23, cpu))
}

pub fn read_address_i8(address: usize, cpu: &mut Cpu) -> i8 {
    read_address(address, cpu) as i8
}

pub fn read_address(address: usize, cpu: &mut Cpu) -> u8 {
    match address {
        0...0x00FF => read_overlap_address(address, cpu),
        0x0100...0x3FFF => read_cart_address(address, cpu),
        0xA000...0xBFFF => read_ram_address(address, cpu),
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

fn get_selected_rom_bank(cpu: &mut Cpu) -> usize {
    let hi_bank = cond!(!cpu.ram_banking_mode, cpu.ram_or_rom_bank << 5, 0);
    let part_bank = hi_bank | cpu.lo_rom_bank;
    match part_bank {
        0x00 => 0x01,
        0x20 => 0x21,
        0x40 => 0x41,
        0x60 => 0x61,
        _ => part_bank,
    }
}

pub fn read_cart_address(address: usize, cpu: &mut Cpu) -> u8 {
    cpu.cart_rom[address]
}

pub fn read_ram_address(address: usize, cpu: &mut Cpu) -> u8 {
    if cpu.ram_enabled {
        let ram_bank = cond!(cpu.ram_banking_mode, cpu.ram_or_rom_bank, 0);
        cpu.ram_memory[address - 0xA000 + 0x2000 * ram_bank]
    } else {
        0
    }
}

pub fn read_bank_address(address: usize, cpu: &mut Cpu) -> u8 {
    if cpu.mbc_1 {
        let bank = get_selected_rom_bank(cpu);
        cpu.cart_rom[address + 0x4000 * (bank - 1)]
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