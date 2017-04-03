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

pub fn increment_ly(cpu: &mut Cpu) {
    let val = (read_address(0xFF44, cpu) + 1) % 154;
    write_address(0xFF44, (read_address(0xFF44, cpu) + 1) % 154, cpu);
    if val == 144 {
        Interrupt::VBlank.request(cpu);
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
    let read_only_val = read_address(0xFF41, cpu) & 0b111;
    write_address(0xFF41, (val & (0xFF - 0b111)) | read_only_val, cpu)
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
