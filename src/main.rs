extern crate gamecrab;
extern crate image;
extern crate piston_window;
use piston_window::*;
use gamecrab::{cpu, opcode, instr, interrupt};

fn get_gameboy_canvas(scale: u32) -> (u32, u32, image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) {
    let (width, height) = (160, 144);
    let mut canvas = image::ImageBuffer::new(width * scale, height * scale);
    for (x, y, pixel) in canvas.enumerate_pixels_mut() {
        *pixel = get_color((x * x / (2 * scale) + y * y / (2 * scale)) % 4);
    }
    (width * scale, height * scale, canvas)
}

fn get_color(idx: u32) -> image::Rgba<u8> {
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
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("kirby.gb");
    let mut next_addr = 0;
    let mut cycle_count = 0;
    let (width, height, canvas) = get_gameboy_canvas(4);
    let mut window: PistonWindow = WindowSettings::new("GameCrab", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap();
    let texture = Texture::from_image(&mut window.factory, &canvas, &TextureSettings::new())
        .unwrap();

    while let Some(e) = window.next() {
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

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            image(&texture, c.transform, g);
        });
    }

}
