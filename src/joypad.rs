extern crate piston_window;
use self::piston_window::*;

pub fn joypad_bit(key: Key) -> (bool, u8) {
    let pos = match key {
        Key::A => 3, // start
        Key::S => 2, // select
        Key::D => 1, // b
        Key::F => 0, // a
        Key::J => 7, // down
        Key::K => 6, // up,
        Key::H => 5, //left
        Key::L => 4, //right
        _ => 8,
    };
    (pos != 8, 1 << (pos % 8))
}
