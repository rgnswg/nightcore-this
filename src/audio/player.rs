use std::error::Error;
use rodio::{OutputStream, Sink};

pub struct AudioPlayer {
    pub active_sink: Option<Sink>,
    pub output_stream: Option<OutputStream>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            active_sink: None,
            output_stream: None,
        }
    }

    pub fn play(&mut self, samples: Vec<f32>, sample_rate: u32) -> Result<(), Box<dyn Error>> {
        self.stop();

        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        
        let source = rodio::buffer::SamplesBuffer::new(
            2,
            sample_rate,
            samples
        );
        
        sink.append(source);
        sink.play();
        
        self.active_sink = Some(sink);
        self.output_stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.active_sink {
            sink.stop();
        }
        self.active_sink = None;
        self.output_stream = None;
    }
}
