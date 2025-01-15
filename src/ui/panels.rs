use eframe::egui;
use rfd::FileDialog;
use crate::app::NightcoreApp;

pub fn render_side_panel(app: &mut NightcoreApp, ctx: &egui::Context) {
    egui::SidePanel::left("file_panel").min_width(200.0).max_width(200.0).show(ctx, |ui| {
        ui.add_space(50.0);
        
        ui.vertical_centered(|ui| {
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(180.0, 180.0),
                egui::Sense::click_and_drag()
            );

            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY)
            );

            if response.hovered() {
                ui.ctx().output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }

            let text = if app.audio_path.is_none() {
                "Drop an audio file\nor click here".to_string()
            } else {
                let filename = app.audio_path
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy();
                
                let truncated_name = if filename.len() > 20 {
                    format!("{}...", &filename[..17])
                } else {
                    filename.to_string()
                };
                
                format!("File:\n{}", truncated_name)
            };

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &text,
                egui::TextStyle::Button.resolve(ui.style()),
                egui::Color32::LIGHT_GRAY,
            );

            if let Some(file) = ctx.input(|i| {
                i.raw.dropped_files
                    .first()
                    .and_then(|f| f.path.clone())
            }) {
                if let Some(ext) = file.extension() {
                    let ext = ext.to_str().unwrap_or("").to_lowercase();
                    if ext == "mp3" || ext == "wav" || ext == "flac" {
                        app.audio_path = Some(file);
                    }
                }
            }

            if response.clicked() {
                let file_dialog = FileDialog::new()
                    .add_filter("Audio", &["mp3", "wav", "flac"])
                    .set_title("Select audio file");
                    
                if let Some(path) = file_dialog.pick_file() {
                    app.audio_path = Some(path);
                }
            }

            ui.add_space(10.0);
            
            if app.audio_path.is_some() {
                let button_response = ui.add_sized(
                    [120.0, 30.0],
                    egui::Button::new(
                        egui::RichText::new("Clear selection")
                            .size(14.0)
                            .color(egui::Color32::WHITE)
                    )
                );
                if button_response.clicked() {
                    app.audio_path = None;
                }
            }
        });
    });
}

pub fn render_central_panel(app: &mut NightcoreApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Nightcore This");
        ui.add_space(20.0);
        
        ui.add_enabled_ui(!*app.processing_state.is_processing.lock().unwrap(), |ui| {
            ui.add(
                egui::Slider::new(&mut app.tempo, 0.5..=2.0)
                    .text("Tempo")
                    .suffix("x")
                    .custom_formatter(|n, _| format!("{:.2}", n))
                    .smart_aim(true)
            );

            ui.add(
                egui::Slider::new(&mut app.pitch, -12.0..=12.0)
                    .text("Pitch")
                    .suffix(" st")
                    .smart_aim(true)
            );
        });
        
        ui.add_space(20.0);

        let is_processing = *app.processing_state.is_processing.lock().unwrap();
        let button_text = if is_processing { "Processing..." } else { "Process" };

        let process_button = ui.add_enabled(
            !is_processing && app.audio_path.is_some(),
            egui::Button::new(button_text)
        );

        if process_button.clicked() {
            if let Some(path) = app.audio_path.as_ref() {
                app.process_button_click(ctx, path.to_string_lossy().to_string());
            }
        }

        if is_processing {
            ui.add_space(10.0);
            ui.spinner();
        }

        if !is_processing {
            let is_playing = *app.processing_state.is_playing.lock().unwrap();
            
            if !app.processing_state.preview_samples.lock().unwrap().is_empty() {
                let button_text = if is_playing { "Stop" } else { "Play Preview" };
                if ui.button(button_text).clicked() {
                    if is_playing {
                        app.stop_preview();
                    } else {
                        if let Err(e) = app.play_preview() {
                            eprintln!("Error al reproducir: {}", e);
                        }
                    }
                }

                ui.add_space(10.0);
                if ui.button("Export").clicked() {
                    let samples = app.processing_state.preview_samples.lock().unwrap().clone();
                    let sample_rate = *app.processing_state.sample_rate.lock().unwrap();
                    
                    let file_dialog = FileDialog::new()
                        .add_filter("WAV", &["wav"])
                        .set_file_name("nightcore_output.wav")
                        .set_title("Save processed audio");
                        
                    if let Some(path) = file_dialog.save_file() {
                        if let Err(e) = crate::audio::decoder::save_processed_audio(
                            &samples, 
                            2,  // stereo
                            sample_rate, 
                            &path
                        ) {
                            eprintln!("Error saving audio: {}", e);
                        }
                    }
                }
            }
        }
    });
}
