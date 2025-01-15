use eframe::egui;
use std::path::PathBuf;
use std::error::Error;
use std::sync::mpsc;
use crate::state::ProcessingState;
use crate::audio::{processor, player::AudioPlayer};
use crate::ui::panels;

pub struct NightcoreApp {
    pub processing_state: ProcessingState,
    pub audio_player: AudioPlayer,
    pub audio_path: Option<PathBuf>,
    pub tempo: f32,
    pub pitch: f32,
    pub processed_samples: Option<(Vec<f32>, usize, u32)>,
    pub processing_receiver: Option<mpsc::Receiver<Result<(Vec<f32>, usize, u32), Box<dyn Error + Send>>>>,
}

impl NightcoreApp {
    pub fn new() -> Self {
        Self {
            processing_state: ProcessingState::new(),
            audio_player: AudioPlayer::new(),
            audio_path: None,
            tempo: 1.0,
            pitch: 0.0,
            processed_samples: None,
            processing_receiver: None,
        }
    }

    pub fn process_button_click(&mut self, ctx: &egui::Context, path: String) {
        self.stop_preview();
        
        let preview_samples = self.processing_state.preview_samples.clone();
        let sample_rate = self.processing_state.sample_rate.clone();
        let is_processing = self.processing_state.is_processing.clone();
        let path_clone = PathBuf::from(path);
        let tempo = self.tempo;
        let pitch = self.pitch;

        let ctx = ctx.clone();
        
        let (tx, rx) = mpsc::channel();
        self.processing_receiver = Some(rx);
        
        std::thread::spawn(move || {
            println!("Starting processing...");
            *is_processing.lock().unwrap() = true;

            let result = processor::process_audio_static(
                path_clone.as_path(),
                tempo,
                pitch,
                preview_samples,
                sample_rate
            );

            *is_processing.lock().unwrap() = false;
            
            let _ = tx.send(result);
            
            ctx.request_repaint();
        });
    }

    pub fn play_preview(&mut self) -> Result<(), Box<dyn Error>> {
        let samples = self.processing_state.preview_samples.lock().unwrap().clone();
        let sample_rate = *self.processing_state.sample_rate.lock().unwrap();
        
        self.audio_player.play(samples, sample_rate)?;
        *self.processing_state.is_playing.lock().unwrap() = true;
        
        Ok(())
    }

    pub fn stop_preview(&mut self) {
        self.audio_player.stop();
        *self.processing_state.is_playing.lock().unwrap() = false;
    }
}

impl Default for NightcoreApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for NightcoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        panels::render_side_panel(self, ctx);
        panels::render_central_panel(self, ctx);

        if let Some(rx) = &self.processing_receiver {
            if let Ok(Ok(processed)) = rx.try_recv() {
                self.processed_samples = Some(processed);
                self.processing_receiver = None;
            }
        }
    }
}
