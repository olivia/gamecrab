extern crate sdl2;
use self::sdl2::audio::{AudioCallback, AudioSpecDesired};
use std;
use std::time::Duration;
use cpu::*;

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

pub struct TriangleWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0...0.5 => self.volume,
                _ => -self.volume,
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a triangle wave
        for x in out.iter_mut() {
            *x = -self.volume + (self.phase + self.phase) * self.volume * 2.0;
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn get_duty(duty: u8, freq: u16) -> u16 {
    match duty {
        0 => freq / 8,
        1 => freq / 4,
        2 => freq / 2,
        3 => (freq / 4) * 3,
        _ => unreachable!(),
    }
}
fn gen_channel_2(cpu: &mut Cpu) -> Vec<i16> {
    let (NR21, NR22, NR23, NR24) = read_channel_2_addresses(cpu);
    let time_freq = (NR24 as u16 & 0b111) << 8 | NR23 as u16;
    let duty = NR21 >> 6;
    let length_load = NR21 & 0x3F;
    let not_time_freq = 2048 - time_freq as u32;
    let period = (44100 * not_time_freq / 131072) as u16;
    let volume_step = (1 << 9) as i16;
    let init_volume = ((NR22 & 0xF0) >> 4) as i16;
    let sample_count = period as usize;
    let mut result = vec![0; sample_count];
    let volume = volume_step * init_volume;
    let high_len = get_duty(duty, period);
    if volume == 0 {
        return Vec::new();
    } else {
        for x in 0..sample_count {
            let wave_pos = x as u16 % period;
            result[x] = if wave_pos <= high_len {
                volume
            } else {
                -volume
            };
        }
        result
    }
}

fn mix_channel_2(bytes_to_write: i32, result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (NR21, NR22, NR23, NR24) = read_channel_2_addresses(cpu);
    let freq = (NR24 as u16 & 0b111) << 8 | NR23 as u16;
    if freq == 0 {
        return;
    }
    let duty = NR21 >> 6;
    let length_load = NR21 & 0x3F;
    let period = 4 * (2048 - freq) as u16 | 1;
    // let period = 44100 / 440;
    println!("Period: {:?}", period);
    let volume_step = (1 << 9) as i16;
    let init_volume = ((NR22 & 0xF0) >> 4) as i16;
    let sample_count = bytes_to_write as usize;
    if init_volume != 0 {
        let volume = volume_step * 10;
        let high_len = get_duty(duty, period);
        for x in 0..sample_count {
            let wave_pos = x as u16 % period;
            result[x] += if wave_pos <= high_len {
                volume
            } else {
                -volume
            };
        }
    }
}
fn mix_channel_1(bytes_to_write: i32, result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (NR10, NR11, NR12, NR13, NR14) = read_channel_1_addresses(cpu);
    let freq = (NR14 as u16 & 0b111) << 8 | NR13 as u16;
    let duty = NR11 >> 6;
    let length_load = NR11 & 0x3F;
    let period = ((freq as u32 * 44100) / (4194305 / 16)) as u16 | 1;
    println!("Period: {:?}", period);
    let volume_step = (1 << 9) as i16;
    let init_volume = ((NR12 & 0xF0) >> 4) as i16;
    let sample_count = bytes_to_write as usize;
    if init_volume != 0 {
        let volume = volume_step * 10;
        let high_len = get_duty(duty, period);
        for x in 0..sample_count {
            let wave_pos = x as u16 % period;
            result[x] += if wave_pos <= high_len {
                volume
            } else {
                -volume
            };
        }
    }
}

pub fn init_audio() -> sdl2::audio::AudioQueue<i16> {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        // mono  -
        samples: None, // default sample size
    };



    let device = audio_subsystem.open_queue::<i16>(None, &desired_spec).unwrap();
    device.resume();
    device
}

pub fn play_audio(device: &sdl2::audio::AudioQueue<i16>, cpu: &mut Cpu) {
    //    let target_bytes = (44100 / 128) as i32;
    //    let mut result = vec![0; target_bytes as usize];
    //    mix_channel_1(target_bytes, &mut result, cpu);
    let result = gen_channel_2(cpu);
    device.queue(&result);
    // Start playback
}