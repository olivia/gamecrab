extern crate gamecrab;
extern crate image;
extern crate piston_window;
extern crate fps_counter;
extern crate time;
use piston_window::*;
use piston_window::texture::Filter;
use fps_counter::*;
use gamecrab::{cpu, opcode, instr, interrupt, lcd, ppu};

fn get_gameboy_canvas(scale: u32) -> (u32, u32, image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let (width, height) = (160, 144);
    let mut canvas = image::ImageBuffer::new(width, height);
    for (_, _, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = image::Rgba([0, 0, 0, 255]);
    }
    (width * scale, height * scale, canvas)
}
fn disassemble_rom(start: usize, limit: usize) {
    let mut cpu: cpu::Cpu = Default::default();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("test_instrs/01-special.gb");
    let mut next_addr = start;
    for _ in 0..limit {
        let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);
        println!("0x{:4>0X}:\t{:?}", next_addr, instr);
        next_addr += op_length;
    }
}

fn run_rom() {
    let opengl = OpenGL::V3_2;
    let mut cpu: cpu::Cpu = Default::default();
    let mut counter = FPSCounter::new();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("tetris.gb");
    let mut next_addr = 0;
    let scale = 3;
    let (width, height, canvas) = get_gameboy_canvas(scale);
    let mut window: PistonWindow = WindowSettings::new("🎮🦀", [width, height])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let factory = window.factory.clone();
    let font = "FiraSans-Regular.ttf";
    let mut glyphs = Glyphs::new(font, factory).unwrap();
    let mut texture_settings = TextureSettings::new();
    texture_settings.set_filter(Filter::Nearest);
    let mut texture = Texture::from_image(&mut window.factory, &canvas, &texture_settings).unwrap();
    let mut frame = canvas;
    let mut mod_cycles = 0;
    let mut frame_mod_cycles = 0;
    let mut tick_mod_cycles = false;
    let line_scan_cycles = 456;
    let frame_cycles = 70224;
    let mut screen_buffer = [image::Rgba([0x7F, 0x85, 0x51, 255]); 256 * 256];
    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::A => cpu.key_start = true,
                Key::S => cpu.key_select = true,
                Key::D => cpu.key_b = true,
                Key::F => cpu.key_a = true,
                Key::Up => cpu.key_up = true,
                Key::Down => cpu.key_down = true,
                Key::Left => cpu.key_left = true,
                Key::Right => cpu.key_right = true,
                _ => {}
            };
        };
        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::A => cpu.key_start = false,
                Key::S => cpu.key_select = false,
                Key::D => cpu.key_b = false,
                Key::F => cpu.key_a = false,
                Key::Up => cpu.key_up = false,
                Key::Down => cpu.key_down = false,
                Key::Left => cpu.key_left = false,
                Key::Right => cpu.key_right = false,
                _ => {}
            };
        };
        if let Some(_) = e.render_args() {
            while !lcd::LCDC::Power.is_set(&mut cpu) || frame_mod_cycles < frame_cycles {
                if lcd::LCDC::Power.is_set(&mut cpu) {
                    tick_mod_cycles = true
                }
                let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);
                if false && cpu.has_booted {
                    println!("0x{:4>0X}:\t{:?}", next_addr, instr);
                }

                next_addr += op_length;
                let (cycle_offset, new_addr) = instr::exec_instr(instr, next_addr, &mut cpu);

                next_addr = new_addr;

                if tick_mod_cycles {
                    mod_cycles += cycles + cycle_offset;
                    frame_mod_cycles += cycles + cycle_offset;
                }

                // Finished ~456 clocks
                if mod_cycles > line_scan_cycles {
                    if lcd::LCDC::Power.is_set(&mut cpu) {
                        lcd::increment_ly(&mut cpu);
                    }
                    mod_cycles %= line_scan_cycles;
                }

                cpu.inc_clocks(cycles + cycle_offset);
                lcd::update_status(frame_mod_cycles, &mut cpu);
                next_addr = interrupt::exec_interrupts(next_addr, &mut cpu);
            }
            frame_mod_cycles %= frame_cycles;

            ppu::render_frame(&mut screen_buffer, &mut frame, &mut cpu);
            texture.update(&mut window.encoder, &frame).unwrap();
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


        } else {
            // println!("Decode: {:?}ns\tExec: {:?}ns\tInterrupt: {:?}ns",
            // t2 - t1,
            // t3 - t2,
            // t4 - t3);
            //

        }
    }

}

fn main() {
    run_rom();
    // disassemble_rom(0x38, 100);
}
