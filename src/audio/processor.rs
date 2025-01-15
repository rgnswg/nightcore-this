use std::error::Error;
use std::sync::{Arc, Mutex};
use std::path::Path;
use super::decoder;

pub fn process_audio_static(
    input_path: &Path,
    tempo: f32,
    pitch: f32,
    preview_samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
) -> Result<(Vec<f32>, usize, u32), Box<dyn Error + Send>> {
    let start_time = std::time::Instant::now();

    let (samples, channels, decoded_rate) = decoder::decode_audio(input_path)?;
    println!("Audio loaded: {} channels, {} samples, {} Hz", 
            channels, samples.len(), decoded_rate);

    *sample_rate.lock().unwrap() = decoded_rate;
    
    if (tempo - 1.0).abs() < 0.001 && pitch.abs() < 0.001 {
        println!("No changes requested, saving original audio");
        let samples_for_preview = samples.clone();
        *preview_samples.lock().unwrap() = samples_for_preview;
        return Ok((samples, channels, decoded_rate));
    }

    println!("\nDEBUG VALUES:");
    println!("Input tempo: {}", tempo);
    println!("Input pitch: {}", pitch);

    let samples_per_channel = samples.len() / channels;
    let mut channels_data: Vec<Vec<f32>> = vec![Vec::with_capacity(samples_per_channel); channels];
    
    for (i, sample) in samples.iter().enumerate() {
        channels_data[i % channels].push(*sample);
    }

    let pitch_ratio = 2.0f32.powf(pitch / 12.0);
    let tempo_ratio = tempo;
    let final_ratio = tempo_ratio * pitch_ratio;

    println!("\nRATIOS:");
    println!("Pitch ratio: {}", pitch_ratio);
    println!("Tempo ratio: {}", tempo_ratio);
    println!("Final ratio: {}", final_ratio);

    let new_len = (samples_per_channel as f32 / final_ratio) as usize;
    let mut processed_channels: Vec<Vec<f32>> = vec![Vec::with_capacity(new_len); channels];

    for (channel_idx, channel) in channels_data.iter().enumerate() {
        let mut processed = Vec::with_capacity(new_len);
        
        for i in 0..new_len {
            let src_pos = i as f32 * final_ratio;
            let src_idx = src_pos as usize;
            let frac = src_pos - src_idx as f32;
            
            let sample = if src_idx + 1 < channel.len() {
                channel[src_idx] * (1.0 - frac) + channel[src_idx + 1] * frac
            } else if src_idx < channel.len() {
                channel[src_idx]
            } else {
                0.0
            };
            
            processed.push(sample);
        }
        
        processed_channels[channel_idx] = processed;
    }

    let max_amplitude = processed_channels.iter()
        .flat_map(|channel| channel.iter())
        .fold(0.0f32, |max, &sample| max.max(sample.abs()));

    if max_amplitude > 1.0 {
        let scale = 0.95 / max_amplitude;
        for channel in &mut processed_channels {
            for sample in channel.iter_mut() {
                *sample *= scale;
            }
        }
    }

    let mut final_samples = Vec::with_capacity(new_len * channels);
    for i in 0..new_len {
        for c in 0..channels {
            if i < processed_channels[c].len() {
                final_samples.push(processed_channels[c][i]);
            }
        }
    }

    let samples_for_preview = final_samples.clone();
    *preview_samples.lock().unwrap() = samples_for_preview;
    
    println!("Total processing time: {:.2}s", start_time.elapsed().as_secs_f32());
    Ok((final_samples, channels, decoded_rate))
}
