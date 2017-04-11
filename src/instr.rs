use cpu::*;
use register::*;
use flag;
use flag::Flag;
use opcode::*;
use utility::*;

pub fn ld_m(reg: Register, val: u16, curr_addr: usize, cpu: &mut Cpu) -> usize {
    write_multi_register(reg, val, cpu);
    curr_addr
}

pub fn ldhl_sp(val: i8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let sp_val = cpu.sp as i32;
    let sp = cpu.sp;
    let new_val = wrapping_off_u16_i8(sp, val);
    cpu.sp = new_val;
    flag::reset(Flag::Z, cpu);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::H,
                   0x000F < ((sp_val & 0x000F) + ((val as i32) & 0x000F)),
                   cpu);
    flag::bool_set(Flag::C,
                   0x00FF < ((sp_val & 0x00FF) + ((val as i32) & 0x00FF)),
                   cpu);
    curr_addr
}

pub fn add_to_sp(num: i8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let sp_val = cpu.sp as i32;
    cpu.sp = wrapping_off_u16_i8(cpu.sp, num);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::H,
                   0x000F < ((sp_val & 0x000F) + ((num as i32) & 0x000F)),
                   cpu);
    flag::bool_set(Flag::C,
                   0x00FF < ((sp_val & 0x00FF) + ((num as i32) & 0x00FF)),
                   cpu);
    curr_addr
}


pub fn add_to_hl(num: u16, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let hl_val = read_multi_register(Register::HL, cpu);
    write_multi_register(Register::HL, hl_val.wrapping_add(num), cpu);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::H, 0x0FFF < ((hl_val & 0x0FFF) + (num & 0x0FFF)), cpu);
    flag::bool_set(Flag::C, 0xFFFF < (hl_val as u32 + num as u32), cpu);
    curr_addr
}

pub fn a_or_val(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    cpu.a |= num;
    flag::bool_set(Flag::Z, cpu.a == 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
    flag::reset(Flag::C, cpu);
    curr_addr
}

pub fn a_and_val(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    cpu.a &= num;
    flag::bool_set(Flag::Z, cpu.a == 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::set(Flag::H, cpu);
    flag::reset(Flag::C, cpu);
    curr_addr
}

pub fn add_to_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    let res = a_val.wrapping_add(num);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::Z, res == 0, cpu);
    flag::bool_set(Flag::H, 0x10 <= ((a_val & 0x0F) + (num & 0x0F)), cpu);
    flag::bool_set(Flag::C, 0x0100 <= (a_val as u16 + num as u16), cpu);
    cpu.a = res;
    curr_addr
}

pub fn addc_to_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    let carry = if flag::is_set(Flag::C, cpu) { 1 } else { 0 };
    let res = a_val.wrapping_add(num).wrapping_add(carry);
    flag::reset(Flag::N, cpu);
    flag::bool_set(Flag::Z, res == 0, cpu);
    flag::bool_set(Flag::H,
                   0x10 <= ((a_val & 0x0F) + (num & 0x0F) + carry),
                   cpu);
    flag::bool_set(Flag::C,
                   0x0100 <= (a_val as u16 + num as u16 + carry as u16),
                   cpu);
    cpu.a = res;
    curr_addr
}

pub fn sub_from_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    flag::set(Flag::N, cpu);
    flag::bool_set(Flag::Z, a_val == num, cpu);
    flag::bool_set(Flag::H, (a_val & 0x0F) < (num & 0x0F), cpu);
    flag::bool_set(Flag::C, a_val < num, cpu);
    cpu.a = a_val.wrapping_sub(num);
    curr_addr
}

pub fn subc_from_a(num: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    let a_val = cpu.a;
    let carry = if flag::is_set(Flag::C, cpu) { 1 } else { 0 };
    let res = a_val.wrapping_sub(num).wrapping_sub(carry);
    flag::set(Flag::N, cpu);
    flag::bool_set(Flag::Z, res == 0, cpu);
    flag::bool_set(Flag::H, (a_val & 0x0F) < ((num & 0x0F) + carry), cpu);
    flag::bool_set(Flag::C, (a_val as u16) < (num as u16 + carry as u16), cpu);
    cpu.a = res;
    curr_addr
}

pub fn ld(reg: Register, val: u8, curr_addr: usize, cpu: &mut Cpu) -> usize {
    write_register(reg, val, cpu);
    curr_addr
}

pub fn ld_r(to_reg: Register, from_reg: Register, curr_addr: usize, cpu: &mut Cpu) -> usize {
    match (to_reg, from_reg) {
        (Register::HLP, _) |
        (Register::HLM, _) => {
            write_register(Register::HL_ADDR, read_register(from_reg, cpu), cpu);

            let hl = read_multi_register(Register::HL, cpu);
            let new_hl = match to_reg {
                Register::HLP => hl.wrapping_add(1),
                _ => hl.wrapping_sub(1),
            };
            write_multi_register(Register::HL, new_hl, cpu);
            curr_addr
        }
        (_, Register::HLP) |
        (_, Register::HLM) => {
            let hl_val = read_register(Register::HL_ADDR, cpu);
            write_register(to_reg, hl_val, cpu);

            let hl_addr = read_multi_register(Register::HL, cpu);
            let new_hl_addr = match from_reg {
                Register::HLP => hl_addr.wrapping_add(1),
                _ => hl_addr.wrapping_sub(1),
            };
            write_multi_register(Register::HL, new_hl_addr, cpu);
            curr_addr
        } 
        (Register::CH, _) => {
            write_register(to_reg, read_register(from_reg, cpu), cpu);
            curr_addr
        }
        (_, Register::CH) => {
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

pub fn daa(curr_addr: usize, cpu: &mut Cpu) -> usize {
    let mut a_val = cpu.a as u16;
    let h_flag = flag::is_set(Flag::H, cpu);
    let c_flag = flag::is_set(Flag::C, cpu);
    let low_n = a_val & 0xF;

    if !flag::is_set(Flag::N, cpu) {
        if h_flag || low_n > 9 {
            a_val = a_val.wrapping_add(0x06);
        }
        if c_flag || a_val > 0x9F {
            a_val = a_val.wrapping_add(0x60);
        }
    } else {
        if h_flag {
            a_val = a_val.wrapping_sub(6) & 0xFF;
        }
        if c_flag {
            a_val = a_val.wrapping_sub(0x60);
        }
    }

    flag::reset(Flag::H, cpu);
    flag::bool_set(Flag::C, (a_val & 0x100) == 0x100, cpu);
    a_val &= 0xFF;
    flag::bool_set(Flag::Z, a_val == 0, cpu);
    cpu.a = a_val as u8;
    curr_addr
}

pub fn xor_d8(val: u8, cpu: &mut Cpu) -> () {
    cpu.a ^= val;
    flag::bool_set(Flag::Z, cpu.a == 0, cpu);
    flag::reset(Flag::N, cpu);
    flag::reset(Flag::H, cpu);
    flag::reset(Flag::C, cpu);
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
    write_register(reg, (read_register(reg, cpu) >> 1) + (old_c_bit << 7), cpu);
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
    write_register(reg, (read_register(reg, cpu) >> 1) + (new_c_bit << 7), cpu);
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
    flag::bool_set(Flag::H, (res & 0x0F) == 0, cpu);
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
    flag::bool_set(Flag::H, (reg_val & 0x0F) == 0, cpu);
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
    use self::Register::*;

    let offset = cycle_offset(op, cpu);
    let new_addr = match op {
        DAA => daa(curr_addr, cpu),
        DI => {
            cpu.interrupt_master_enabled = false;
            curr_addr
        }
        EI => {
            cpu.interrupt_master_enabled = true;
            curr_addr
        }
        RES(pos, reg) => {
            let val = read_register(reg, cpu);
            let new_val = val & (0xFF - (1 << pos));
            write_register(reg, new_val, cpu);
            curr_addr
        }
        SET(pos, reg) => {
            write_register(reg, read_register(reg, cpu) | (1 << pos), cpu);
            let val = read_register(reg, cpu);
            let new_val = val | (1 << pos);
            write_register(reg, new_val, cpu);
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
            xor_d8(read_register(reg, cpu), cpu);
            curr_addr
        }
        XOR_d8(num) => {
            xor_d8(num, cpu);
            curr_addr
        }
        LD(reg, val) => ld(reg, val, curr_addr, cpu),
        LD_M(reg, val) => ld_m(reg, val, curr_addr, cpu),
        LDHL_SP(val) => ldhl_sp(val, curr_addr, cpu),
        LD_SP_HL => {
            cpu.sp = read_multi_register(Register::HL, cpu);
            curr_addr
        }
        LD_ADDR_SP(addr) => {
            let sp = cpu.sp;
            safe_write_address(addr as usize, (sp & 0xFF) as u8, cpu);
            safe_write_address(addr.wrapping_add(1) as usize, (sp >> 8) as u8, cpu);
            curr_addr
        }
        LD_R(to_reg, from_reg) => ld_r(to_reg, from_reg, curr_addr, cpu),
        BIT(bit_pos, reg) => {
            bit(bit_pos, reg, cpu);
            curr_addr
        }
        JR(offset) => wrapping_off_u16_i8(curr_addr as u16, offset) as usize,
        JR_C(cond, offset) => {
            if test_cond(cond, cpu) {
                wrapping_off_u16_i8(curr_addr as u16, offset) as usize
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
            let val = read_multi_register(reg, cpu);
            stack_push(val, cpu);
            curr_addr
        }
        POP(reg) => {
            let val = stack_pop(cpu);
            write_multi_register(reg, val, cpu);
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
        RETI => {
            cpu.interrupt_master_enabled = true;
            stack_pop(cpu) as usize
        }
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
        AND(reg) => a_and_val(read_register(reg, cpu), curr_addr, cpu),
        AND_d8(num) => a_and_val(num, curr_addr, cpu),
        OR(reg) => a_or_val(read_register(reg, cpu), curr_addr, cpu),
        OR_d8(num) => a_or_val(num, curr_addr, cpu),
        ADD_r8(SP, offset) => add_to_sp(offset, curr_addr, cpu),
        ADD_d8(A, num) => add_to_a(num, curr_addr, cpu),
        ADD(HL, reg) => add_to_hl(read_multi_register(reg, cpu), curr_addr, cpu),
        ADD(A, reg) => add_to_a(read_register(reg, cpu), curr_addr, cpu),
        ADD_C(A, reg) => addc_to_a(read_register(reg, cpu), curr_addr, cpu),
        ADD_C_d8(A, num) => addc_to_a(num, curr_addr, cpu),
        SUB(reg) => sub_from_a(read_register(reg, cpu), curr_addr, cpu),
        SUB_d8(num) => sub_from_a(num, curr_addr, cpu),
        SUB_C(A, reg) => subc_from_a(read_register(reg, cpu), curr_addr, cpu),
        SUB_C_d8(A, num) => subc_from_a(num, curr_addr, cpu),
        _ => {
            println!("Please implement {:?}", op);
            unreachable!()
        }
    };

    (offset, new_addr)
}