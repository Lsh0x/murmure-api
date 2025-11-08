use anyhow::Result;
use hound::{WavSpec, WavWriter};
use std::io::Cursor;

pub fn write_wav_bytes(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::new(cursor, spec)?;

    for &sample in samples {
        // Clamp and convert f32 [-1.0, 1.0] to i16
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(int_sample)?;
    }

    writer.finalize()?;
    Ok(buffer)
}
