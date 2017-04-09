use cpu::*;
use interrupt::Interrupt;

pub enum LCDC {
    Power,
    WindowTileMap,
    WindowEnable,
    Tileset,
    BGTileMap,
    SpriteSize,
    SpritesEnable,
    BGEnable,
}

use self::LCDC::*;

impl LCDC {
    pub fn is_set(&self, cpu: &mut Cpu) -> bool {
        read_address(0xFF40, cpu) & self.bit_mask() != 0
    }

    fn bit_mask(&self) -> u8 {
        let shift = match *self {
            Power => 7,
            WindowTileMap => 6,
            WindowEnable => 5,
            Tileset => 4,
            BGTileMap => 3,
            SpriteSize => 2,
            SpritesEnable => 1,
            BGEnable => 0,
        };
        1 << shift
    }
}

pub fn update_status(frames: usize, cpu: &mut Cpu) {
    if LCDC::Power.is_set(cpu) {
        let ly = read_address(0xFF44, cpu);
        if ly >= 144 {
            ScreenMode::VBlank.set(cpu);
        } else {
            let (interrupt_enabled, new_mode) = match frames % 456 {
                0...202 => (stat_is_set(STAT::Mode0HBlankCheck, cpu), ScreenMode::HBlank),
                203...283 => (stat_is_set(STAT::Mode2OAMCheck, cpu), ScreenMode::Searching),
                284...455 => (false, ScreenMode::Transferring),
                _ => unreachable!(),
            };

            if !new_mode.is_set(cpu) {
                if interrupt_enabled && cpu.interrupt_master_enabled {
                    Interrupt::LCD.request(cpu);
                }
                new_mode.set(cpu);
            }
        }
    }

}

pub fn increment_ly(cpu: &mut Cpu) {
    let val = (read_address(0xFF44, cpu) + 1) % 154;
    write_address(0xFF44, (read_address(0xFF44, cpu) + 1) % 154, cpu);
    if cpu.interrupt_master_enabled {
        if val == 144 {
            Interrupt::VBlank.request(cpu);
        }
        if stat_is_set(STAT::Mode1VBlankCheck, cpu) && val == 144 {
            Interrupt::LCD.request(cpu);
        }
        if stat_is_set(STAT::LYLYCCheck, cpu) && val == read_address(0xFF45, cpu) {
            Interrupt::LCD.request(cpu);
        }
    }

    // set lcy=ly comparison flag
    if val == read_address(0xFF45, cpu) {
        let stat = read_address(0xFF41, cpu);
        write_address(0xFF41, stat | 0b100, cpu);
    } else {
        let stat = read_address(0xFF41, cpu);
        write_address(0xFF41, stat & (0xFF - 0b100), cpu);
    }
}

pub enum STAT {
    LYLYCCheck,
    Mode2OAMCheck,
    Mode1VBlankCheck,
    Mode0HBlankCheck,
    LYLYCSignal,
    SM(ScreenMode),
}

#[derive(Clone, Copy)]
pub enum ScreenMode {
    HBlank,
    VBlank,
    Searching,
    Transferring,
}

use self::ScreenMode::*;

impl ScreenMode {
    pub fn is_set(&self, cpu: &mut Cpu) -> bool {
        let val = read_stat_address(cpu) & self.stat_mask();
        val == self.val()
    }

    pub fn set(&self, cpu: &mut Cpu) {
        let val = read_stat_address(cpu) & (0xFF - self.stat_mask());
        write_address(0xFF41, val | self.val(), cpu);
    }

    pub fn val(&self) -> u8 {
        match *self {
            HBlank => 0,
            VBlank => 1,
            Searching => 2,
            Transferring => 3,        
        }
    }

    pub fn stat_mask(&self) -> u8 {
        0b11
    }
}

pub fn read_stat_address(cpu: &mut Cpu) -> u8 {
    let val = (1 << 7) | read_address(0xFF41, cpu);
    if LCDC::Power.is_set(cpu) {
        val
    } else {
        val & (0xFF - 0b111)
    }
}

pub fn write_stat_address(val: u8, cpu: &mut Cpu) {
    let prev_val = read_address(0xFF41, cpu);
    let write_val = (val & (0xFF - 0b111)) | (prev_val & 0b111);
    write_address(0xFF41, write_val, cpu)
}

pub fn stat_is_set(stat: STAT, cpu: &mut Cpu) -> bool {
    read_stat_address(cpu) & stat_bit(stat) != 0
}

pub fn screen_mode_is_set(screen_mode: ScreenMode, cpu: &mut Cpu) -> bool {
    let val = read_stat_address(cpu) & stat_bit(STAT::SM(screen_mode));
    val == screen_mode_val(screen_mode)
}

pub fn screen_mode_set(screen_mode: ScreenMode, cpu: &mut Cpu) {
    let val = read_stat_address(cpu) & (0xFF - stat_bit(STAT::SM(screen_mode)));
    write_address(0xFF41, val | screen_mode_val(screen_mode), cpu);
}

pub fn screen_mode_val(screen_mode: ScreenMode) -> u8 {
    use self::ScreenMode::*;
    match screen_mode {
        HBlank => 0,
        VBlank => 1,
        Searching => 2,
        Transferring => 3,        
    }
}

pub fn stat_bit(stat: STAT) -> u8 {
    use self::STAT::*;
    match stat {
        SM(_) => 0b11,
        _ => {
            (1 <<
             match stat {
                LYLYCCheck => 6,
                Mode2OAMCheck => 5,
                Mode1VBlankCheck => 4,
                Mode0HBlankCheck => 3,
                LYLYCSignal => 2,
                _ => unreachable!(),
            })
        } 
    }
}
