//! Example Rust client for Murmure gRPC Server
//!
//! This client records audio from your microphone and sends it to the Murmure server
//! for transcription.
//!
//! ## Usage
//!
//! First, ensure the server is running:
//! ```bash
//! cd ../src-tauri
//! cargo run --bin murmure-server
//! ```
//!
//! Then run this client:
//! ```bash
//! cd examples
//! cargo run --example rust_record_client -- --duration 5
//! ```
//!
//! Options:
//! - `--server <address>` - Server address (default: http://localhost:50051)
//! - `--duration <seconds>` - Recording duration (default: 5)
//!
//! See ../docs/examples/README_RUST_CLIENT.md for detailed documentation.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavReader, WavSpec, WavWriter};
use murmure_core::tts::{SynthesisService, TtsConfig, TtsModel};
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::time::Duration;
use tonic::Request;

// Include generated proto code from build script
pub mod murmure {
    include!(concat!(env!("OUT_DIR"), "/murmure.rs"));
}

use murmure::transcription_service_client::TranscriptionServiceClient;
use murmure::TranscribeFileRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let server_address = args
        .iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .unwrap_or(&"http://localhost:50051".to_string())
        .clone();

    let duration_secs = args
        .iter()
        .position(|a| a == "--duration")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);

    println!("🎤 Murmure Audio Recording Client");
    println!("Server: {}", server_address);
    println!("Recording duration: {} seconds", duration_secs);
    println!("Press Ctrl+C to stop early\n");

    // Record audio
    println!("🎙️  Recording audio...");
    let audio_data = record_audio(duration_secs)?;
    println!("✅ Recording complete ({} bytes)", audio_data.len());

    // Connect to server
    println!("📡 Connecting to server...");
    let mut client = TranscriptionServiceClient::connect(server_address).await?;
    println!("✅ Connected to server");

    // Transcribe
    println!("🔊 Sending audio for transcription...");
    let request = Request::new(TranscribeFileRequest {
        audio_data,
        use_dictionary: true,
    });

    let response = client.transcribe_file(request).await?;
    let transcription = response.into_inner();

    if transcription.success {
        println!("\n📝 Transcription:");
        if transcription.text.is_empty() {
            println!("(Empty transcription - audio may be too short, silent, or unrecognized)");
            println!("\n💡 Possible reasons:");
            println!("   - Audio was too quiet or silent");
            println!("   - Audio format mismatch");
            println!("   - Server processed but found no speech");
            println!("   - Try speaking louder or checking microphone levels");
        } else {
            println!("{}", transcription.text);
            
            // Synthesize and play using TTS
            println!("\n🔊 Synthesizing speech...");
            if let Err(e) = synthesize_and_play(&transcription.text).await {
                eprintln!("⚠️  TTS error: {} (continuing anyway)", e);
            }
        }
    } else {
        eprintln!("\n❌ Transcription failed: {}", transcription.error);
        if transcription.error.is_empty() {
            eprintln!("   (No error message provided by server)");
        }
    }

    Ok(())
}

async fn synthesize_and_play(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TTS service
    let tts_config = TtsConfig::from_env().unwrap_or_default();
    let tts_model = Arc::new(TtsModel::new(tts_config.clone()));
    let tts_service = SynthesisService::new(tts_model, Arc::new(tts_config))
        .map_err(|e| format!("Failed to initialize TTS: {}", e))?;

    // Synthesize text to audio
    let wav_bytes = tts_service
        .synthesize_text(text)
        .map_err(|e| format!("Synthesis failed: {}", e))?;

    println!("✅ Synthesis complete ({} bytes)", wav_bytes.len());

    // Play the audio
    println!("🔊 Playing audio...");
    play_wav_bytes(&wav_bytes)?;
    println!("✅ Playback complete");

    Ok(())
}

fn play_wav_bytes(wav_bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Cursor;

    // Read WAV file from bytes
    let cursor = Cursor::new(wav_bytes);
    let mut reader = WavReader::new(cursor)?;
    let spec = reader.spec();

    // Convert samples to f32
    let samples: Result<Vec<f32>, hound::Error> = reader
        .samples::<i16>()
        .map(|s| {
            s.map(|sample| sample as f32 / i16::MAX as f32)
        })
        .collect();
    let samples = samples.map_err(|e| format!("Failed to read WAV samples: {}", e))?;

    if samples.is_empty() {
        return Err("No audio samples to play".into());
    }

    // Get default output device
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("No default output device available")?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    println!("   Using output device: {}", device_name);

    // Create output config matching WAV file
    let config = cpal::StreamConfig {
        channels: spec.channels as u16,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Use a channel to feed samples to the stream
    let (tx, rx) = std::sync::mpsc::channel();
    let samples_len = samples.len();
    
    // Send samples in chunks
    std::thread::spawn(move || {
        for chunk in samples.chunks(1024) {
            let chunk_vec = chunk.to_vec();
            if tx.send(chunk_vec).is_err() {
                break;
            }
        }
    });

    // Create output stream
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Try to get samples from channel, otherwise fill with zeros
            if let Ok(chunk) = rx.try_recv() {
                let len = data.len().min(chunk.len());
                data[..len].copy_from_slice(&chunk[..len]);
                if len < data.len() {
                    data[len..].fill(0.0);
                }
            } else {
                data.fill(0.0);
            }
        },
        |err| eprintln!("Playback error: {}", err),
        None,
    )?;

    stream.play()?;

    // Wait for playback to complete
    let duration = samples_len as f64 / spec.sample_rate as f64;
    std::thread::sleep(Duration::from_secs_f64(duration + 0.1));

    Ok(())
}

fn record_audio(duration_secs: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    // List all available input devices for debugging
    let input_devices: Vec<_> = host.input_devices()?.collect();
    if input_devices.is_empty() {
        return Err("❌ No input devices found. Please check microphone permissions.".into());
    }

    println!("   Available input devices:");
    for (i, dev) in input_devices.iter().enumerate() {
        if let Ok(name) = dev.name() {
            println!("     {}. {}", i + 1, name);
        }
    }

    let device = host
        .default_input_device()
        .ok_or("❌ No default input device available. Check microphone permissions in System Settings > Privacy & Security > Microphone")?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    println!("\n   Using device: {}", device_name);

    let config = match device.default_input_config() {
        Ok(config) => config,
        Err(e) => {
            return Err(format!(
                "❌ Failed to get input config: {}\n   This usually means microphone access is denied. Please grant microphone permission to Terminal/iTerm/Cursor in System Settings > Privacy & Security > Microphone",
                e
            ).into());
        }
    };

    println!("   Sample rate: {} Hz", config.sample_rate().0);
    println!("   Channels: {}", config.channels());

    // Create temporary WAV file
    let temp_file = std::env::temp_dir().join(format!("murmure-record-{}.wav", std::process::id()));

    let spec = WavSpec {
        channels: 1, // Force mono
        sample_rate: config.sample_rate().0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let file = File::create(&temp_file)?;
    let writer = WavWriter::new(BufWriter::new(file), spec)?;
    let writer_arc = Arc::new(std::sync::Mutex::new(writer));

    println!("   Testing microphone access...");
    let result = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, writer_arc.clone()),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, writer_arc.clone()),
        cpal::SampleFormat::I32 => build_stream::<i32>(&device, &config, writer_arc.clone()),
        _ => return Err("Unsupported sample format".into()),
    };

    let (stream, audio_stats) = match result {
        Ok((s, stats)) => (s, stats),
        Err(e) => {
            return Err(format!(
                "❌ Failed to create audio stream: {}\n   This usually means:\n   1. Microphone permission denied - Check System Settings > Privacy & Security > Microphone\n   2. Microphone is in use by another app\n   3. Microphone hardware issue",
                e
            ).into());
        }
    };

    println!("   ✅ Microphone stream created (this doesn't guarantee permission)");

    if let Err(e) = stream.play() {
        return Err(format!(
            "❌ Failed to start recording stream: {}\n   Check microphone permissions and try again.",
            e
        ).into());
    }

    println!("   Recording for {} seconds...", duration_secs);
    println!("   💡 Speak now! If audio levels stay at 0, microphone permission is likely denied.");

    // Record and monitor audio levels in real-time
    let check_interval = std::time::Duration::from_millis(500);
    let start = std::time::Instant::now();
    let mut last_amplitude: i16 = 0;
    let mut warning_printed = false;

    loop {
        std::thread::sleep(std::cmp::min(
            check_interval,
            Duration::from_secs(duration_secs).saturating_sub(start.elapsed()),
        ));

        let stats = audio_stats.lock().unwrap();
        let current_amplitude = stats.1;
        let elapsed = start.elapsed();

        if current_amplitude != last_amplitude {
            println!(
                "   📊 Audio level: {} ({} samples processed)",
                current_amplitude, stats.0
            );
            last_amplitude = current_amplitude;
        }

        if elapsed >= Duration::from_secs(duration_secs) {
            break;
        }

        // Check early if completely silent after 2 seconds (only warn once)
        if !warning_printed && elapsed >= Duration::from_secs(2) && stats.1 == 0 && stats.0 > 0 {
            println!("\n   ⚠️  WARNING: No audio detected after 2 seconds!");
            println!("   This likely means microphone permission is denied on macOS.");
            println!("\n   📋 To fix this:");
            println!("   1. Open System Settings → Privacy & Security → Microphone");
            println!("   2. Find your terminal app (Terminal, iTerm, Cursor, etc.)");
            println!("   3. Enable microphone access");
            println!("   4. Restart your terminal app");
            println!("\n   Or run: open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone'");
            println!("   (Recording will continue, but will be silent without permission)\n");
            warning_printed = true;
        }
    }

    let final_stats = audio_stats.lock().unwrap();
    println!("\n   Recording complete.");
    println!(
        "   Final stats: {} samples, max amplitude: {}",
        final_stats.0, final_stats.1
    );

    drop(stream);

    // Finalize WAV file
    {
        let mut writer = writer_arc.lock().unwrap();
        writer.flush()?;
        drop(writer); // Explicitly drop to get ownership back
    }

    // Get the writer back and finalize
    let writer = Arc::try_unwrap(writer_arc).map_err(|_| "Failed to unwrap Arc")?;
    writer.into_inner().unwrap().finalize()?;

    // Read WAV file into memory
    let audio_data = std::fs::read(&temp_file)?;

    // Debug: Check if audio has non-zero samples
    let reader = WavReader::open(&temp_file)?;
    let samples: Result<Vec<i16>, _> = reader.into_samples().collect();
    if let Ok(samples) = samples {
        let non_zero_count = samples.iter().filter(|&&s| s != 0).count();
        let total_count = samples.len();
        let max_amplitude = samples.iter().map(|&s| s.abs()).max().unwrap_or(0);

        println!("   Audio analysis:");
        println!("   - Total samples: {}", total_count);
        println!(
            "   - Non-zero samples: {} ({:.1}%)",
            non_zero_count,
            if total_count > 0 {
                (non_zero_count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("   - Max amplitude: {} / {}", max_amplitude, i16::MAX);

        if non_zero_count == 0 {
            println!("   ❌ ERROR: Audio appears to be completely silent!");
            println!("   This indicates the microphone is not capturing audio.");
            println!("   Possible causes:");
            println!("   1. Microphone permission denied");
            println!("   2. Microphone is muted or volume is zero");
            println!("   3. Wrong microphone selected");
            println!("   4. Microphone hardware disconnected");
            return Err(
                "Audio recording is silent - microphone may not have access or is muted".into(),
            );
        } else if max_amplitude < 100 {
            println!("   ⚠️  WARNING: Audio is very quiet (max amplitude < 100)");
            println!("   Try speaking louder or increasing microphone input volume.");
        } else {
            println!("   ✅ Audio levels look good!");
        }
    }

    // Optionally keep the file for debugging (comment out cleanup)
    // Uncomment the next line to keep the file for inspection:
    // println!("   Debug: WAV file saved at: {}", temp_file.display());

    // Clean up
    let _ = std::fs::remove_file(&temp_file);

    Ok(audio_data)
}

type WavWriterType = WavWriter<BufWriter<File>>;
type StreamResult =
    Result<(cpal::Stream, Arc<std::sync::Mutex<(usize, i16)>>), Box<dyn std::error::Error>>;

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::SupportedStreamConfig,
    writer: Arc<std::sync::Mutex<WavWriterType>>,
) -> StreamResult
where
    T: cpal::Sample + cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels() as usize;

    // Track audio levels in real-time
    let audio_stats = Arc::new(std::sync::Mutex::new((0usize, 0i16))); // (sample_count, max_amplitude)
    let stats_clone = audio_stats.clone();

    let stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut writer = writer.lock().unwrap();
            let mut stats = stats_clone.lock().unwrap();
            stats.0 += data.len() / channels;

            for frame in data.chunks_exact(channels) {
                let sample = if channels == 1 {
                    frame[0].to_sample::<f32>()
                } else {
                    // Convert to mono by averaging
                    frame.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                };

                // Convert to i16 and write
                let sample_i16 =
                    (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let amplitude = sample_i16.abs();
                if amplitude > stats.1 {
                    stats.1 = amplitude;
                }
                let _ = writer.write_sample(sample_i16);
            }
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    Ok((stream, audio_stats))
}
