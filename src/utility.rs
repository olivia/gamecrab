macro_rules! cond {
    ($a:expr, $b:expr, $c:expr) => (
    if $a { $b } else { $c }
    )
}

pub fn wrapping_off_u16_i8(u_num: u16, i_num: i8) -> u16 {
    if i_num < 0 {
        u_num.wrapping_sub(i_num.abs() as u16)
    } else {
        u_num.wrapping_add(i_num as u16)
    }
}