#![feature(test)]
macro_rules! cond {
    ($a:expr, $b:expr, $c:expr) => (
    if $a { $b } else { $c }
    )
}
extern crate test;
extern crate gamecrab;
use gamecrab::cpu::*;

// pub fn mix_channel_4(result: &mut Vec<i16>) {
// let nr43 = 0xBF;//cpu.memory[0xFF22];
// let nr51 = 0xF; //cpu.memory[0xFF25];
// let right_out = (nr51 >> 3) & 1;
// let left_out = (nr51 >> 7) & 1;
// let divisors = [8, 16, 32, 48, 64, 80, 96, 112];
// let dividing_ratio = divisors[(nr43 & 0x7) as usize];
// let shift_clock_freq = nr43 >> 4 as u32;
// let mut timer_freq = dividing_ratio << shift_clock_freq;
// let volume_step = (1 << 9) as i16;
// let volume_init = 15 as i16;//cpu.apu.channel_4.volume as i16;
// let volume = match left_out + right_out {
// 2 => volume_step * volume_init,
// 1 => volume_step * volume_init / 2,
// 0 => 0,
// _ => unreachable!(),
// };
// let half_width = nr43 & 0x08 != 0; // whether the shift register is 15bits of 7 bits
// let mut channel_lfsr = 0xFF;
// if shift_clock_freq < 15 && timer_freq as u32 != 0 {
// let downsample = 1 + 8192 / result.len();
// let mut freq = timer_freq / 3;
// for x in 0..8192 {
// cycles
// if freq == 0 {
// freq = timer_freq;
// let lfsr = channel_lfsr;
// let new_bit = (lfsr ^ (lfsr >> 1)) & 1;
// let part_res = (new_bit << 14) | (lfsr >> 1);
// channel_lfsr = if half_width {
// (part_res & (0x7FFF - 0x0040)) | (new_bit << 6)
// } else {
// part_res
// };
// }
// freq -= 1;
// let sample_idx = x / downsample;
// if volume != 0 && x % downsample == 0 {
// result[sample_idx] += cond!((channel_lfsr & 1) == 0, volume, -volume);
// }
// }
// timer_freq = freq;
// }
// }
//
pub fn mix_channel_4(result: &mut Vec<i16>, nr43: u8, nr51: u8) {
    let right_out = (nr51 >> 3) & 1;
    let left_out = (nr51 >> 7) & 1;
    let divisors = [8, 16, 32, 48, 64, 80, 96, 112];
    let dividing_ratio = divisors[(nr43 & 0x7) as usize];
    let shift_clock_freq = nr43 >> 4 as u32;
    let mut timer_freq = dividing_ratio << shift_clock_freq;
    let volume_step = (1 << 9) as i16;
    let volume_init = 15 as i16;//cpu.apu.channel_4.volume as i16;
    let volume = match left_out + right_out {
        2 => volume_step * volume_init,
        1 => volume_step * volume_init / 2,
        0 => 0,
        _ => unreachable!(),
    };
    let half_width = nr43 & 0x08 != 0; // whether the shift register is 15bits of 7 bits
    let mut channel_lfsr = 0xFF;
    if shift_clock_freq < 15 && timer_freq as u32 != 0 {
        let downsample = 1 + 8192 / result.len();
        let mut freq = timer_freq / 3;
        for x in 0..8192 {
            // cycles
            if freq == 0 {
                freq = timer_freq;
                let lfsr = channel_lfsr;
                let new_bit = (lfsr ^ (lfsr >> 1)) & 1;
                let part_res = (new_bit << 14) | (lfsr >> 1);
                channel_lfsr = if half_width {
                    (part_res & 0x7FBF) | (new_bit << 6)
                } else {
                    part_res
                };
            }
            freq -= 1;
            if volume != 0 && x % downsample == 0 {
                let sample_idx = x / downsample;
                result[sample_idx] += cond!((channel_lfsr & 1) == 0, volume, -volume);
            }
        }
        timer_freq = freq;
    }
}

#[cfg(test)]
use test::Bencher;

#[bench]
fn bench_noise(b: &mut Bencher) {
    b.iter(|| {

        let mut vec = vec![0;86];
        let n = test::black_box(1000);
        mix_channel_4(&mut vec, 0xBF, 0xF);
    });
}

#[bench]
fn bench_zeros(b: &mut Bencher) {
    b.iter(|| {
        for y in 0..512 {
            let mut vec = vec![2; 87];
            let mut vec = vec![2; 87];
            let mut vec = vec![2; 87];
            let mut vec = vec![2; 87];
            let n = test::black_box(1000);
            let mut m = 0;
            for i in 0..8192 {
                if i % 95 == 0 {
                    let offset = 4;
                    vec[m] += 4;
                    m += 1;
                }
            }
        }
    });
}