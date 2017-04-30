extern crate sdl2;
use self::sdl2::audio::AudioSpecDesired;
use cpu::*;

pub struct WaveChannel {
    pub counter: u16,
    pub enabled: bool,
    pub freq_pos: u32,
    pub wave_pos: usize,
    pub volume: u8,
}

pub struct SquareChannel {
    pub counter: u16,
    pub enabled: bool,
    pub envelope_pos: u8,
    pub envelope_period: u8,
    pub freq_pos: u32,
    pub incr_vol: bool,
    pub volume: u8,
    pub wave_pos: usize,
    pub duty_table: [u8; 32],
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
    pub sweep_clock: u8,
    pub sweep_negate: bool,
    pub envelope_clock: u32,
    pub sample_length_arr: [u8; 512],
    pub sweep_period: u8,
    pub sweeping: bool,
    pub channel_1_shadow_freq: u32, // shadow frequency for sweeping
    pub channel_1_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_2_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_3_pos: u32,
    pub channel_3_wave_pos: u32,
    pub channel_1: SquareChannel,
    pub channel_2: SquareChannel,
    pub channel_3: WaveChannel,
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

impl Default for SquareChannel {
    fn default() -> SquareChannel {
        SquareChannel {
            counter: 0,
            enabled: false,
            envelope_pos: 0,
            freq_pos: 0,
            volume: 0,
            incr_vol: false,
            envelope_period: 0,
            wave_pos: 0,
            duty_table: get_duty_table(),
        }
    }
}
impl Default for WaveChannel {
    fn default() -> WaveChannel {
        WaveChannel {
            counter: 0,
            enabled: false,
            freq_pos: 0,
            wave_pos: 0,
            volume: 0,
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
            sweep_period: 0,
            sweep_negate: false,
            envelope_clock: 0,
            sample_length_arr: sample_len_arr(),
            audio_queue: init_audio(44100),
            sweeping: false,
            channel_1_shadow_freq: 0,
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

impl Apu {
    pub fn freq_sweep(&self, shift: u8, negate: bool) -> u32 {
        let freq = self.channel_1_shadow_freq;
        let delta = freq >> shift;
        cond!(negate, freq - delta, freq + delta)
    }
}

pub fn gen_samples(sample_len: u8, cpu: &mut Cpu) {
    let mut result = vec![0; sample_len as usize];
    let nr51 = cpu.memory[0xFF25];
    if cpu.apu.channel_1.enabled {
        let registers = read_channel_1_addresses(cpu);
        let mut channel = &mut cpu.apu.channel_1;
        let sound = ((nr51 >> 4) & 1, nr51 & 1);
        mix_channel_square(&mut result, channel, registers, sound);
    }
    if cpu.apu.channel_2.enabled {
        let registers = read_channel_2_addresses(cpu);
        let mut channel = &mut cpu.apu.channel_2;
        let sound = ((nr51 >> 5) & 1, (nr51 >> 1) & 1);
        mix_channel_square(&mut result, channel, registers, sound);
    }
    if cpu.apu.channel_3.enabled {
        let channel_3_registers = read_channel_3_addresses(cpu);
        let wave_table = &cpu.memory[0xFF30..(0xFF30 + 16)];
        let mut channel_3 = &mut cpu.apu.channel_3;
        let sound = ((nr51 >> 6) & 1, (nr51 >> 2) & 1);
        mix_channel_3(&mut result,
                      &mut channel_3,
                      wave_table,
                      channel_3_registers,
                      sound);
    }
    if cpu.apu.channel_4.enabled {
        let channel_4_registers = cpu.memory[0xFF22];
        let mut channel_4 = &mut cpu.apu.channel_4;
        mix_channel_4(&mut result, &mut channel_4, channel_4_registers, nr51);
    }
    cpu.apu.audio_vec_queue.append(&mut result);
}

pub fn queue(cpu: &mut Cpu) {
    if cpu.apu.audio_queue.size() == 0 {
        //    println!("Empty queue");
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
    if cpu.apu.channel_1.enabled && channel_1_length_enable {
        if cpu.apu.channel_1.counter == 0 {
            cpu.apu.channel_1.enabled = false;
        }
        cpu.apu.channel_1.counter -= 1;
    }
    if cpu.apu.channel_2.enabled && channel_2_length_enable {
        if cpu.apu.channel_2.counter == 0 {
            cpu.apu.channel_2.enabled = false;
        }
        cpu.apu.channel_2.counter -= 1;
    }

    if cpu.apu.channel_3.enabled && channel_3_length_enable {
        if cpu.apu.channel_3.counter == 0 {
            cpu.apu.channel_3.enabled = false;
        }
        cpu.apu.channel_3.counter -= 1;
    }

    if cpu.apu.channel_4.enabled && channel_4_length_enable {
        if cpu.apu.channel_4.counter == 0 {
            cpu.apu.channel_4.enabled = false;
        }
        cpu.apu.channel_4.counter -= 1;
    }
}

#[allow(non_snake_case)]
pub fn mix_channel_square(result: &mut Vec<i16>,
                          channel: &mut SquareChannel,
                          (duty_reg, freq_lo, freq_hi): (u8, u8, u8),
                          (left_out, right_out): (u8, u8)) {
    let time_freq = (freq_hi as u16 & 0b111) << 8 | freq_lo as u16;
    let duty = (duty_reg >> 6) as usize;
    let not_time_freq = 4 * (2048 - time_freq as u32);
    let init_volume = channel.volume as i16;
    let volume = match left_out + right_out {
        2 => (1 << 9) * init_volume,
        1 => (1 << 8) * init_volume,
        0 => 0,
        _ => unreachable!(),
    };
    let duty_start = duty * 8;
    let duty_table = &channel.duty_table[duty_start..];
    let downsample = 1 + 8192 / result.len();
    let init_freq = channel.freq_pos;
    let init_wave = channel.wave_pos;
    let volume_on = volume != 0;
    if volume_on {
        for x in 0..result.len() {
            let est_wave_pos = cond!(init_freq as usize > not_time_freq as usize + x * downsample,
                      init_wave,
                      (init_wave +
                       ((x * downsample + not_time_freq as usize - init_freq as usize) /
                        not_time_freq as usize) as usize) % 8);
            let duty_low = duty_table[est_wave_pos] == 0;
            let sample = cond!(duty_low, -volume, volume);
            result[x] += sample;
        }
    }
    channel.wave_pos = cond!(init_freq as usize > not_time_freq as usize + 8191,
                             init_wave,
                             (init_wave +
                              ((8191 + not_time_freq as usize - init_freq as usize) /
                               not_time_freq as usize) as usize) % 8);
    channel.freq_pos = cond!(init_freq < 8192,
                             (not_time_freq - (8192 - init_freq) % not_time_freq) % not_time_freq,
                             init_freq - 8192);
}

#[allow(non_snake_case)]
pub fn mix_channel_3(result: &mut Vec<i16>,
                     channel: &mut WaveChannel,
                     wave_table: &[u8],
                     registers: (u8, u8, u8, u8),
                     (left_out, right_out): (u8, u8)) {
    let (NR30, NR32, NR33, NR34) = registers;
    let time_freq = ((NR34 as u16 & 0b111) << 8) | NR33 as u16;
    let not_time_freq = 2 * (2048 - time_freq as u32);
    let volume_step = match left_out + right_out {
        2 => (1 << 9),
        1 => (1 << 8),
        0 => 0,
        _ => unreachable!(),
    } as i16;
    let volume_shift = ((NR32 & 0x60) >> 5) as i16;
    if (NR30 & 0x80) != 0 {
        let downsample = 1 + 8192 / result.len();
        let init_freq = channel.freq_pos;
        let init_wave = channel.wave_pos;
        for x in 0..result.len() {
            if volume_shift != 0 {
                let est_wave_pos =
                    cond!(init_freq as usize > not_time_freq as usize + x * downsample,
                          init_wave,
                          (init_wave +
                           ((x * downsample + not_time_freq as usize - init_freq as usize) /
                            not_time_freq as usize) as usize) % 32);
                let sample_cell = wave_table[est_wave_pos as usize / 2];
                let sample_is_left = est_wave_pos & 1 == 0;
                let sample = cond!(sample_is_left, sample_cell >> 4, sample_cell & 0x0F);
                result[x] += (volume_step * (2 * sample as i16 - 15)) / (1 << (volume_shift - 1));
            }
        }

        channel.wave_pos = cond!(init_freq as usize > not_time_freq as usize + 8191,
                      init_wave,
                      (init_wave +
                       ((8191 + not_time_freq as usize - init_freq as usize) /
                        not_time_freq as usize) as usize) % 32);
        channel.freq_pos = cond!(init_freq < 8192,
                                 (not_time_freq - (8192 - init_freq) % not_time_freq) %
                                 not_time_freq,
                                 init_freq - 8192);
    }
}

#[allow(non_snake_case)]
pub fn mix_channel_4(result: &mut Vec<i16>, channel: &mut NoiseChannel, nr43: u8, nr51: u8) {
    let right_out = (nr51 >> 3) & 1;
    let left_out = (nr51 >> 7) & 1;
    let divisors = [8, 16, 32, 48, 64, 80, 96, 112];
    let dividing_ratio = divisors[(nr43 & 0x7) as usize];
    let shift_clock_freq = nr43 >> 4 as u32;
    let timer_freq = dividing_ratio << shift_clock_freq;
    let volume_step = match left_out + right_out {
        2 => (1 << 9),
        1 => (1 << 8),
        0 => 0,
        _ => unreachable!(),
    } as i16;
    let volume_init = channel.volume as i16;
    let volume = volume_step * volume_init;
    let half_width = nr43 & 0x08 != 0; // whether the shift register is 15bits of 7 bits
    if shift_clock_freq < 14 && timer_freq as u32 != 0 {
        let downsample = 1 + 8192 / result.len();
        let mut freq = channel.freq_pos;
        let mut sample_idx = 0;
        for x in 0..8192 {
            // cycles
            if freq == 0 {
                freq = timer_freq;
                let lfsr = channel.lfsr;
                // Even though the documentation says that it should use the lower two bits, this sounds more accurate :/
                let tap_shift = cond!(half_width, 0, 0);
                let new_bit = ((lfsr ^ (lfsr >> 1)) >> tap_shift) & 1;
                let part_res = (new_bit << 14) | (lfsr >> 1);
                channel.lfsr = if half_width {
                    (part_res & (0x7FFF - 0x0040)) | (new_bit << 6)
                } else {
                    part_res
                };
            }
            freq -= 1;
            if volume != 0 && x % downsample == 0 {
                result[sample_idx] += cond!((channel.lfsr & 1) == 0, volume, -volume);
                sample_idx += 1;
            }
        }
        channel.freq_pos = freq;
    }
}

pub fn step_sweep(cpu: &mut Cpu) {
    let nr10 = read_address(0xFF10, cpu);
    let sweep_period = (nr10 >> 4) & 7;

    if cpu.apu.sweeping && sweep_period != 0 {
        if cpu.apu.sweep_clock == 0 {
            let (shift, negate) = (nr10 & 7, nr10 & 8 != 0);
            let sweep = cpu.apu.freq_sweep(shift, negate);
            cpu.apu.sweep_clock = sweep_period;
            if sweep <= 2047 {
                cpu.apu.channel_1_shadow_freq = sweep;
                let nr14 = read_address(0xFF14, cpu);
                write_address(0xFF13, (sweep & 0xFF) as u8, cpu);
                write_address(0xFF14, (nr14 & 0xF0) | (sweep >> 8) as u8, cpu);
                // check again
                let sweep_check = cpu.apu.freq_sweep(shift, negate);
                if sweep_check > 2047 {
                    cpu.apu.channel_1.enabled = false;
                }
            }
        } else {
            cpu.apu.sweep_clock -= 1;
        }
    }
}

pub fn step_envelope(cpu: &mut Cpu) {
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
}

pub fn step(cpu: &mut Cpu) {
    cpu.apu.master_clock = (cpu.apu.master_clock + 1) % 512;
    let sample_len = cpu.apu.sample_length_arr[cpu.apu.master_clock as usize];
    if cpu.apu.master_clock % 2 == 0 {
        step_length(cpu);
    }
    if cpu.apu.master_clock % 4 == 0 {
        step_sweep(cpu);
    }
    if cpu.apu.master_clock % 8 == 7 {
        step_envelope(cpu);
    }
    gen_samples(sample_len, cpu);
}

fn get_duty_table() -> [u8; 32] {
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0]
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
