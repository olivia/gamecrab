extern crate gamecrab;
use std::io::prelude::*;
use std::fs::File;
use gamecrab::{cpu, opcode, instr};

fn main() {
    let mut cpu : cpu::Cpu = Default::default();
    cpu.load_bootrom("DMG_ROM.bin");
    cpu.load_cart("kirby.gb");
    let mut f = File::open("DMG_ROM.bin").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).ok();
    let mut next_addr = 0;
    for _ in 1..10240009 {
        let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &mut cpu);
        println!("Address {:4>0X}: {:?} taking {:?} cycles", next_addr, instr, cycles);
        next_addr += op_length;
        next_addr = instr::exec_instr(instr, next_addr, &mut cpu);
    }
}
