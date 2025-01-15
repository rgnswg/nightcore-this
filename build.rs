use std::env;
use std::path::Path;
use std::fs;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let ffmpeg_dir = Path::new(&out_dir).join("ffmpeg");
        fs::create_dir_all(&ffmpeg_dir).unwrap();
        let ffmpeg_src = Path::new("resources/windows/bin/ffmpeg.exe");
        let ffmpeg_dst = ffmpeg_dir.join("ffmpeg.exe");
        fs::copy(ffmpeg_src, ffmpeg_dst).unwrap();
        println!("cargo:rustc-env=FFMPEG_PATH={}", ffmpeg_dir.display());
    }
}
