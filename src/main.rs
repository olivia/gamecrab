extern crate gamecrab;
extern crate image;
extern crate piston_window;
extern crate fps_counter;
extern crate time;
use std::cmp;
use piston_window::*;
use piston_window::texture::Filter;
use fps_counter::*;
use gamecrab::{cpu, opcode, instr, interrupt, lcd};
use lcd::*;

fn get_gameboy_canvas(scale: u32) -> (u32, u32, image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let (width, height) = (160, 144);
    let mut canvas = image::ImageBuffer::new(width, height);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = get_color((x * x / 2 + y * y / 2) as u8 % 4);
    }
    (width * scale, height * scale, canvas)
}

fn render_frame(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, cpu: &mut cpu::Cpu) {
    let mut screen_buffer = [get_color(0); 256 * 256];
    if lcd::LCDC::Power.is_set(cpu) {
        write_background(&mut screen_buffer, cpu);
        write_window(&mut screen_buffer, cpu);
        write_sprites(&mut screen_buffer, cpu);
        buffer_to_image_buffer(canvas, screen_buffer)
    }
}

fn write_background(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    use cpu::*;
    if lcd::LCDC::BGEnable.is_set(cpu) {
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
            write_tile(tile_num,
                       (256 + 8 * (offset % 32) as isize - scroll_x as isize) % 256,
                       (256 + 8 * (offset / 32) as isize - scroll_y as isize) % 256,
                       tile_map_start,
                       buffer,
                       cpu);
        }
    }
}

fn write_window(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    use cpu::*;
    if lcd::LCDC::WindowEnable.is_set(cpu) {
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

fn write_sprites(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    use cpu::*;
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

fn write_sprite_tile(tile_num: usize,
                     x: isize,
                     y: isize,
                     pallette_address: usize,
                     buffer: &mut [image::Rgba<u8>; 256 * 256],
                     cpu: &mut cpu::Cpu) {
    use cpu::*;
    use cmp;
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

fn write_tile(tile_num: usize,
              x: isize,
              y: isize,
              tile_map_start: usize,
              buffer: &mut [image::Rgba<u8>; 256 * 256],
              cpu: &mut cpu::Cpu) {
    use cpu::*;
    use cmp;
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

fn lookup_color_idx(address: usize, pallete_idx: u8, cpu: &mut cpu::Cpu) -> u8 {
    use cpu::*;
    (read_address(address, cpu) >> (pallete_idx * 2)) & 0b11
}

fn buffer_to_image_buffer(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                          buffer: [image::Rgba<u8>; 256 * 256]) {
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let idx = x + 256 * y;
        *pixel = buffer[idx as usize];
    }
}

fn get_color(idx: u8) -> image::Rgba<u8> {
    image::Rgba(match idx {
        0 => [0x7F, 0x85, 0x51, 255],
        1 => [0x58, 0x7B, 0x48, 255],
        2 => [0x38, 0x5D, 0x49, 255],
        3 => [0x2B, 0x45, 0x3C, 255],
        _ => unreachable!(),
    })
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut cpu: cpu::Cpu = Default::default();
    let mut counter = FPSCounter::new();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("kirby.gb");
    let mut next_addr = 0;
    let mut cycle_count = 0;
    let scale = 3;
    let (width, height, canvas) = get_gameboy_canvas(scale);
    let mut window: PistonWindow = WindowSettings::new("🎮🦀", [width, height])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();
    window.set_ups(10000);

    let factory = window.factory.clone();
    let font = "FiraSans-Regular.ttf";
    let mut glyphs = Glyphs::new(font, factory).unwrap();

    let mut texture_settings = TextureSettings::new();
    texture_settings.set_filter(Filter::Nearest);
    let mut texture = Texture::from_image(&mut window.factory, &canvas, &texture_settings).unwrap();
    let mut frame = canvas;
    let mut mod_cycles = 0;
    let mut debug_address = false;

    while let Some(e) = window.next() {
        let t1 = time::precise_time_ns();
        if next_addr != 0x100 {
            let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);
            if false {
                println!("Address {:4>0X}: {:?} taking {:?} cycles",
                         next_addr,
                         instr,
                         cycles);
            }

            next_addr += op_length;
            let t2 = time::precise_time_ns();
            let (cycle_offset, new_addr) = instr::exec_instr(instr, next_addr, &mut cpu);

            next_addr = new_addr;
            cycle_count += cycles + cycle_offset;
            mod_cycles += cycles + cycle_offset;
            if mod_cycles > (456 * 4) {
                if lcd::LCDC::Power.is_set(&mut cpu) {
                    // println!("LY: {:?}", cpu::read_address(0xFF44, &mut cpu));
                    if cpu::read_address(0xFF44, &mut cpu) == 144 {
                        debug_address = true;
                    } else {
                        lcd::increment_ly(&mut cpu);
                        debug_address = false;
                    };
                }
                mod_cycles = mod_cycles % (456 * 4);
            }
            let t3 = time::precise_time_ns();
            next_addr = interrupt::exec_interrupts(next_addr, &mut cpu);
            let t4 = time::precise_time_ns();
        }

        if let Some(_) = e.render_args() {
            let t4 = time::precise_time_ns();
            render_frame(&mut frame, &mut cpu);
            texture.update(&mut window.encoder, &frame)
                .unwrap();
            window.draw_2d(&e, |c, g| {
                let transform = c.transform.trans(10.0, 30.0);

                clear([1.0; 4], g);
                image(&texture, c.transform.scale(scale as f64, scale as f64), g);
                text::Text::new_color([0.0, 1.0, 1.0, 1.0], 32).draw(&(counter.tick().to_string()),
                                                                     &mut glyphs,
                                                                     &c.draw_state,
                                                                     transform,
                                                                     g);

            });
            let t5 = time::precise_time_ns();
            println!("Draw Frame: {:?}ms", (t5 - t4) / 1000000);


        } else {
            // println!("Decode: {:?}ns\tExec: {:?}ns\tInterrupt: {:?}ns",
            // t2 - t1,
            // t3 - t2,
            // t4 - t3);
            //

        }
    }


}
