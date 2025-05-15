use std::path::Path;
use std::process::Command;
use std::error::Error;
use std::io::{Read, Write};
use std::env;
use std::path::PathBuf;

fn get_ffmpeg_path() -> PathBuf {
     #[cfg(windows)]
    {
        if let Ok(dir) = env::var("FFMPEG_PATH") {
            PathBuf::from(dir).join("ffmpeg.exe")
        } else {
            PathBuf::from("ffmpeg.exe")
        }
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("ffmpeg")
    }
}

pub fn decode_audio(path: &Path) -> Result<(Vec<f32>, usize, u32), Box<dyn Error + Send>> {
    println!("Decoding audio with FFmpeg...");
    let start = std::time::Instant::now();

    let ffmpeg_path = get_ffmpeg_path();
    
    let mut child = Command::new(ffmpeg_path)
        .args([
            "-i", path.to_str().unwrap(),
            "-vn",
            "-ar", "48000",
            "-ac", "2",
            "-f", "f32le",
            "-acodec", "pcm_f32le",
            "-loglevel", "info",
            "pipe:1"
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    let mut raw_samples = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut raw_samples)
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    let status = child.wait()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
        
    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "FFmpeg failed to decode audio"
        )) as Box<dyn Error + Send>);
    }

    if raw_samples.len() % 8 != 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid number of samples"
        )) as Box<dyn Error + Send>);
    }

    let samples: Vec<f32> = raw_samples.chunks_exact(4)
        .map(|chunk| {
            let arr = [chunk[0], chunk[1], chunk[2], chunk[3]];
            f32::from_le_bytes(arr)
        })
        .collect();

    if samples.len() % 2 != 0 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid number of samples for stereo"
        )) as Box<dyn Error + Send>);
    }

    println!("FFmpeg decode completed in {:.2}s", start.elapsed().as_secs_f32());
    println!("Sample count: {}", samples.len());
    println!("Duration: {:.2}s", samples.len() as f32 / (48000.0 * 2.0));
    
    Ok((samples, 2, 48000))
}

pub fn save_processed_audio(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
    output_path: &Path
) -> Result<(), Box<dyn Error + Send>> {
    let mut command = Command::new("ffmpeg");
    command
        .arg("-y")
        .arg("-f").arg("f32le")
        .arg("-ar").arg(sample_rate.to_string())
        .arg("-ac").arg(channels.to_string())
        .arg("-i").arg("-")
        .arg(output_path);

    let mut child = command.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    if let Some(mut stdin) = child.stdin.take() {
        let bytes: Vec<u8> = samples.iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();
        stdin.write_all(&bytes)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
    }

    let status = child.wait()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "FFmpeg failed to encode audio"
        )) as Box<dyn Error + Send>);
    }

    Ok(())
}
