use crate::dictionary::{fix_transcription_with_dictionary, get_cc_rules_path, Dictionary};
use crate::engine::{
    engine::ParakeetEngine, engine::ParakeetModelParams, transcription_engine::TranscriptionEngine,
};
use crate::model::Model;
use crate::config::ServerConfig;
use anyhow::{Context, Result};
use hound;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

static ENGINE: Lazy<parking_lot::Mutex<Option<ParakeetEngine>>> =
    Lazy::new(|| parking_lot::Mutex::new(None));

pub fn read_wav_samples(wav_path: &std::path::Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();

    if spec.bits_per_sample != 16 {
        return Err(anyhow::anyhow!(
            "Expected 16 bits per sample, found {}",
            spec.bits_per_sample
        ));
    }

    if spec.sample_format != hound::SampleFormat::Int {
        return Err(anyhow::anyhow!(
            "Expected Int sample format, found {:?}",
            spec.sample_format
        ));
    }

    let raw_i16: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
    let mut raw_i16 = raw_i16?;

    if spec.channels > 1 {
        let ch = spec.channels as usize;
        let mut mono: Vec<i16> = Vec::with_capacity(raw_i16.len() / ch);
        for frame in raw_i16.chunks_exact(ch) {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            let avg = (sum / ch as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            mono.push(avg);
        }
        raw_i16 = mono;
    }

    let samples_f32: Vec<f32> = raw_i16
        .into_iter()
        .map(|s| s as f32 / i16::MAX as f32)
        .collect();

    let out = if spec.sample_rate != 16000 {
        resample_linear(&samples_f32, spec.sample_rate as usize, 16000)
    } else {
        samples_f32
    };

    Ok(out)
}

pub fn preload_engine(model: &Model) -> Result<()> {
    let mut engine = ENGINE.lock();

    if engine.is_none() {
        let model_path = model
            .get_model_path()
            .map_err(|e| anyhow::anyhow!("Failed to get model path: {}", e))?;

        let mut new_engine = ParakeetEngine::new();
        new_engine
            .load_model_with_params(&model_path, ParakeetModelParams::int8())
            .map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

        *engine = Some(new_engine);
        println!("Model loaded and cached in memory");
    }

    Ok(())
}

pub fn transcribe_audio(
    audio_path: &std::path::Path,
    model: &Model,
    dictionary: Option<&Dictionary>,
    config: &ServerConfig,
) -> Result<String> {
    let samples = read_wav_samples(audio_path)?;

    let mut engine = ENGINE.lock();
    let engine = engine
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Engine not loaded"))?;

    let result = engine
        .transcribe_samples(samples, None)
        .map_err(|e| anyhow::anyhow!("Transcription failed: {}", e))?;

    let raw_text = result.text;

    // Apply dictionary corrections if available
    let text = if let Some(dict) = dictionary {
        match get_cc_rules_path(config) {
            Ok(cc_rules_path) => {
                let dict_words = dict.get();
                fix_transcription_with_dictionary(raw_text, dict_words, cc_rules_path)
            }
            Err(_) => {
                eprintln!("Warning: CC rules not found, skipping dictionary correction");
                raw_text
            }
        }
    } else {
        raw_text
    };

    Ok(text)
}

fn resample_linear(input: &[f32], src_hz: usize, dst_hz: usize) -> Vec<f32> {
    if input.is_empty() || src_hz == 0 || dst_hz == 0 {
        return Vec::new();
    }
    if src_hz == dst_hz {
        return input.to_vec();
    }
    let ratio = dst_hz as f64 / src_hz as f64;
    let out_len = ((input.len() as f64) * ratio).ceil() as usize;
    if out_len == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(out_len);
    let last_idx = input.len().saturating_sub(1);
    for i in 0..out_len {
        let t = (i as f64) / ratio;
        let idx = t.floor() as usize;
        let frac = (t - idx as f64) as f32;
        let a = input[idx];
        let b = input[std::cmp::min(idx + 1, last_idx)];
        out.push(a + (b - a) * frac);
    }
    out
}
