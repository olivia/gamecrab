extern crate gamecrab;
extern crate image;
extern crate piston_window;
extern crate fps_counter;
extern crate time;
use piston_window::*;
use fps_counter::*;
use gamecrab::{cpu, opcode, instr, interrupt, lcd};

fn get_gameboy_canvas(scale: u32) -> (u32, u32, image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let (width, height) = (160, 144);
    let mut canvas = image::ImageBuffer::new(width * scale, height * scale);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = get_color((x * x / (2 * scale) + y * y / (2 * scale)) as u8 % 4);
    }
    (width * scale, height * scale, canvas)
}

fn render_frame(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                scale: u32,
                cpu: &mut cpu::Cpu) {
    let mut screen_buffer = [get_color(0); 256 * 256];
    if lcd::LCDC::Power.is_set(cpu) {
        write_background(&mut screen_buffer, cpu);
        write_window(&mut screen_buffer, cpu);
        write_sprites(&mut screen_buffer, cpu);
        buffer_to_image_buffer(canvas, scale, screen_buffer)
    }
}

fn write_background(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    use cpu::*;
    if lcd::LCDC::BGEnable.is_set(cpu) {
        let start = 0x9800;
        for offset in 0..(32 * 32) {
            write_tile(read_address(start + offset, cpu) as usize,
                       8 * (offset % 256),
                       8 * (offset / 256),
                       buffer,
                       cpu);
        }
    }
}

fn write_tile(tile_num: usize,
              x: usize,
              y: usize,
              buffer: &mut [image::Rgba<u8>; 256 * 256],
              cpu: &mut cpu::Cpu) {
    use cpu::*;
    let tile_map_start = 0x8000;
    for row in 0..8 {
        // write row
        let left_line = read_address(tile_map_start + 16 * tile_num + row * 2, cpu);
        let right_line = read_address(tile_map_start + 16 * tile_num + 1 + row * 2, cpu);

        buffer[(y + row) * 256 + x] = get_color((left_line >> 6) & 0b11);
        buffer[(y + row) * 256 + x + 1] = get_color((left_line >> 4) & 0b11);
        buffer[(y + row) * 256 + x + 2] = get_color((left_line >> 2) & 0b11);
        buffer[(y + row) * 256 + x + 3] = get_color(left_line & 0b11);
        buffer[(y + row) * 256 + x + 4] = get_color((right_line >> 6) & 0b11);
        buffer[(y + row) * 256 + x + 5] = get_color((right_line >> 4) & 0b11);
        buffer[(y + row) * 256 + x + 6] = get_color((right_line >> 2) & 0b11);
        buffer[(y + row) * 256 + x + 7] = get_color(right_line & 0b11);
    }
}

fn write_window(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    if lcd::LCDC::WindowEnable.is_set(cpu) {
    }
}

fn write_sprites(buffer: &mut [image::Rgba<u8>; 256 * 256], cpu: &mut cpu::Cpu) {
    if lcd::LCDC::SpritesEnable.is_set(cpu) {

    }
}

fn buffer_to_image_buffer(canvas: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
                          scale: u32,
                          buffer: [image::Rgba<u8>; 256 * 256]) {
    let (width, height) = (160, 144);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        let idx = (x / scale) + 256 * (y / scale);
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
    let mut cpu: cpu::Cpu = Default::default();
    let mut counter = FPSCounter::new();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("kirby.gb");
    let mut next_addr = 0;
    let mut cycle_count = 0;
    let (width, height, canvas) = get_gameboy_canvas(4);
    let mut screen_buffer = [image::Rgba([0, 0, 0, 255]); 256 * 256];
    let mut window: PistonWindow = WindowSettings::new("GameCrab", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap();
    window.set_ups(1000000);

    let factory = window.factory.clone();
    let font = "FiraSans-Regular.ttf";
    let mut glyphs = Glyphs::new(font, factory).unwrap();

    let mut texture = Texture::from_image(&mut window.factory, &canvas, &TextureSettings::new())
        .unwrap();
    let mut frame = canvas;
    while let Some(e) = window.next() {
        let start = time::precise_time_ns();
        let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);
        println!("Address {:4>0X}: {:?} taking {:?} cycles",
                 next_addr,
                 instr,
                 cycles);
        next_addr += op_length;
        let (cycle_offset, new_addr) = instr::exec_instr(instr, next_addr, &mut cpu);

        next_addr = new_addr;
        cycle_count += cycles + cycle_offset;
        next_addr = interrupt::exec_interrupts(next_addr, &mut cpu);

        if let Some(_) = e.render_args() {
            render_frame(&mut frame, 4, &mut cpu);
            texture.update(&mut window.encoder, &frame)
                .unwrap();
            window.draw_2d(&e, |c, g| {
                let transform = c.transform.trans(10.0, 30.0);

                clear([1.0; 4], g);
                image(&texture, c.transform, g);
                text::Text::new_color([0.0, 1.0, 1.0, 1.0], 32).draw(&(counter.tick().to_string()),
                                                                     &mut glyphs,
                                                                     &c.draw_state,
                                                                     transform,
                                                                     g);

            });
        }
    }

}