use cpu::*;
use self::Interrupt::*;

// Ordered in descending priority
#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank,
    LCD,
    Timer,
    Joypad,
    Serial,
}

pub fn exec_halt_interrupts(address: usize, cpu: &mut Cpu) -> usize {
    let interrupts = [VBlank, LCD, Timer, Serial, Joypad];
    interrupts.iter()
        .find(|&interrupt| interrupt.is_requested(cpu) && interrupt.is_enabled(cpu))
        .map_or(address, |interrupt| if cpu.interrupt_master_enabled {
            cpu.halted = false;
            interrupt.exec(address + 1, cpu)
        } else {
            cpu.halted = false;
            address + 1
        })
}

pub fn exec_interrupts(address: usize, cpu: &mut Cpu) -> usize {
    if cpu.interrupt_master_enabled {
        let interrupts = [VBlank, LCD, Timer, Serial, Joypad];
        interrupts.iter()
            .find(|&interrupt| interrupt.is_requested(cpu) && interrupt.is_enabled(cpu))
            .map_or(address, |interrupt| {
                println!("Exec: {:?}", interrupt);
                interrupt.exec(address, cpu)
            })
    } else {
        address
    }
}


impl Interrupt {
    pub fn request(&self, cpu: &mut Cpu) {
        let requests = get_requests(cpu);
        let mask = 1 << self.bit_pos();
        write_address(0xFF0F, requests | mask, cpu);
    }

    fn exec(&self, address: usize, cpu: &mut Cpu) -> usize {
        cpu.interrupt_master_enabled = false;
        self.reset_request(cpu);
        stack_push(address as u16, cpu);
        self.interrupt_address()
    }

    fn reset_request(&self, cpu: &mut Cpu) {
        let requests = get_requests(cpu);
        let mask = 0xFF - (1 << self.bit_pos());
        write_address(0xFF0F, requests & mask, cpu);
    }

    fn is_requested(&self, cpu: &mut Cpu) -> bool {
        let val = read_address(0xFF0F, cpu) & (1 << self.bit_pos());
        val != 0
    }

    fn is_enabled(&self, cpu: &mut Cpu) -> bool {
        let val = read_address(0xFFFF, cpu) & (1 << self.bit_pos());
        val != 0
    }

    fn bit_pos(&self) -> u8 {
        match *self {
            VBlank => 0,
            LCD => 1,
            Timer => 2,
            Serial => 3,
            Joypad => 4,
        }
    }

    fn interrupt_address(&self) -> usize {
        match *self {
            VBlank => 0x40,
            LCD => 0x48,
            Timer => 0x50,
            Serial => 0x58,
            Joypad => 0x60,
        }
    }
}

fn get_requests(cpu: &mut Cpu) -> u8 {
    read_address(0xFF0F, cpu)
}