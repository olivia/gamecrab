extern crate sdl2;
use self::sdl2::audio::AudioSpecDesired;
use cpu::*;

pub struct AudioChannel {
    pub counter: u16,
    pub enabled: bool,
    pub envelope_pos: u8,
    pub envelope_period: u8,
    pub incr_vol: bool,
    pub volume: u8,
}

pub struct NoiseChannel {
    pub counter: u16,
    pub enabled: bool,
    pub envelope_pos: u8,
    pub volume: u8,
    pub lfsr: u16,
    pub freq_pos: u32,
    pub incr_vol: bool,
    pub envelope_period: u8,
}

pub struct Apu {
    pub master_clock: u32,
    pub length_clock: u32,
    pub sweep_clock: u32,
    pub envelope_clock: u32,
    pub sample_length_arr: [u8; 512],
    pub channel_1_time_freq: u32, // shadow frequency for sweeping
    pub channel_1_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_2_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_3_pos: u32,
    pub channel_3_wave_pos: u32,
    pub channel_1: AudioChannel,
    pub channel_2: AudioChannel,
    pub channel_3: AudioChannel,
    pub channel_4: NoiseChannel,
    pub audio_queue: sdl2::audio::AudioQueue<i16>,
    pub prev_size: i32,
    pub audio_vec_queue: Vec<i16>,
    pub audio_freq: u32,
}

impl Default for NoiseChannel {
    fn default() -> NoiseChannel {
        NoiseChannel {
            counter: 0,
            enabled: false,
            envelope_pos: 0,
            volume: 0,
            freq_pos: 0,
            lfsr: 0x7FFF,
            incr_vol: false,
            envelope_period: 0,
        }
    }
}
impl Default for AudioChannel {
    fn default() -> AudioChannel {
        AudioChannel {
            counter: 0,
            enabled: false,
            envelope_pos: 0,
            volume: 0,
            incr_vol: false,
            envelope_period: 0,
        }
    }
}

impl Default for Apu {
    fn default() -> Apu {
        let audio_freq = 44100;
        Apu {
            master_clock: 0,
            length_clock: 0,
            sweep_clock: 0,
            envelope_clock: 0,
            sample_length_arr: sample_len_arr(),
            audio_queue: init_audio(44100),
            channel_1_time_freq: 0,
            channel_1_pos: 0,
            channel_2_pos: 0,
            channel_3_pos: 0,
            channel_3_wave_pos: 0,
            channel_1: Default::default(),
            channel_2: Default::default(),
            channel_3: Default::default(),
            channel_4: Default::default(),
            audio_freq: audio_freq,
            audio_vec_queue: Vec::new(),
            prev_size: 0,
        }
    }
}

pub fn gen_samples(sample_len: u8, cpu: &mut Cpu) {
    let mut result = vec![0; sample_len as usize];
    if cpu.apu.channel_1.enabled {
        mix_channel_1(&mut result, cpu);
    }
    if cpu.apu.channel_2.enabled {
        mix_channel_2(&mut result, cpu);
    }
    if cpu.apu.channel_3.enabled {
        mix_channel_3(&mut result, cpu);
    }
    if cpu.apu.channel_4.enabled {
        mix_channel_4(&mut result, cpu);
    }
    cpu.apu.audio_vec_queue.append(&mut result);
}

pub fn queue(cpu: &mut Cpu) {
    if cpu.apu.audio_queue.size() == 0 {
        println!("Empty queue");
    }

    // This is the maximum the queue can be delayed by
    cpu.apu.audio_queue.queue(&cpu.apu.audio_vec_queue);
    cpu.apu.audio_vec_queue.clear();
    cpu.apu.prev_size = cpu.apu.audio_queue.size() as i32;
}

pub fn step_length(cpu: &mut Cpu) {
    let channel_1_length_enable = read_address(0xFF14, cpu) & 0x40 != 0;
    let channel_2_length_enable = read_address(0xFF19, cpu) & 0x40 != 0;
    let channel_3_length_enable = read_address(0xFF1E, cpu) & 0x40 != 0;
    let channel_4_length_enable = read_address(0xFF23, cpu) & 0x40 != 0;
    if cpu.apu.channel_4.enabled && channel_4_length_enable && cpu.apu.channel_4.counter != 0 {
        if cpu.apu.channel_4.counter == 0 {
            cpu.apu.channel_4.enabled = false;
        }
        cpu.apu.channel_4.counter -= 1;
    }
    if cpu.apu.channel_1.enabled && channel_1_length_enable && cpu.apu.channel_1.counter != 0 {
        if cpu.apu.channel_1.counter == 0 {
            cpu.apu.channel_1.enabled = false;
        }
        cpu.apu.channel_1.counter -= 1;
    }
    if cpu.apu.channel_2.enabled && channel_2_length_enable && cpu.apu.channel_2.counter != 0 {
        if cpu.apu.channel_2.counter == 0 {
            cpu.apu.channel_2.enabled = false;
        }
        cpu.apu.channel_2.counter -= 1;
    }

    if cpu.apu.channel_3.enabled && channel_3_length_enable && cpu.apu.channel_3.counter != 0 {
        if cpu.apu.channel_3.counter == 0 {
            cpu.apu.channel_3.enabled = false;
        }
        cpu.apu.channel_3.counter -= 1;
    }
    // TODO do other channels
}
#[allow(non_snake_case)]
pub fn mix_channel_1(result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (_NR10, NR11, NR12, NR13, NR14) = read_channel_1_addresses(cpu);
    let time_freq = (NR14 as u16 & 0b111) << 8 | NR13 as u16;
    let duty = NR11 >> 6;
    let not_time_freq = 2048.0 - time_freq as f32;
    let period = 44100.0 * not_time_freq / 131072.0;
    let volume_step = (1 << 9) as i16;
    let init_volume = cpu.apu.channel_1.volume as i16;
    let sample_count = result.len();
    let volume = volume_step * init_volume;
    let high_len = get_duty(duty, period) as u32;
    if period as u32 != 0 {
        for x in 0..sample_count {
            let wave_pos = (x as u32 + cpu.apu.channel_1_pos) % period as u32;
            result[x] += cond!(wave_pos <= high_len, volume, -volume);
        }
    }
    cpu.apu.channel_1_pos = cpu.apu.channel_1_pos + sample_count as u32;
}

#[allow(non_snake_case)]
pub fn mix_test(result: &mut Vec<i16>, cpu: &mut Cpu) {
    let duty = 1;
    let not_time_freq = 440.0;
    let period = (44100.0 * not_time_freq / 131072.0) as f32;
    let volume_step = (1 << 9) as i16;
    let init_volume = 10 as i16;
    let sample_count = result.len();
    let volume = volume_step * init_volume;
    let high_len = get_duty(duty, period) as u16;
    for x in 0..sample_count {
        let wave_pos = (x as u16 + cpu.apu.channel_2_pos as u16) % (period as u16);
        result[x] += cond!(wave_pos <= high_len, volume, -volume);
    }
    cpu.apu.channel_2_pos = cpu.apu.channel_2_pos + sample_count as u32;
}

#[allow(non_snake_case)]
pub fn mix_channel_2(result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (NR21, NR22, NR23, NR24) = read_channel_2_addresses(cpu);
    let time_freq = (NR24 as u16 & 0b111) << 8 | NR23 as u16;
    let duty = NR21 >> 6;
    let not_time_freq = 2048.0 - time_freq as f32;
    let period = 44100.0 * not_time_freq / 131072.0;
    let volume_step = (1 << 9) as i16;
    let init_volume = cpu.apu.channel_2.volume as i16;
    let sample_count = result.len();
    let volume = volume_step * init_volume;
    let high_len = get_duty(duty, period) as u16;
    if period as u32 != 0 {
        for x in 0..sample_count {
            let wave_pos = (x as u16 + cpu.apu.channel_2_pos as u16) % (period as u16);
            result[x] += cond!(wave_pos <= high_len, volume, -volume);
        }
    }
    cpu.apu.channel_2_pos = cpu.apu.channel_2_pos + sample_count as u32;

}


#[allow(non_snake_case)]
pub fn mix_channel_3(result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (NR30, _, NR32, NR33, NR34) = read_channel_3_addresses(cpu);
    let time_freq = ((NR34 as u16 & 0b111) << 8) | NR33 as u16;
    let not_time_freq = 2 * (2048 - time_freq as u32);
    let volume_step = (1 << 9) as i16;
    let volume_shift = ((NR32 & 0x60) >> 4) as i16;
    if (NR30 & 0x80) != 0 && not_time_freq as u32 != 0 {
        let downsample = 1 + 8192 / result.len();
        let mut freq = cpu.apu.channel_3_pos;
        for x in 0..8192 {
            // cycles
            if freq == 0 {
                freq = not_time_freq;
                cpu.apu.channel_3_wave_pos += 1;
                cpu.apu.channel_3_wave_pos %= 32;
                // reload
            }
            freq -= 1;
            let sample_cell = read_address(0xFF30 + (cpu.apu.channel_3_wave_pos as usize) / 2, cpu);
            let sample = cond!(cpu.apu.channel_3_wave_pos % 2 == 0,
                               sample_cell >> 4,
                               sample_cell & 0x0F);
            let sample_idx = x / downsample;
            if volume_shift != 0 && x % downsample == 0 {
                result[sample_idx] += (volume_step * (2 * sample as i16 - 15)) /
                                      (1 << (volume_shift - 1));
            }
        }
        cpu.apu.channel_3_pos = freq;
    }
}

#[allow(non_snake_case)]
pub fn mix_channel_4(result: &mut Vec<i16>, cpu: &mut Cpu) {
    let (_, NR42, NR43, _) = read_channel_4_addresses(cpu);
    let divisors = [8, 16, 32, 48, 64, 80, 96, 112];
    let dividing_ratio = divisors[(NR43 & 0x7) as usize];
    let shift_clock_freq = (NR43 >> 4) as u32;
    let timer_freq = (dividing_ratio << shift_clock_freq);
    let volume_step = (1 << 9) as i16;
    let volume_init = cpu.apu.channel_4.volume as i16;
    let volume = volume_step * volume_init;
    let half_width = NR43 & 0x08 != 0; // whether the shift register is 15bits of 7 bits
    if shift_clock_freq < 15 && timer_freq as u32 != 0 {
        let downsample = 1 + 8192 / result.len();
        let mut freq = cpu.apu.channel_4.freq_pos;
        for x in 0..8192 {
            // cycles
            if freq == 0 {
                freq = timer_freq;
                let lfsr = cpu.apu.channel_4.lfsr;
                let new_bit = (lfsr ^ (lfsr >> 1)) & 1;
                let part_res = (new_bit << 14) | (lfsr >> 1);
                cpu.apu.channel_4.lfsr = if half_width {
                    (part_res & (0x7FFF - 0x0040)) | (new_bit << 6)
                } else {
                    part_res
                };
            }
            freq -= 1;
            let sample_idx = x / downsample;
            if volume != 0 && x % downsample == 0 {
                result[sample_idx] += cond!((cpu.apu.channel_4.lfsr & 1) == 0, volume, -volume);
            }
        }
        cpu.apu.channel_4.freq_pos = freq;
    }
}

pub fn step(cpu: &mut Cpu) {
    cpu.apu.master_clock = (cpu.apu.master_clock + 1) % 512;
    let sample_len = cpu.apu.sample_length_arr[cpu.apu.master_clock as usize];
    if cpu.apu.master_clock % 2 == 0 {
        step_length(cpu);
    }
    if cpu.apu.master_clock % 4 == 0 {
        cpu.apu.sweep_clock = (cpu.apu.sweep_clock + 1) % 512;
    }
    if cpu.apu.master_clock % 8 == 7 {
        if cpu.apu.channel_1.enabled && cpu.apu.channel_1.envelope_period != 0 {
            if cpu.apu.channel_1.incr_vol && cpu.apu.channel_1.volume < 15 {
                if cpu.apu.channel_1.envelope_pos == 0 {
                    cpu.apu.channel_1.volume += 1;
                    cpu.apu.channel_1.envelope_pos = cpu.apu.channel_1.envelope_period;
                } else {
                    cpu.apu.channel_1.envelope_pos -= 1;
                }
            } else if !cpu.apu.channel_1.incr_vol && cpu.apu.channel_1.volume > 0 {
                if cpu.apu.channel_1.envelope_pos == 0 {
                    cpu.apu.channel_1.volume -= 1;
                    cpu.apu.channel_1.envelope_pos = cpu.apu.channel_1.envelope_period;
                } else {
                    cpu.apu.channel_1.envelope_pos -= 1;
                }
            }
        }
        if cpu.apu.channel_2.enabled && cpu.apu.channel_2.envelope_period != 0 {
            if cpu.apu.channel_2.incr_vol && cpu.apu.channel_2.volume < 15 {
                if cpu.apu.channel_2.envelope_pos == 0 {
                    cpu.apu.channel_2.volume += 1;
                    cpu.apu.channel_2.envelope_pos = cpu.apu.channel_2.envelope_period;
                } else {
                    cpu.apu.channel_2.envelope_pos -= 1;
                }
            } else if !cpu.apu.channel_2.incr_vol && cpu.apu.channel_2.volume > 0 {
                if cpu.apu.channel_2.envelope_pos == 0 {
                    cpu.apu.channel_2.volume -= 1;
                    cpu.apu.channel_2.envelope_pos = cpu.apu.channel_2.envelope_period;
                } else {
                    cpu.apu.channel_2.envelope_pos -= 1;
                }
            }
        }
        if cpu.apu.channel_4.enabled && cpu.apu.channel_4.envelope_period != 0 {
            if cpu.apu.channel_4.incr_vol && cpu.apu.channel_4.volume < 15 {
                if cpu.apu.channel_4.envelope_pos == 0 {
                    cpu.apu.channel_4.volume += 1;
                    cpu.apu.channel_4.envelope_pos = cpu.apu.channel_4.envelope_period;
                } else {
                    cpu.apu.channel_4.envelope_pos -= 1;
                }
            } else if !cpu.apu.channel_4.incr_vol && cpu.apu.channel_4.volume > 0 {
                if cpu.apu.channel_4.envelope_pos == 0 {
                    cpu.apu.channel_4.volume -= 1;
                    cpu.apu.channel_4.envelope_pos = cpu.apu.channel_4.envelope_period;
                } else {
                    cpu.apu.channel_4.envelope_pos -= 1;
                }
            }
        }
        cpu.apu.envelope_clock = (cpu.apu.envelope_clock + 1) % 512;
    }
    gen_samples(sample_len, cpu);
}

fn get_duty(duty: u8, freq: f32) -> f32 {
    match duty {
        0 => freq / 8.0,
        1 => freq / 4.0,
        2 => freq / 2.0,
        3 => (freq / 4.0) * 3.0,
        _ => unreachable!(),
    }
}

pub fn init_audio(freq: i32) -> sdl2::audio::AudioQueue<i16> {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(freq),
        channels: Some(1),
        // mono  -
        samples: None, /* default sample size
                        *        samples: Some(32768), // default sample size */
    };

    let device = audio_subsystem.open_queue::<i16>(None, &desired_spec)
        .unwrap();
    device.resume();
    device
}

pub fn sample_len_arr() -> [u8; 512] {
    let mut arr = [0; 512];
    for i in 0..512 {
        arr[i] = cond!(i % 128 == 0 || i % 8 == 4, 87, 86);
    }
    arr
}
