use std::{f32::consts::PI, fs::File, env};
use hound::{WavWriter, WavSpec, SampleFormat};
use realfft::{num_complex::Complex32, RealFftPlanner};
use std::io::BufWriter;

// fourier series expressions of basic waveforms..

fn saw(i: i32) -> Complex32 {
    Complex32::new(0., (-1f32).powi(i as i32) / (PI * i as f32))
}

fn sine(i: i32) -> Complex32 {
    assert_eq!(i, 1);
    Complex32::new(0., -0.5)
}

fn square(i: i32) -> Complex32 {
    Complex32::new(0., -2. / (PI * i as f32))
}

fn triangle(i: i32) -> Complex32 {
    Complex32::new(0., -4. / (PI * PI) * (-1f32).powi((i as i32 - 1) / 2) / (i * i) as f32)
}

fn main() {

    let path = env::args().nth(1).expect("please specifiy a file path");

    let mut fft = RealFftPlanner::new();
    let c2r = fft.plan_fft_inverse(2048);

    let mut input = c2r.make_input_vec();
    let mut output = c2r.make_output_vec();
    let mut scratch = c2r.make_scratch_vec();

    let file = BufWriter::with_capacity(
        2048 * 256 * 4 + 128,
        File::create(
            path
        ).unwrap()
    );

    let mut writer = WavWriter::new(
        file,
        WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float
        }
    ).unwrap();

    let frames_per_waveform = 256 / 4;
    let num_bins = 2048 as i32 / 2;

    for (range, step, func) in [
        // all harmonics
        (1..num_bins, 1, saw as fn(i32) -> Complex32),
        (1..2, 1, sine),
        // odd harmonics
        (1..num_bins, 2, square),
        (1..num_bins, 2, triangle)
    ] {

        input.fill(Complex32::new(0., 0.));
        for (i, bin) in range.zip(&mut input[1..]).step_by(step) {
            *bin = func(i);
        }

        c2r.process_with_scratch(&mut input, &mut output, &mut scratch).expect("wrong fft buffer lengths");

        for _ in 0..frames_per_waveform {
            for &sample in output.iter() {
                writer.write_sample(sample).unwrap();
            }
        }
    }
}