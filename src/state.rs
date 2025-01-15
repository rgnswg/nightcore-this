use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ProcessingState {
    pub preview_samples: Arc<Mutex<Vec<f32>>>,
    pub sample_rate: Arc<Mutex<u32>>,
    pub is_processing: Arc<Mutex<bool>>,
    pub is_playing: Arc<Mutex<bool>>,
}

impl ProcessingState {
    pub fn new() -> Self {
        Self {
            preview_samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(48000)),
            is_processing: Arc::new(Mutex::new(false)),
            is_playing: Arc::new(Mutex::new(false)),
        }
    }
}
