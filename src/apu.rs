extern crate sdl2;
use self::sdl2::audio::AudioSpecDesired;
use cpu::*;

pub struct AudioChannel {
    pub counter: u8,
    pub enabled: bool,
    pub envelope_pos: u16,
    pub volume: u8,
}

pub struct Apu {
    pub master_clock: u32,
    pub length_clock: u32,
    pub sweep_clock: u32,
    pub envelope_clock: u32,
    pub sample_length_arr: [u8; 512],
    pub channel_1_time_freq: u32, // shadow frequency for sweeping
    pub channel_1_handle_trigger: bool,
    pub channel_1_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_2_pos: u32, // cap the lowest possible frequency to 63hz => 700 samples (GB is 64hz)
    pub channel_2_handle_trigger: bool,
    pub channel_1: AudioChannel,
    pub channel_2: AudioChannel,
    pub audio_queue: sdl2::audio::AudioQueue<i16>,
    pub prev_size: i32,
    pub audio_vec_queue: Vec<i16>,
    pub audio_freq: u32,
}

impl Default for AudioChannel {
    fn default() -> AudioChannel {
        AudioChannel {
            counter: 0,
            enabled: false,
            envelope_pos: 0,
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
            envelope_clock: 0,
            sample_length_arr: sample_len_arr(),
            audio_queue: init_audio(44100),
            channel_1_time_freq: 0,
            channel_1_pos: 0,
            channel_1_handle_trigger: false,
            channel_2_pos: 0,
            channel_2_handle_trigger: false,
            channel_1: Default::default(),
            channel_2: Default::default(),
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
    if cpu.apu.channel_1.enabled && channel_1_length_enable && cpu.apu.channel_1.counter != 0 {
        cpu.apu.channel_1.counter -= 1;
        if cpu.apu.channel_1.counter == 0 {
            cpu.apu.channel_1.enabled = false;
        }
    }
    if cpu.apu.channel_2.enabled && channel_2_length_enable && cpu.apu.channel_2.counter != 0 {
        cpu.apu.channel_2.counter -= 1;
        if cpu.apu.channel_2.counter == 0 {
            cpu.apu.channel_2.enabled = false;
        }
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
    let init_volume = ((NR12 & 0xF0) >> 4) as i16;
    let sample_count = result.len();
    let volume = volume_step * init_volume;
    let high_len = get_duty(duty, period) as u32;
    // println!("");
    if period as u32 != 0 {
        for x in 0..sample_count {
            let wave_pos = (x as u32 + cpu.apu.channel_1_pos) % period as u32;
            result[x] += cond!(wave_pos <= high_len, volume, -volume);
            //   print!("{:?}", cond!(wave_pos <= high_len, 0, 1));
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
    let init_volume = ((NR22 & 0xF0) >> 4) as i16;
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

pub fn handle_triggers(cpu: &mut Cpu) {
    if cpu.apu.channel_2_handle_trigger {
        cpu.apu.channel_2.enabled = true;
        cpu.apu.channel_2.counter = 64;
        cpu.apu.channel_2.envelope_pos = 0;
        cpu.apu.channel_2_pos = 0;
        let nr22 = read_address(0xFF17, cpu);
        cpu.apu.channel_2.volume = (nr22 & 0xF0) >> 4;
        cpu.apu.channel_2_handle_trigger = false;
        cpu.apu.channel_2.enabled = nr22 & 0xF8 != 0;
    }
    if cpu.apu.channel_1_handle_trigger {
        cpu.apu.channel_1.enabled = true;
        cpu.apu.channel_1.counter = 64;
        cpu.apu.channel_1.envelope_pos = 0;
        cpu.apu.channel_1_pos = 0;
        let nr12 = read_address(0xFF12, cpu);
        cpu.apu.channel_1.volume = (nr12 & 0xF0) >> 4;
        cpu.apu.channel_1_handle_trigger = false;
        cpu.apu.channel_1.enabled = nr12 & 0xF8 != 0;
    }
}
pub fn step(cpu: &mut Cpu) {
    cpu.apu.master_clock = (cpu.apu.master_clock + 1) % 512;
    let sample_len = cpu.apu.sample_length_arr[cpu.apu.master_clock as usize];
    handle_triggers(cpu);
    if cpu.apu.master_clock % 2 == 0 {
        step_length(cpu);
    }
    if cpu.apu.master_clock % 4 == 0 {
        cpu.apu.sweep_clock = (cpu.apu.sweep_clock + 1) % 512;
    }
    if cpu.apu.master_clock % 8 == 7 {
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
