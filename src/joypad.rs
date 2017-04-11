extern crate piston_window;
use self::piston_window::*;

pub fn joypad_bit(key: Key) -> (bool, u8) {
    let pos = match key {
        Key::A => 3, // start
        Key::S => 2, // select
        Key::D => 1, // b
        Key::F => 0, // a
        Key::Down => 7, // down
        Key::Up => 6, // down,
        Key::Left => 5,
        Key::Right => 4,
        _ => 8,
    };
    (pos != 8, 1 << (pos % 8))
}