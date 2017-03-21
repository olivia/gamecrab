use cpu::*;
use register::*;
use flag;
use flag::Flag;
use opcode::*;

pub fn ld_m(reg: Register, val: u16, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => { write_multi_register(reg, val, cpu); curr_addr }
    }
}

pub fn ld(reg: Register, val: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => { write_register(reg, val, cpu); curr_addr }
    }
}

pub fn ld_r(to_reg: Register,  from_reg: Register, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match (to_reg, from_reg) {
        (Register::HLP, _) | (Register::HLM, _) => { 
            let hl = read_multi_register(Register::HL, cpu);
            let new_hl = match to_reg { Register::HLP => hl + 1, _ => hl - 1 };
            write_register(Register::HL_ADDR, read_register(from_reg, cpu), cpu); 
            write_multi_register(Register::HL, new_hl, cpu); 
            curr_addr 
        },
        (Register::CH, _) => {
            write_register(to_reg, read_register(from_reg, cpu), cpu);
            curr_addr 
        }
        _ => { 
            write_register(to_reg, read_register(from_reg, cpu), cpu); 
            curr_addr 
        }
    }
}

pub fn bit(bit_pos: u8, reg: Register, cpu: &mut Cpu) -> () {
    flag::set(Flag::H, cpu);
    flag::reset(Flag::N, cpu);
    if read_register(reg, cpu) & (1 << bit_pos) == 0 { 
        flag::set(Flag::Z, cpu) 
    } else { 
        flag::reset(Flag::Z, cpu) 
    }
}

pub fn xor(reg: Register, cpu: &mut Cpu) -> () {
    let reg_a_val = read_register(Register::A, cpu);
    let reg_val = read_register(reg, cpu);
    let res = reg_a_val^reg_val;
    let res_f = (if res == 0 { 1 } else { 0 }) << 7;

    write_register(Register::A, res, cpu);
    write_register(Register::F, res_f, cpu);
}

pub fn cp(num: u8, cpu: &mut Cpu) {
    let a_val = cpu.a;
    flag::set(Flag::N, cpu);
    if a_val == num { flag::set(Flag::Z, cpu) } else { flag::reset(Flag::Z, cpu) }
    if (a_val & 0x0F) < (num & 0x0F) { flag::set(Flag::H, cpu) } else { flag::reset(Flag::H, cpu) }
    if a_val < num { flag::set(Flag::C, cpu) } else { flag::reset(Flag::C, cpu) }
}

pub fn rl(reg: Register, cpu: &mut Cpu) {
    let old_c_bit = (read_register(Register::F, cpu) & flag::flag_bit(Flag::C)) >> 4; 
    let new_c_bit = (read_register(reg, cpu) & 0b10000000) >> 7; 
    write_register(reg, read_register(reg, cpu) << 1 + old_c_bit, cpu);
    if read_register(reg, cpu) == 0 { flag::set(Flag::Z, cpu) } else { flag::reset(Flag::Z, cpu) }
    if new_c_bit == 0 { flag::reset(Flag::C, cpu) } else { flag::set(Flag::C, cpu) }
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn rlc(reg: Register, cpu: &mut Cpu) {
    let new_c_bit = (read_register(reg, cpu) & 0b10000000) >> 7; 
    write_register(reg, read_register(reg, cpu) << 1 + new_c_bit, cpu);
    if read_register(reg, cpu) == 0 { flag::set(Flag::Z, cpu) } else { flag::reset(Flag::Z, cpu) }
    if new_c_bit == 0 { flag::reset(Flag::C, cpu) } else { flag::set(Flag::C, cpu) }
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn inc_u8(reg: Register, cpu: &mut Cpu) {
  let reg_val = read_register(reg, cpu);
  let res = reg_val.wrapping_add(1);
  if res == 0 { flag::set(Flag::Z, cpu) };
  if (reg_val & 0b00001000) != 0 && (res & 0b00001000) == 0 {
      flag::set(Flag::H, cpu)
  } else { 
      flag::reset(Flag::H, cpu) 
  }
  flag::reset(Flag::N, cpu);
} 

pub fn inc_u16(reg: Register, cpu: &mut Cpu) {
  write_multi_register(reg, read_multi_register(reg, cpu).wrapping_add(1), cpu);
}

pub fn dec_u8(reg: Register, cpu: &mut Cpu) {
  let reg_val = read_register(reg, cpu);
  let res = reg_val.wrapping_sub(1);
  if res == 0 { flag::set(Flag::Z, cpu) };
  if (reg_val & 0b00001000) == 0 && (res & 0b00001000) != 0 {
      flag::reset(Flag::H, cpu) 
  } else { 
      flag::set(Flag::H, cpu)
  }
  flag::set(Flag::N, cpu);
} 

pub fn dec_u16(reg: Register, cpu: &mut Cpu) {
  write_multi_register(reg, read_multi_register(reg, cpu).wrapping_sub(1), cpu);
}

pub fn exec_instr(op: OpCode, curr_addr: usize, cpu: &mut Cpu) -> usize {
    use opcode::OpCode::*;

    match op {
        JP(addr) => addr as usize,
        JP_HL => read_multi_register(Register::HL, cpu) as usize,
        NOP => curr_addr,
        XOR(reg) => { xor(reg, cpu); curr_addr },
        LD(reg, val) => ld(reg, val, curr_addr, cpu),
        LD_M(reg, val) => ld_m(reg, val, curr_addr, cpu),
        LD_R(to_reg, from_reg) => ld_r(to_reg, from_reg, curr_addr, cpu),
        BIT(bit_pos, reg) => { bit(bit_pos, reg, cpu); curr_addr },
        JR_C(cond, offset) => if test_cond(cond, cpu) { (curr_addr as i16 + offset as i16) as usize } else { curr_addr },
        DEC_F(reg) => { dec_u8(reg, cpu); curr_addr },
        DEC(reg) => { dec_u16(reg, cpu); curr_addr },
        INC_F(reg) => { inc_u8(reg, cpu); curr_addr },
        INC(reg) => { inc_u16(reg, cpu); curr_addr },
        CALL(reg_addr) => { stack_push(curr_addr as u16, cpu); reg_addr as usize },
        CALL_C(cond, reg_addr) => if test_cond(cond, cpu) { stack_push(curr_addr as u16, cpu); reg_addr as usize } else { curr_addr },
        PUSH(reg) => { stack_push(read_multi_register(reg, cpu), cpu); curr_addr },
        POP(reg) => { write_multi_register(reg, stack_pop(cpu), cpu); curr_addr },
        RL(reg) => { rl(reg, cpu); curr_addr },
        RLC(reg) => { rlc(reg, cpu); curr_addr },
        RLA => { rl(Register::A, cpu); curr_addr },
        RLCA => { rlc(Register::A, cpu); curr_addr },
        RET => { stack_pop(cpu) as usize },
        CP(reg) => { cp(read_register(reg, cpu), cpu); curr_addr },
        CP_d8(num) => { cp(num, cpu); curr_addr },
        _ => unreachable!()
    }
}