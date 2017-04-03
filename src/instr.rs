use cpu::*;
use register::*;
use flag;
use flag::Flag;
use opcode::*;

pub fn ld_m(reg: Register, val: u16, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => {
            write_multi_register(reg, val, cpu);
            curr_addr
        }
    }
}

pub fn add_from_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    cpu.a = a_val.wrapping_add(num);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::Z, cpu.a == 0, cpu);
    flag::bool_set(Flag::H, 0x10 <= ((a_val & 0x0F) + (num & 0x0F)), cpu);
    flag::bool_set(Flag::C, 0x0100 <= (a_val as u16 + num as u16), cpu);
    curr_addr
}

pub fn sub_from_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    flag::set(Flag::N, cpu);
    flag::bool_set(Flag::Z, a_val == num, cpu);
    flag::bool_set(Flag::H, (a_val & 0x0F) < (num & 0x0F), cpu);
    flag::bool_set(Flag::C, a_val < num, cpu);
    cpu.a = cpu.a.wrapping_sub(num);
    curr_addr
}

pub fn ld(reg: Register, val: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match reg {
        _ => {
            write_register(reg, val, cpu);
            curr_addr
        }
    }
}

pub fn ld_r(to_reg: Register, from_reg: Register, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match (to_reg, from_reg) {
        (Register::HLP, _) |
        (Register::HLM, _) => {
            let hl = read_multi_register(Register::HL, cpu);
            let new_hl = match to_reg {
                Register::HLP => hl + 1,
                _ => hl - 1,
            };
            write_register(Register::HL_ADDR, read_register(from_reg, cpu), cpu);
            write_multi_register(Register::HL, new_hl, cpu);
            curr_addr
        }
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
    flag::bool_set(Flag::Z, read_register(reg, cpu) & (1 << bit_pos) == 0, cpu);
}

pub fn xor(reg: Register, cpu: &mut Cpu) -> () {
    let reg_a_val = read_register(Register::A, cpu);
    let reg_val = read_register(reg, cpu);
    let res = reg_a_val ^ reg_val;
    let res_f = (if res == 0 { 1 } else { 0 }) << 7;

    write_register(Register::A, res, cpu);
    write_register(Register::F, res_f, cpu);
}

pub fn cp(num: u8, cpu: &mut Cpu) {
    let a_val = cpu.a;
    flag::set(Flag::N, cpu);
    flag::bool_set(Flag::Z, a_val == num, cpu);
    flag::bool_set(Flag::H, (a_val & 0x0F) < (num & 0x0F), cpu);
    flag::bool_set(Flag::C, a_val < num, cpu);
}

pub fn rl(reg: Register, conditional_z: bool, cpu: &mut Cpu) {
    let old_c_bit = if flag::is_set(Flag::C, cpu) { 1 } else { 0 };
    let new_c_bit = read_register(reg, cpu) & 0x80;
    write_register(reg, (read_register(reg, cpu) << 1) + old_c_bit, cpu);
    flag::bool_set(Flag::Z, conditional_z && read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, new_c_bit != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn rlc(reg: Register, conditional_z: bool, cpu: &mut Cpu) {
    let new_c_bit = read_register(reg, cpu) & 0x80;
    write_register(reg, read_register(reg, cpu).rotate_left(1), cpu);
    flag::bool_set(Flag::Z, conditional_z && read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, new_c_bit != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn rr(reg: Register, conditional_z: bool, cpu: &mut Cpu) {
    let old_c_bit = if flag::is_set(Flag::C, cpu) { 1 } else { 0 };
    let new_c_bit = read_register(reg, cpu) & 1;
    write_register(reg, read_register(reg, cpu) >> 1 + old_c_bit << 7, cpu);
    flag::bool_set(Flag::Z, conditional_z && read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, new_c_bit != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn sra(reg: Register, cpu: &mut Cpu) {
    let new_c_bit = read_register(reg, cpu) & 1;
    let msb = read_register(reg, cpu) & 0b10000000;
    write_register(reg, (read_register(reg, cpu) >> 1) | msb, cpu);
    flag::bool_set(Flag::Z, read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, new_c_bit != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn srl(reg: Register, cpu: &mut Cpu) {
    let lsb = read_register(reg, cpu) & 1;
    write_register(reg, read_register(reg, cpu) >> 1, cpu);
    flag::bool_set(Flag::Z, read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, lsb != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn sla(reg: Register, cpu: &mut Cpu) {
    let msb = read_register(reg, cpu) & 0b10000000;
    write_register(reg, read_register(reg, cpu) << 1, cpu);
    flag::bool_set(Flag::Z, read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, msb != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn rrc(reg: Register, conditional_z: bool, cpu: &mut Cpu) {
    let new_c_bit = read_register(reg, cpu) & 1;
    write_register(reg, read_register(reg, cpu) >> 1 + new_c_bit << 7, cpu);
    flag::bool_set(Flag::Z, conditional_z && read_register(reg, cpu) == 0, cpu);
    flag::bool_set(Flag::C, new_c_bit != 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
}

pub fn inc_u8(reg: Register, cpu: &mut Cpu) {
    let reg_val = read_register(reg, cpu);
    let res = reg_val.wrapping_add(1);
    write_register(reg, res, cpu);
    flag::bool_set(Flag::Z, res == 0, cpu);
    flag::bool_set(Flag::H,
                   (reg_val & 0b00001000) != 0 && (res & 0b00001000) == 0,
                   cpu);
    flag::reset(Flag::N, cpu);
}

pub fn inc_u16(reg: Register, cpu: &mut Cpu) {
    write_multi_register(reg, read_multi_register(reg, cpu).wrapping_add(1), cpu);
}

pub fn dec_u8(reg: Register, cpu: &mut Cpu) {
    let reg_val = read_register(reg, cpu);
    let res = reg_val.wrapping_sub(1);
    write_register(reg, res, cpu);
    flag::bool_set(Flag::Z, res == 0, cpu);
    flag::bool_set(Flag::H,
                   !((reg_val & 0b00001000) == 0 && (res & 0b00001000) != 0),
                   cpu);
    flag::set(Flag::N, cpu);
}

pub fn dec_u16(reg: Register, cpu: &mut Cpu) {
    write_multi_register(reg, read_multi_register(reg, cpu).wrapping_sub(1), cpu);
}

pub fn ccf(cpu: &mut Cpu) {
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
    flag::bool_set(Flag::C, !flag::is_set(Flag::C, cpu), cpu);
}

pub fn scf(cpu: &mut Cpu) {
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
    flag::set(Flag::C, cpu);
}

pub fn cpl(cpu: &mut Cpu) {
    write_register(Register::A, 0xFF - read_register(Register::A, cpu), cpu);
    flag::set(Flag::N, cpu);
    flag::set(Flag::H, cpu);
}

fn swap(reg: Register, cpu: &mut Cpu) {
    let reg_val = read_register(reg, cpu);
    write_register(reg, ((reg_val & 0x0F) << 4) | ((reg_val & 0xF0) >> 4), cpu);
    flag::bool_set(Flag::Z, reg_val == 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
    flag::reset(Flag::C, cpu);
}

fn cycle_offset(op: OpCode, cpu: &mut Cpu) -> usize {
    use opcode::OpCode::*;

    match op {
        JP_C(cond, _) | JR_C(cond, _) => if test_cond(cond, cpu) { 4 } else { 0 },
        CALL_C(cond, _) | RET_C(cond) => if test_cond(cond, cpu) { 12 } else { 0 },
        _ => 0,
    }
}

pub fn exec_instr(op: OpCode, curr_addr: usize, cpu: &mut Cpu) -> (usize, usize) {
    use opcode::OpCode::*;

    let offset = cycle_offset(op, cpu);
    let new_addr = match op {
        DI => {
            cpu.interrupt_master_enabled = false;
            curr_addr
        }
        EI => {
            cpu.interrupt_master_enabled = true;
            curr_addr
        }
        JP(addr) => addr as usize,
        JP_C(cond, reg_addr) => {
            if test_cond(cond, cpu) {
                reg_addr as usize
            } else {
                curr_addr
            }
        }
        JP_HL => read_multi_register(Register::HL, cpu) as usize,
        NOP => curr_addr,
        XOR(reg) => {
            xor(reg, cpu);
            curr_addr
        }
        LD(reg, val) => ld(reg, val, curr_addr, cpu),
        LD_M(reg, val) => ld_m(reg, val, curr_addr, cpu),
        LD_R(to_reg, from_reg) => ld_r(to_reg, from_reg, curr_addr, cpu),
        BIT(bit_pos, reg) => {
            bit(bit_pos, reg, cpu);
            curr_addr
        }
        JR(offset) => (curr_addr as i16 + offset as i16) as usize,
        JR_C(cond, offset) => {
            if test_cond(cond, cpu) {
                (curr_addr as i16 + offset as i16) as usize
            } else {
                curr_addr
            }
        }
        DEC_F(reg) => {
            dec_u8(reg, cpu);
            curr_addr
        }
        DEC(reg) => {
            dec_u16(reg, cpu);
            curr_addr
        }
        INC_F(reg) => {
            inc_u8(reg, cpu);
            curr_addr
        }
        INC(reg) => {
            inc_u16(reg, cpu);
            curr_addr
        }
        CALL(reg_addr) | RST(reg_addr) => {
            stack_push(curr_addr as u16, cpu);
            reg_addr as usize
        }
        CALL_C(cond, reg_addr) => {
            if test_cond(cond, cpu) {
                stack_push(curr_addr as u16, cpu);
                reg_addr as usize
            } else {
                curr_addr
            }
        }
        PUSH(reg) => {
            stack_push(read_multi_register(reg, cpu), cpu);
            curr_addr
        }
        POP(reg) => {
            write_multi_register(reg, stack_pop(cpu), cpu);
            curr_addr
        }
        RL(reg) => {
            rl(reg, true, cpu);
            curr_addr
        }
        RLC(reg) => {
            rlc(reg, true, cpu);
            curr_addr
        }
        RLA => {
            rl(Register::A, false, cpu);
            curr_addr
        }
        RLCA => {
            rlc(Register::A, false, cpu);
            curr_addr
        }
        RR(reg) => {
            rr(reg, true, cpu);
            curr_addr
        }
        RRC(reg) => {
            rrc(reg, true, cpu);
            curr_addr
        }
        RRA => {
            rr(Register::A, false, cpu);
            curr_addr
        }
        RRCA => {
            rrc(Register::A, false, cpu);
            curr_addr
        }
        SRA(reg) => {
            sra(reg, cpu);
            curr_addr
        }
        SLA(reg) => {
            sla(reg, cpu);
            curr_addr
        }
        SRL(reg) => {
            srl(reg, cpu);
            curr_addr
        }
        RET => stack_pop(cpu) as usize,
        RET_C(cond) => {
            if test_cond(cond, cpu) {
                stack_pop(cpu) as usize
            } else {
                curr_addr
            }
        }
        CP(reg) => {
            cp(read_register(reg, cpu), cpu);
            curr_addr
        }
        CP_d8(num) => {
            cp(num, cpu);
            curr_addr
        }
        SCF => {
            scf(cpu);
            curr_addr
        }
        CCF => {
            ccf(cpu);
            curr_addr
        }
        CPL => {
            cpl(cpu);
            curr_addr
        }
        SWAP(reg) => {
            swap(reg, cpu);
            curr_addr
        }
        ADD(Register::A, reg) => add_from_a(read_register(reg, cpu), curr_addr, cpu),
        SUB(reg) => sub_from_a(read_register(reg, cpu), curr_addr, cpu),
        _ => {
            println!("Please implement {:?}", op);
            unreachable!()
        }
    };

    (offset, new_addr)
}