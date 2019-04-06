extern crate fps_counter;
extern crate gamecrab;
extern crate image;
extern crate piston_window;
extern crate time;
use self::image::{ImageBuffer, Rgba};
use fps_counter::*;
use gamecrab::{apu, cpu, instr, interrupt, keyboard, lcd, opcode, ppu};
use piston_window::texture::Filter;
use piston_window::*;

fn get_gameboy_canvas(scale: u32) -> (u32, u32, ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let (width, height) = (160, 144);
    let mut canvas = ImageBuffer::new(width, height);
    for (_, _, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = image::Rgba([0, 0, 0, 255]);
    }
    (width * scale, height * scale, canvas)
}

#[allow(dead_code)]
fn disassemble_rom(start: usize, limit: usize) {
    let mut cpu: cpu::Cpu = Default::default();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.has_booted = true;
    cpu.load_cart("Stargate.gb");
    let mut next_addr = start;
    for _ in 0..limit {
        let (op_length, instr, _) = opcode::lookup_op(next_addr, &mut cpu);
        println!("0x{:4>0X}:\t{:?}", next_addr, instr);
        next_addr += op_length;
    }
}

fn run_rom() {
    let opengl = OpenGL::V3_2;
    let mut cpu: cpu::Cpu = Default::default();
    let mut counter = FPSCounter::new();

    let mut next_addr = 0;
    let scale = 4;
    let (width, height, canvas) = get_gameboy_canvas(scale);
    let mut window: PistonWindow = WindowSettings::new("🎮🦀", [width, height])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("a.gb");
    let factory = window.factory.clone();
    let font = "FiraSans-Regular.ttf";
    let mut texture_settings = TextureSettings::new();
    texture_settings.set_filter(Filter::Nearest);
    let mut texture = Texture::from_image(&mut window.factory, &canvas, &texture_settings).unwrap();
    let mut glyphs = Glyphs::new(font, factory, texture_settings).unwrap();
    let mut frame = canvas;
    let mut mod_cycles = 0;
    let mut frame_mod_cycles = 0;
    let line_scan_cycles = 456;
    let frame_cycles = 70224;
    let _cpu_hz = 4194304;
    let hz_512_div = 8192;
    let mut screen_buffer = [0; 256 * 256];
    let mut apu_mod_cycles = 0;
    let mut start_updating = false;
    window.set_max_fps(60);
    window.set_ups(512);
    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            keyboard::handle_keypress(key, &mut cpu);
        };

        if let Some(Button::Keyboard(key)) = e.release_args() {
            keyboard::handle_key_release(key, &mut cpu);
        };

        if let Some(_) = e.idle_args() {
            if cpu.cart_loaded {
                start_updating = true;
            };
        }
        if !cpu.cart_loaded {
            continue;
        };

        if let Some(_) = e.update_args() {
            if !start_updating {
                continue;
            }
            apu::queue(&mut cpu);
            let mut lcd_power_on = lcd::LCDC::Power.is_set(&mut cpu);
            while apu_mod_cycles <= hz_512_div {
                if cpu.halted {
                    if !lcd_power_on {
                        mod_cycles = 0;
                        frame_mod_cycles = 0;
                        lcd::ScreenMode::VBlank.set(&mut cpu);
                    } else {
                        mod_cycles += 4;
                        frame_mod_cycles += 4;
                        apu_mod_cycles += 4;
                        // Finished ~456 clocks
                        if mod_cycles > line_scan_cycles {
                            let ly = cpu::read_address(0xFF44, &mut cpu);
                            ppu::render_scanline(ly, &mut screen_buffer, &mut frame, &mut cpu);
                            lcd::increment_ly(&mut cpu);
                            mod_cycles %= line_scan_cycles;
                        }
                        lcd::update_status(frame_mod_cycles, &mut cpu);
                    }
                    if cpu.dma_transfer_cycles_left > 0 {
                        cpu.dma_transfer_cycles_left -= 4 as i32;
                    }
                    if cpu.serial_transfer_timer > 0 {
                        cpu.serial_transfer_timer -= 4;
                        if cpu.serial_transfer_timer <= 0 {
                            cpu::write_address(0xFF01, 0xFF, &mut cpu);
                            interrupt::Interrupt::Serial.request(&mut cpu);
                        }
                    }
                    cpu.inc_clocks(4);
                    let interrupt_addr = interrupt::exec_halt_interrupts(next_addr, &mut cpu);
                    if !cpu.halted && (interrupt_addr == next_addr + 1) {
                        mod_cycles += 4;
                        apu_mod_cycles += 4;
                        cpu.inc_clocks(4);
                        frame_mod_cycles += 4;
                    } else if !cpu.halted {
                        mod_cycles += 24;
                        apu_mod_cycles += 24;
                        cpu.inc_clocks(24);
                        frame_mod_cycles += 24;
                    }
                    next_addr = interrupt_addr;
                } else {
                    let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);

                    if false && cpu.has_booted {
                        println!("0x{:4>0X}:\t{:?}", next_addr, instr);
                    }

                    match instr {
                        opcode::OpCode::HALT => {
                            cpu.halted = true;
                            continue;
                        }
                        _ => {}
                    }

                    next_addr += op_length;
                    let (cycle_offset, new_addr) = instr::exec_instr(instr, next_addr, &mut cpu);

                    next_addr = new_addr;
                    lcd_power_on = lcd::LCDC::Power.is_set(&mut cpu);

                    if !lcd_power_on {
                        mod_cycles = 0;
                        frame_mod_cycles = 0;
                        lcd::ScreenMode::VBlank.set(&mut cpu);
                    } else {
                        apu_mod_cycles += cycles + cycle_offset;
                        mod_cycles += cycles + cycle_offset;
                        frame_mod_cycles += cycles + cycle_offset;
                        // Finished ~456 clocks
                        if mod_cycles > line_scan_cycles {
                            let ly = cpu::read_address(0xFF44, &mut cpu);
                            ppu::render_scanline(ly, &mut screen_buffer, &mut frame, &mut cpu);
                            lcd::increment_ly(&mut cpu);
                            mod_cycles %= line_scan_cycles;
                        }
                        lcd::update_status(frame_mod_cycles, &mut cpu);
                    }
                    if cpu.dma_transfer_cycles_left > 0 {
                        cpu.dma_transfer_cycles_left -= (cycles + cycle_offset) as i32;
                    }
                    if cpu.serial_transfer_timer > 0 {
                        cpu.serial_transfer_timer -= (cycles + cycle_offset) as i32;
                        if cpu.serial_transfer_timer <= 0 {
                            cpu::write_address(0xFF01, 0xFF, &mut cpu);
                            interrupt::Interrupt::Serial.request(&mut cpu);
                        }
                    }

                    cpu.inc_clocks(cycles + cycle_offset);
                    let interrupt_addr = interrupt::exec_interrupts(next_addr, &mut cpu);
                    if next_addr != interrupt_addr {
                        mod_cycles += 20;
                        cpu.inc_clocks(20);
                        frame_mod_cycles += 20;
                        if lcd_power_on {
                            apu_mod_cycles += 20;
                        }
                        next_addr = interrupt_addr;
                    }
                    lcd_power_on = lcd::LCDC::Power.is_set(&mut cpu);
                }
            }
            if lcd_power_on {
                apu::step(&mut cpu);
            }
            apu_mod_cycles %= hz_512_div;
        }

        if let Some(_) = e.render_args() {
            if !start_updating {
                continue;
            }
            if cpu.cart_loaded {}
            let lcd_power_on = lcd::LCDC::Power.is_set(&mut cpu);
            if lcd_power_on && frame_mod_cycles > frame_cycles {
                frame_mod_cycles %= frame_cycles;
                texture.update(&mut window.encoder, &frame).unwrap();
            }
            window.draw_2d(&e, |c, g| {
                let transform = c.transform.trans(10.0, 30.0);

                clear([1.0; 4], g);
                image(&texture, c.transform.scale(scale as f64, scale as f64), g);
                text::Text::new_color([0.0, 1.0, 1.0, 1.0], 32).draw(
                    &counter.tick().to_string(),
                    &mut glyphs,
                    &c.draw_state,
                    transform,
                    g,
                );

                text::Text::new_color([0.0, 1.0, 1.0, 1.0], 32).draw(
                    &(format!(
                        "BG: {:?}, W: {:?}, S: {:?}",
                        cpu.background_mode, cpu.window_mode, cpu.sprite_mode
                    )),
                    &mut glyphs,
                    &c.draw_state,
                    c.transform.trans(50.0, 30.0),
                    g,
                );
            });
        }
    }
}

fn main() {
    run_rom();
} //d
  // disassemble_rom(0xB7, 100);
