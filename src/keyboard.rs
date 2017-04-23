extern crate piston_window;
use cpu;
use joypad;
use self::piston_window::Key;

pub fn handle_key_release(key: Key, cpu: &mut cpu::Cpu) {
    let (handle_key, bit_mask) = joypad::joypad_bit(key);
    if handle_key {
        cpu.keys |= bit_mask;
    }
}
pub fn handle_keypress(key: Key, cpu: &mut cpu::Cpu) {
    if !cpu.cart_loaded {
        match key {
            Key::O => cpu.load_cart("test.gb"),
            _ => {}
        };
    }
    match key {
        Key::D1 => {
            cpu.background_mode = (cpu.background_mode + 1) % 3;
        }
        Key::D2 => {
            cpu.window_mode = (cpu.window_mode + 1) % 3;
        }
        Key::D3 => {
            cpu.sprite_mode = (cpu.sprite_mode + 1) % 3;
        }
        Key::D4 => {}
        _ => {}
    };
    let (handle_key, bit_mask) = joypad::joypad_bit(key);
    if handle_key {
        cpu.keys &= !bit_mask;
    }
}