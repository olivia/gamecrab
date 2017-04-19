extern crate image;
use cpu::*;
use lcd::*;

pub fn render_scanline(ly: u8,
                       screen_buffer: &mut [u8; 256 * 256],
                       canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                       cpu: &mut Cpu) {
    if ly > 143 {
        return;
    }
    if LCDC::Power.is_set(cpu) {
        write_background_line(ly, screen_buffer, cpu);
        write_window_line(ly, screen_buffer, cpu);
        write_sprites_line(ly, screen_buffer, cpu);
        buffer_line_to_image_buffer(ly, canvas, screen_buffer)
    } else {
        clear_buffer(screen_buffer);
        buffer_line_to_image_buffer(ly, canvas, screen_buffer)
    }
}

fn clear_buffer(screen_buffer: &mut [u8; 256 * 256]) {
    for i in 0..(256 * 256) {
        screen_buffer[i] = 0;
    }
}

pub fn write_background_line(ly: u8, buffer: &mut [u8; 256 * 256], cpu: &mut Cpu) {
    if (cpu.background_mode == 0 && LCDC::BGEnable.is_set(cpu)) || cpu.background_mode == 1 {
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
        let scroll_x = read_address(0xFF43, cpu) as usize;
        let scroll_y = read_address(0xFF42, cpu) as usize;
        let start_offset = 32 * (((ly as usize + scroll_y) / 8) % 32);
        let tile_y = (256 + 8 * (start_offset / 32) - scroll_y) % 256;

        for offset in start_offset..(start_offset + 32) {
            let tile_x = (256 + 8 * (offset % 32) - scroll_x) % 256;
            // Skip painting the tiles that are not visible
            if tile_x <= 160 || tile_x >= 248 {
                let tile_num = if LCDC::Tileset.is_set(cpu) {
                    read_address(start + offset, cpu) as usize
                } else {
                    (128 as i16 + (read_address(start + offset, cpu) as i8) as i16) as usize
                };
                write_bg_tile_line(ly,
                                   tile_num,
                                   tile_x as usize,
                                   tile_y as usize,
                                   tile_map_start,
                                   0xFF47,
                                   buffer,
                                   cpu);
            }
        }
    } else {
    }
}

pub fn write_window_line(ly: u8, buffer: &mut [u8; 256 * 256], cpu: &mut Cpu) {
    if (cpu.window_mode == 0 && LCDC::WindowEnable.is_set(cpu)) || cpu.window_mode == 1 {
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
        if ly >= scroll_y {
            let start_offset = (32 * ((ly - scroll_y) / 8)) as usize;
            for offset in start_offset..(32 + start_offset) {
                let tile_num = if LCDC::Tileset.is_set(cpu) {
                    read_address(start + offset, cpu) as usize
                } else {
                    (128 as i16 + (read_address(start + offset, cpu) as i8) as i16) as usize
                };
                write_window_tile_line(ly,
                                       tile_num,
                                       8 * (offset as isize % 32) + scroll_x as isize - 7,
                                       8 * (offset as isize / 32) + scroll_y as isize,
                                       tile_map_start,
                                       buffer,
                                       cpu);
            }
        }
    } else {
    }
}

pub fn write_sprites_line(ly: u8, buffer: &mut [u8; 256 * 256], cpu: &mut Cpu) {
    if (cpu.sprite_mode == 0 && LCDC::SpritesEnable.is_set(cpu)) || cpu.sprite_mode == 1 {
        let start = 0xFE00;
        let square_sprites = !LCDC::SpriteSize.is_set(cpu);
        for i in 0..40 {
            let address = start + i * 4;
            let y = read_address(address, cpu) as isize;
            if y == 0 {
                continue;
            } // sprite is off screen

            let (x, tile_num, sprite_flag) = (read_address(address + 1, cpu) as isize,
                                              read_address(address + 2, cpu) as usize,
                                              read_address(address + 3, cpu));
            let pallette_address = if (sprite_flag & 0b00010000) == 0 {
                0xFF48
            } else {
                0xFF49
            };
            let h_flip = (sprite_flag & 0b00100000) != 0;
            let v_flip = (sprite_flag & 0b01000000) != 0;

            if square_sprites && ((ly as isize) >= y - 8 || (ly as isize) < y - 16) {
                continue;
            } else if !square_sprites && ((ly as isize) >= y || ((ly as isize) < y - 16)) {
                continue;
            }

            if square_sprites {
                write_sprite_tile_line(ly,
                                       tile_num,
                                       x - 8,
                                       y - 16,
                                       pallette_address,
                                       h_flip,
                                       v_flip,
                                       buffer,
                                       cpu);
            } else {
                let (tile_hi, tile_lo) = if v_flip {
                    (tile_num | 0x01, tile_num & 0xFE)
                } else {
                    (tile_num & 0xFE, tile_num | 0x01)
                };

                if (ly as isize) < (y - 8) {
                    write_sprite_tile_line(ly,
                                           tile_hi,
                                           x - 8,
                                           y - 16,
                                           pallette_address,
                                           h_flip,
                                           v_flip,
                                           buffer,
                                           cpu);
                } else {
                    write_sprite_tile_line(ly,
                                           tile_lo,
                                           x - 8,
                                           y - 8,
                                           pallette_address,
                                           h_flip,
                                           v_flip,
                                           buffer,
                                           cpu);
                }
            }
        }
    }
}

pub fn write_bg_tile_line(ly: u8,
                          tile_num: usize,
                          x: usize,
                          y: usize,
                          tile_map_start: usize,
                          pallette_address: usize,
                          buffer: &mut [u8; 256 * 256],
                          cpu: &mut Cpu) {
    let address_start = tile_map_start + 16 * tile_num;
    let row = (256 + ly as usize - y) % 256;
    let left_line = read_address(address_start + row * 2, cpu) as u16;
    let right_line = read_address(address_start + row * 2 + 1, cpu) as u16;
    let y_idx = (y + row) % 256;
    let buffer_start = 256 * y_idx as usize;

    for col in 0..8 {
        let pos_offset = 7 - col;
        let color_map_idx = ((right_line >> pos_offset & 1) << 1) as u8 |
                            (left_line >> pos_offset & 1) as u8;
        let color_idx = lookup_color_idx(pallette_address, color_map_idx, cpu);
        let x_idx = (x + col) % 256;
        let buffer_idx = buffer_start + x_idx;
        buffer[buffer_idx] = color_idx;
    }
}

pub fn write_sprite_tile_line(ly: u8,
                              tile_num: usize,
                              x: isize,
                              y: isize,
                              pallette_address: usize,
                              h_flip: bool,
                              v_flip: bool,
                              buffer: &mut [u8; 256 * 256],
                              cpu: &mut Cpu) {
    use std::cmp;
    let tile_map_start = 0x8000;
    let start_col = cmp::min(8, cmp::max(0, -x)) as usize;
    let end_col = cmp::max(0, cmp::min(8, 255 - x)) as usize;

    let address_start = tile_map_start + 16 * tile_num;
    let row = (ly as isize - y) as usize;
    let normalized_row = if v_flip { 7 - row } else { row };
    let left_line = read_address(address_start + normalized_row * 2, cpu) as u16;
    let right_line = read_address(address_start + normalized_row * 2 + 1, cpu) as u16;

    for col in start_col..end_col {
        let shift = if h_flip { col } else { 7 - col };
        let o_palette_idx = ((right_line >> shift & 1) << 1) as u8 + (left_line >> shift & 1) as u8;
        let color_idx = lookup_color_idx(pallette_address, o_palette_idx, cpu);
        if o_palette_idx != 0 {
            buffer[(ly as usize) * 256 + (x + col as isize) as usize] = color_idx;
        }
    }
}

pub fn write_window_tile_line(ly: u8,
                              tile_num: usize,
                              x: isize,
                              y: isize,
                              tile_map_start: usize,
                              buffer: &mut [u8; 256 * 256],
                              cpu: &mut Cpu) {
    use std::cmp;
    let start_col = cmp::min(8, cmp::max(0, -x)) as usize;
    let end_col = cmp::max(0, cmp::min(8, 255 - x)) as usize;
    let address_start = tile_map_start + 16 * tile_num;

    if y > 255 || x > 255 {
        return;
    }
    let row = ly as usize - y as usize;

    let left_line = read_address(address_start + row * 2, cpu) as u16;
    let right_line = read_address(address_start + row * 2 + 1, cpu) as u16;

    for col in start_col..end_col {
        let color_idx = lookup_color_idx(0xFF47,
                                         ((right_line >> (7 - col) & 1) << 1) as u8 +
                                         (left_line >> (7 - col) & 1) as u8,
                                         cpu);
        buffer[(ly as usize) * 256 + (x + col as isize) as usize] = color_idx;
    }
}

pub fn lookup_color_idx(address: usize, pallete_idx: u8, cpu: &mut Cpu) -> u8 {
    (read_address(address, cpu) >> (pallete_idx * 2)) & 0b11
}

pub fn buffer_to_image_buffer(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                              buffer: &mut [u8; 256 * 256]) {
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let idx = x + 256 * y;
        *pixel = get_color(buffer[idx as usize]);
    }
}

pub fn buffer_line_to_image_buffer(line_num: u8,
                                   canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                                   buffer: &mut [u8; 256 * 256]) {
    let (width, _) = canvas.dimensions();
    for x in 0..width {
        let idx = x as usize + 256 * (line_num as usize);
        canvas.put_pixel(x, line_num as u32, get_color(buffer[idx as usize]));
    }
}

pub fn get_color(idx: u8) -> image::Rgba<u8> {
    image::Rgba(match idx {
        0 => [0x7F, 0x85, 0x51, 255],
        1 => [0x58, 0x7B, 0x48, 255],
        2 => [0x38, 0x5D, 0x49, 255],
        3 => [0x2B, 0x45, 0x3C, 255],
        4 => [0xFF, 0x0, 0x0, 255],
        _ => unreachable!(),
    })
}