extern crate rustgirl;
use std::io::prelude::*;
use std::fs::File;
use rustgirl::opcode;
use rustgirl::cpu::Cpu;
use rustgirl::instr;

fn main() {
    // Representing A, F, B, C, D, E, H, L in that order
    let mut cpu : Cpu = Default::default();
    let mut f = File::open("DMG_ROM.bin").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).ok();
    let mut next_addr = 0;
    for _ in 1..10240 {
        let (op_length, instr, cycles) = opcode::lookup_op(next_addr, &buffer);
        println!("Address {:4>0X}: {:?} taking {:?} cycles", next_addr, instr, cycles);
        next_addr += op_length;
        next_addr = instr::exec_instr(instr, next_addr, &mut cpu);
    }
}
