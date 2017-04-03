extern crate image;
use cpu::*;
use lcd::*;

pub fn write_background(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut Cpu) {
    if LCDC::BGEnable.is_set(cpu) {
        let start = if LCDC::BGTileMap.is_set(cpu) {
            0x9C00
        } else {
            0x9800
        };
        let tile_map_start = if LCDC::Tileset.is_set(cpu) {
            0x8000
        } else {
            0x8800
        };
        let scroll_x = read_address(0xFF43, cpu);
        let scroll_y = read_address(0xFF42, cpu);
        for offset in 0..(32 * 32) {
            let tile_num = if LCDC::Tileset.is_set(cpu) {
                read_address(start + offset, cpu) as usize
            } else {
                (128 as i16 + (read_address(start + offset, cpu) as i8) as i16) as usize
            };
            write_bg_tile(tile_num,
                          (256 + 8 * (offset % 32) as isize - scroll_x as isize) % 256,
                          (256 + 8 * (offset / 32) as isize - scroll_y as isize) % 256,
                          tile_map_start,
                          0xFF47,
                          buffer,
                          cpu);
        }
    } else {
    }
}

pub fn write_window(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut Cpu) {
    if LCDC::WindowEnable.is_set(cpu) {
        let start = if LCDC::WindowTileMap.is_set(cpu) {
            0x9C00
        } else {
            0x9800
        };
        let tile_map_start = if LCDC::Tileset.is_set(cpu) {
            0x8000
        } else {
            0x8800
        };

        let scroll_x = read_address(0xFF4B, cpu);
        let scroll_y = read_address(0xFF4A, cpu);
        for offset in 0..(32 * 32) {
            let tile_num = if LCDC::Tileset.is_set(cpu) {
                read_address(start + offset, cpu) as usize
            } else {
                (128 as i16 + (read_address(start + offset, cpu) as i8) as i16) as usize
            };
            write_tile(tile_num,
                       8 * (offset as isize % 32) - scroll_x as isize - 7,
                       8 * (offset as isize / 32) - scroll_y as isize,
                       tile_map_start,
                       buffer,
                       cpu);
        }
    }
}

pub fn write_sprites(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut Cpu) {
    if LCDC::SpritesEnable.is_set(cpu) {
        let start = 0xFE00;
        let mut sprites_drawn = 0;
        let square_sprites = !LCDC::SpriteSize.is_set(cpu);
        for i in 0..40 {
            let address = start + i * 4;
            let y = read_address(address, cpu) as isize;
            if y == 0 {
                continue;
            } // sprite is off screen
            sprites_drawn += 1;
            if sprites_drawn > 40 {
                break;
            }
            let (x, tile_num, sprite_flag) = (read_address(address + 1, cpu) as isize,
                                              read_address(address + 2, cpu) as usize,
                                              read_address(address + 3, cpu));
            let pallette_address = if (sprite_flag & 0b00010000) == 0 {
                0xFF48
            } else {
                0xFF49
            };

            if square_sprites {
                write_sprite_tile(tile_num, x - 8, y - 16, pallette_address, buffer, cpu);
            } else {
                write_sprite_tile(tile_num, x - 8, y - 16, pallette_address, buffer, cpu);
                write_sprite_tile(tile_num, x - 8, y - 8, pallette_address, buffer, cpu);
            }
        }
    }
}

pub fn write_bg_tile(tile_num: usize,
                     x: isize,
                     y: isize,
                     tile_map_start: usize,
                     pallette_address: usize,
                     buffer: &mut [image::Rgba<u8>; 256 * 256],
                     cpu: &mut Cpu) {
    for row in 0..8 {
        let left_line = read_address(tile_map_start + 16 * tile_num + row * 2, cpu) as u16;
        let right_line = read_address(tile_map_start + 16 * tile_num + 1 + row * 2, cpu) as u16;

        for col in 0..8 {
            let color_idx = lookup_color_idx(pallette_address,
                                             ((right_line >> (7 - col) & 1) << 1) as u8 +
                                             (left_line >> (7 - col) & 1) as u8,
                                             cpu);
            let x_idx = (x + col as isize) % 256;
            let y_idx = (y + row as isize) % 256;
            let buffer_idx = 256 * y_idx as usize + x_idx as usize;
            buffer[buffer_idx] = get_color(color_idx);
        }
    }
}

pub fn write_sprite_tile(tile_num: usize,
                         x: isize,
                         y: isize,
                         pallette_address: usize,
                         buffer: &mut [image::Rgba<u8>; 256 * 256],
                         cpu: &mut Cpu) {
    use std::cmp;
    let tile_map_start = 0x8000;
    let start_col = cmp::min(8, cmp::max(0, -x)) as usize;
    let end_col = cmp::max(0, cmp::min(8, 255 - x)) as usize;
    let start_row = cmp::min(8, cmp::max(0, -y)) as usize;
    let end_row = cmp::max(0, cmp::min(8, 255 - y)) as usize;

    for row in start_row..end_row {
        let left_line = read_address(tile_map_start + 16 * tile_num + row * 2, cpu) as u16;
        let right_line = read_address(tile_map_start + 16 * tile_num + 1 + row * 2, cpu) as u16;

        for col in start_col..end_col {
            let color_idx = lookup_color_idx(pallette_address,
                                             ((right_line >> (7 - col) & 1) << 1) as u8 +
                                             (left_line >> (7 - col) & 1) as u8,
                                             cpu);
            if color_idx != 0 {
                buffer[((y + row as isize) as usize) * 256 + (x + col as isize) as usize] =
                    get_color(color_idx);
            }
        }
    }
}

pub fn write_tile(tile_num: usize,
                  x: isize,
                  y: isize,
                  tile_map_start: usize,
                  buffer: &mut [image::Rgba<u8>; 256 * 256],
                  cpu: &mut Cpu) {
    use std::cmp;
    let start_col = cmp::min(8, cmp::max(0, -x)) as usize;
    let end_col = cmp::max(0, cmp::min(8, 255 - x)) as usize;
    let start_row = cmp::min(8, cmp::max(0, -y)) as usize;
    let end_row = cmp::max(0, cmp::min(8, 255 - y)) as usize;

    for row in start_row..end_row {
        let left_line = read_address(tile_map_start + 16 * tile_num + row * 2, cpu) as u16;
        let right_line = read_address(tile_map_start + 16 * tile_num + 1 + row * 2, cpu) as u16;

        for col in start_col..end_col {
            let color_idx = lookup_color_idx(0xFF47,
                                             ((right_line >> (7 - col) & 1) << 1) as u8 +
                                             (left_line >> (7 - col) & 1) as u8,
                                             cpu);
            buffer[((y + row as isize) as usize) * 256 + (x + col as isize) as usize] =
                get_color(color_idx);
        }
    }
}

pub fn lookup_color_idx(address: usize, pallete_idx: u8, cpu: &mut Cpu) -> u8 {
    (read_address(address, cpu) >> (pallete_idx * 2)) & 0b11
}

pub fn buffer_to_image_buffer(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                              buffer: [image::Rgba<u8>; 256 * 256]) {
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let idx = x + 256 * y;
        *pixel = buffer[idx as usize];
    }
}

pub fn get_color(idx: u8) -> image::Rgba<u8> {
    image::Rgba(match idx {
        0 => [0x7F, 0x85, 0x51, 255],
        1 => [0x58, 0x7B, 0x48, 255],
        2 => [0x38, 0x5D, 0x49, 255],
        3 => [0x2B, 0x45, 0x3C, 255],
        _ => unreachable!(),
    })
}