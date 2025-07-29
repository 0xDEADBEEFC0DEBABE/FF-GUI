use eframe::egui;
use crate::app_state::*;
use crate::language::*;
use crate::codec_manager::*;
use crate::preset_manager::*;

pub struct OperationSelector;

impl OperationSelector {
    pub fn show(ui: &mut egui::Ui, current_operation: &mut Option<OperationType>, translations: &Translations) -> bool {
        let mut operation_changed = false;
        
        ui.group(|ui| {
            ui.set_min_width(200.0);
            ui.vertical(|ui| {
                ui.heading(translations.select_operation_type());
                ui.separator();

                let categories = [
                    (translations.video_processing(), vec![
                        OperationType::VideoConvert,
                        OperationType::VideoCompress,
                        OperationType::VideoResize,
                        OperationType::VideoCrop,
                        OperationType::VideoRotate,
                        OperationType::VideoFilter,
                    ]),
                    (translations.audio_processing(), vec![
                        OperationType::AudioConvert,
                        OperationType::AudioCompress,
                        OperationType::AudioResample,
                        OperationType::AudioVolume,
                        OperationType::AudioTrim,
                        OperationType::AudioMerge,
                    ]),
                    (translations.video_audio_operations(), vec![
                        OperationType::VideoAudioMerge,
                        OperationType::VideoAudioSplit,
                        OperationType::ExtractAudio,
                        OperationType::ExtractVideo,
                    ]),
                    (translations.batch_processing(), vec![
                        OperationType::BatchConvert,
                    ]),
                    (translations.advanced_features(), vec![
                        OperationType::AddSubtitle,
                        OperationType::AddWatermark,
                        OperationType::FrameExtract,
                        OperationType::VideoToGif,
                        OperationType::GifResize,
                    ]),
                ];

                for (category_name, operations) in categories {
                    ui.collapsing(category_name, |ui| {
                        for op in operations {
                            let is_selected = current_operation.as_ref() == Some(&op);
                            if ui.selectable_label(is_selected, op.display_name(translations)).clicked() {
                                *current_operation = Some(op);
                                operation_changed = true;
                            }
                        }
                    });
                }
            });
        });
        
        operation_changed
    }
}

pub struct FileSelector;

impl FileSelector {
    pub fn show<F>(ui: &mut egui::Ui, files: &mut Vec<String>, allow_multiple: bool, label: &str, translations: &Translations, on_files_changed: F) 
    where F: Fn() {
        ui.group(|ui| {
            ui.label(format!("📁 {}", label));
            
            // Create drag-and-drop area with same style as AddSubtitle
            let file_area_height = 60.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), file_area_height),
                egui::Sense::click()
            );
            
            // Handle drag and drop detection
            let is_being_dragged = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());
            let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
            
            // Draw background with same color scheme as AddSubtitle
            let bg_color = if can_accept_drop {
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
            } else if files.is_empty() {
                egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
            } else {
                egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
            };
            
            ui.painter().rect_filled(rect, 5.0, bg_color);
            ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
            
            // Handle click to open file dialog
            if response.clicked() {
                if allow_multiple {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        files.clear();
                        files.extend(paths.iter().map(|p| p.display().to_string()));
                        on_files_changed();
                    }
                } else {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        files.clear();
                        files.push(path.display().to_string());
                        on_files_changed();
                    }
                }
            }
            
            // Process dropped files
            if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                let dropped_paths: Vec<String> = dropped_files
                    .iter()
                    .filter_map(|f| f.path.as_ref())
                    .map(|p| p.display().to_string())
                    .collect();
                
                if !dropped_paths.is_empty() {
                    if allow_multiple {
                        // Only add files that aren't already in the list
                        for path in dropped_paths {
                            if !files.contains(&path) {
                                files.push(path);
                            }
                        }
                    } else {
                        files.clear();
                        if let Some(first_path) = dropped_paths.first() {
                            files.push(first_path.clone());
                        }
                    }
                    on_files_changed();
                }
            }
            
            // Draw text in the drop area
            let text = if files.is_empty() {
                if can_accept_drop {
                    if allow_multiple {
                        "📁 Drop files here".to_string()
                    } else {
                        "📁 Drop file here".to_string()
                    }
                } else {
                    if allow_multiple {
                        "📁 Click to select files or drag & drop here".to_string()
                    } else {
                        "📁 Click to select file or drag & drop here".to_string()
                    }
                }
            } else {
                // Show file count or single file name
                if allow_multiple && files.len() > 1 {
                    format!("📁 {} files selected", files.len())
                } else if let Some(file) = files.first() {
                    let file_name = std::path::Path::new(file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    format!("📁 {}", file_name)
                } else {
                    "📁 No files".to_string()
                }
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                if files.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
            );
            
            ui.add_space(5.0);
            
            // Show controls and status
            ui.horizontal(|ui| {
                if !files.is_empty() {
                    if ui.button("🗑️ Clear").clicked() {
                        files.clear();
                    }
                    if ui.button("📁 Change").clicked() {
                        if allow_multiple {
                            if let Some(paths) = rfd::FileDialog::new().pick_files() {
                                files.clear();
                                files.extend(paths.iter().map(|p| p.display().to_string()));
                                on_files_changed();
                            }
                        } else {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                files.clear();
                                files.push(path.display().to_string());
                                on_files_changed();
                            }
                        }
                    }
                }
            });
            
            // Show status
            if files.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                    if allow_multiple { "⚠ Please select input files" } else { "⚠ Please select input file" });
            } else {
                if allow_multiple {
                    let status_text = format!("✅ {} files ready", files.len());
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), status_text);
                } else {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ File ready");
                }
            }

            // Show file list in a collapsible section if multiple files
            if !files.is_empty() && (allow_multiple || files.len() > 1) {
                ui.add_space(5.0);
                ui.collapsing(format!("📋 {} Files", files.len()), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("file_list_scroll")
                        .max_height(120.0)
                        .show(ui, |ui| {
                            let mut to_remove = None;
                            for (i, file) in files.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("📄");
                                        ui.label(format!("{}.", i + 1));
                                        ui.label(std::path::Path::new(file)
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or(file));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button(translations.delete()).clicked() {
                                                to_remove = Some(i);
                                            }
                                        });
                                    });
                                });
                            }
                            if let Some(i) = to_remove {
                                files.remove(i);
                            }
                        });
                });
            }
        });
    }
}

pub struct OutputSelector;

impl OutputSelector {
    pub fn show(
        ui: &mut egui::Ui, 
        output_file: &mut String, 
        operation: &OperationType,
        video_settings: Option<&VideoSettings>,
        audio_settings: Option<&AudioSettings>,
        translations: &Translations
    ) {
        ui.group(|ui| {
            ui.label("💾 Output File:");
            
            // Create drag-and-drop area for output file
            let output_area_height = 60.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), output_area_height),
                egui::Sense::click()
            );
            
            // Handle drag and drop for output file path setting
            let is_being_dragged = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());
            let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
            
            // Draw background
            let bg_color = if can_accept_drop {
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
            } else if output_file.is_empty() {
                egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
            } else {
                egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
            };
            
            ui.painter().rect_filled(rect, 5.0, bg_color);
            ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
            
            // Handle click to open save dialog
            if response.clicked() {
                let (extension, filter_name, filter_extensions) = Self::get_extension_and_filter(
                    operation, 
                    video_settings, 
                    audio_settings,
                    translations
                );
                
                // Generate timestamped filename with milliseconds for uniqueness
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();
                
                let mut dialog = rfd::FileDialog::new()
                    .set_file_name(&format!("output_{}.{}", timestamp, extension));
                
                // Add file type filters
                if !filter_extensions.is_empty() {
                    dialog = dialog.add_filter(&filter_name, &filter_extensions);
                }
                dialog = dialog.add_filter(translations.all_files(), &["*"]);
                
                if let Some(path) = dialog.save_file() {
                    *output_file = path.display().to_string();
                }
            }
            
            // Process dropped files for output file path setting
            if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                if let Some(file) = dropped_files.first() {
                    if let Some(path) = &file.path {
                        // For output, use the directory and create a suggested filename
                        let parent_dir = if path.is_dir() {
                            path.clone()
                        } else {
                            path.parent().unwrap_or(path).to_path_buf()
                        };
                        
                        let (extension, _, _) = Self::get_extension_and_filter(
                            operation, 
                            video_settings, 
                            audio_settings,
                            translations
                        );
                        
                        // Generate timestamped filename with milliseconds for uniqueness
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis();
                        
                        let output_path = parent_dir.join(format!("output_{}.{}", timestamp, extension));
                        *output_file = output_path.display().to_string();
                    }
                }
            }
            
            // Draw text in the drop area
            let text = if output_file.is_empty() {
                if can_accept_drop {
                    "💾 Drop folder here to set output location"
                } else {
                    "💾 Click to select output location or drag folder here"
                }
            } else {
                // Show file name
                let file_name = std::path::Path::new(output_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                &format!("💾 {}", file_name)
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                if output_file.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
            );
            
            ui.add_space(5.0);
            
            // Show controls and status
            ui.horizontal(|ui| {
                if !output_file.is_empty() {
                    if ui.button("🗑️ Clear").clicked() {
                        output_file.clear();
                    }
                    if ui.button("📁 Change").clicked() {
                        let (extension, filter_name, filter_extensions) = Self::get_extension_and_filter(
                            operation, 
                            video_settings, 
                            audio_settings,
                            translations
                        );
                        
                        let mut dialog = rfd::FileDialog::new()
                            .set_file_name(&format!("output.{}", extension));
                        
                        if !filter_extensions.is_empty() {
                            dialog = dialog.add_filter(&filter_name, &filter_extensions);
                        }
                        dialog = dialog.add_filter(translations.all_files(), &["*"]);
                        
                        if let Some(path) = dialog.save_file() {
                            *output_file = path.display().to_string();
                        }
                    }
                    if ui.button("📋 Copy Path").clicked() {
                        ui.ctx().copy_text(output_file.clone());
                    }
                }
            });
            
            // Show status for output file
            if output_file.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                    "⚠ Please select an output location");
            } else {
                // Check if parent directory exists
                let parent_exists = std::path::Path::new(output_file)
                    .parent()
                    .map(|p| p.exists())
                    .unwrap_or(false);
                
                if parent_exists {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ Output location ready");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                        "⚠ Output directory does not exist");
                }
            }
        });
    }

    fn get_extension_and_filter(
        operation: &OperationType,
        video_settings: Option<&VideoSettings>,
        audio_settings: Option<&AudioSettings>,
        translations: &Translations
    ) -> (String, String, Vec<String>) {
        match operation {
            OperationType::AudioConvert | OperationType::AudioCompress | 
            OperationType::AudioResample | OperationType::AudioVolume | 
            OperationType::AudioTrim | OperationType::AudioMerge |
            OperationType::ExtractAudio => {
                if let Some(audio_settings) = audio_settings {
                    Self::get_audio_extension_and_filter(&audio_settings.codec, translations)
                } else {
                    ("mp3".to_string(), if translations.language == Language::Chinese { "音频文件" } else { "Audio Files" }.to_string(), vec!["mp3".to_string()])
                }
            },
            
            OperationType::VideoConvert | OperationType::VideoCompress | 
            OperationType::VideoResize | OperationType::VideoCrop | 
            OperationType::VideoRotate | OperationType::VideoFilter |
            OperationType::ExtractVideo => {
                if let Some(video_settings) = video_settings {
                    Self::get_video_extension_and_filter(&video_settings.codec, translations)
                } else {
                    ("mp4".to_string(), translations.video_files().to_string(), vec!["mp4".to_string()])
                }
            },
            
            OperationType::VideoAudioMerge => {
                if let Some(video_settings) = video_settings {
                    Self::get_video_extension_and_filter(&video_settings.codec, translations)
                } else {
                    ("mp4".to_string(), translations.video_files().to_string(), vec!["mp4".to_string()])
                }
            },
            
            OperationType::FrameExtract => {
                ("png".to_string(), translations.image_files().to_string(), vec!["png".to_string(), "jpg".to_string(), "bmp".to_string()])
            },
            
            OperationType::VideoToGif | OperationType::GifResize => {
                ("gif".to_string(), 
                 if translations.language == Language::Chinese { "GIF文件" } else { "GIF Files" }.to_string(), 
                 vec!["gif".to_string()])
            },
            
            _ => ("mp4".to_string(), translations.video_files().to_string(), vec!["mp4".to_string()]),
        }
    }
    
    fn get_audio_extension_and_filter(codec: &str, translations: &Translations) -> (String, String, Vec<String>) {
        let (ext, name_key, exts) = match codec {
            "libmp3lame" => ("mp3", "MP3", vec!["mp3"]),
            "aac" => ("aac", "AAC", vec!["aac", "m4a"]),
            "flac" => ("flac", "FLAC", vec!["flac"]),
            "pcm_s16le" => ("wav", "WAV", vec!["wav"]),
            "libvorbis" => ("ogg", "OGG", vec!["ogg"]),
            "libopus" => ("opus", "Opus", vec!["opus"]),
            "ac3" => ("ac3", "AC3", vec!["ac3"]),
            "auto" => ("mp3", "", vec!["mp3", "aac", "flac", "wav"]),
            _ => ("mp3", "", vec!["mp3"]),
        };
        
        let display_name = if name_key.is_empty() {
            if translations.language == Language::Chinese { "音频文件" } else { "Audio Files" }
        } else {
            &format!("{} {}", name_key, if translations.language == Language::Chinese { "音频文件" } else { "Audio Files" })
        };
        
        (ext.to_string(), display_name.to_string(), exts.into_iter().map(|s| s.to_string()).collect())
    }
    
    fn get_video_extension_and_filter(codec: &str, translations: &Translations) -> (String, String, Vec<String>) {
        match codec {
            "libx264" | "libx265" | "libaom-av1" => {
                ("mp4".to_string(), "MP4视频文件".to_string(), vec!["mp4".to_string(), "mov".to_string()])
            },
            "libvpx-vp9" => {
                ("webm".to_string(), "WebM视频文件".to_string(), vec!["webm".to_string()])
            },
            "auto" => {
                ("mp4".to_string(), if translations.language == Language::Chinese { "视频文件" } else { "Video Files" }.to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string()])
            },
            _ => ("mp4".to_string(), if translations.language == Language::Chinese { "视频文件" } else { "Video Files" }.to_string(), vec!["mp4".to_string()]),
        }
    }
    
    fn get_default_extension(operation: &OperationType) -> &'static str {
        match operation {
            OperationType::VideoConvert | OperationType::VideoCompress | 
            OperationType::VideoResize | OperationType::VideoCrop | 
            OperationType::VideoRotate | OperationType::VideoFilter |
            OperationType::VideoAudioMerge => "mp4",
            
            OperationType::AudioConvert | OperationType::AudioCompress | 
            OperationType::AudioResample | OperationType::AudioVolume | 
            OperationType::AudioTrim | OperationType::AudioMerge |
            OperationType::ExtractAudio => "mp3",
            
            OperationType::ExtractVideo => "mp4",
            OperationType::FrameExtract => "png",
            _ => "mp4",
        }
    }
}

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn show_video_settings(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations, cached_hw_encoders: &[String]) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(format!("🎬 {}", translations.video_settings()));
                
                // Video encoder selection removed - now handled by comprehensive UI in operation settings

                ui.horizontal(|ui| {
                    ui.label(translations.preset());
                    egui::ComboBox::from_id_salt("video_preset")
                        .selected_text(&settings.preset)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.preset, "auto".to_string(), translations.preset_auto());
                            ui.selectable_value(&mut settings.preset, "ultrafast".to_string(), translations.preset_ultrafast());
                            ui.selectable_value(&mut settings.preset, "superfast".to_string(), translations.preset_superfast());
                            ui.selectable_value(&mut settings.preset, "veryfast".to_string(), translations.preset_veryfast());
                            ui.selectable_value(&mut settings.preset, "faster".to_string(), translations.preset_faster());
                            ui.selectable_value(&mut settings.preset, "fast".to_string(), translations.preset_fast());
                            ui.selectable_value(&mut settings.preset, "medium".to_string(), translations.preset_medium());
                            ui.selectable_value(&mut settings.preset, "slow".to_string(), translations.preset_slow());
                            ui.selectable_value(&mut settings.preset, "slower".to_string(), translations.preset_slower());
                            ui.selectable_value(&mut settings.preset, "veryslow".to_string(), translations.preset_veryslow());
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.profile());
                    egui::ComboBox::from_id_salt("video_profile")
                        .selected_text(&settings.profile)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.profile, "auto".to_string(), translations.preset_auto());
                            ui.selectable_value(&mut settings.profile, "baseline".to_string(), translations.profile_baseline());
                            ui.selectable_value(&mut settings.profile, "main".to_string(), translations.profile_main());
                            ui.selectable_value(&mut settings.profile, "high".to_string(), translations.profile_high());
                            ui.selectable_value(&mut settings.profile, "high10".to_string(), "High 10-bit");
                            ui.selectable_value(&mut settings.profile, "high422".to_string(), "High 4:2:2");
                            ui.selectable_value(&mut settings.profile, "high444".to_string(), "High 4:4:4");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.tune());
                    egui::ComboBox::from_id_salt("video_tune")
                        .selected_text(&settings.tune)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.tune, "auto".to_string(), translations.preset_auto());
                            ui.selectable_value(&mut settings.tune, "film".to_string(), translations.tune_film());
                            ui.selectable_value(&mut settings.tune, "animation".to_string(), translations.tune_animation());
                            ui.selectable_value(&mut settings.tune, "grain".to_string(), translations.tune_grain());
                            ui.selectable_value(&mut settings.tune, "stillimage".to_string(), translations.tune_stillimage());
                            ui.selectable_value(&mut settings.tune, "psnr".to_string(), translations.tune_psnr());
                            ui.selectable_value(&mut settings.tune, "ssim".to_string(), translations.tune_ssim());
                            ui.selectable_value(&mut settings.tune, "fastdecode".to_string(), translations.tune_fastdecode());
                            ui.selectable_value(&mut settings.tune, "zerolatency".to_string(), translations.tune_zerolatency());
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.quality());
                    ui.add(egui::Slider::new(&mut settings.quality, 0..=51).text(""));
                    ui.label(if translations.language == Language::Chinese {
                        "(0=无损, 23=默认, 51=最低质量)"
                    } else {
                        "(0=lossless, 23=default, 51=lowest quality)"
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.bitrate());
                    ui.text_edit_singleline(&mut settings.bitrate);
                    ui.label(if translations.language == Language::Chinese { "(如: 2M, 1000k, 或 auto)" } else { "(e.g.: 2M, 1000k, or auto)" });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.framerate());
                    ui.text_edit_singleline(&mut settings.fps);
                    ui.label(if translations.language == Language::Chinese { "(如: 30, 60, 或 auto)" } else { "(e.g.: 30, 60, or auto)" });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.resolution());
                    ui.add(egui::DragValue::new(&mut settings.resolution.0).prefix(translations.width()));
                    ui.add(egui::DragValue::new(&mut settings.resolution.1).prefix(translations.height()));
                    ui.label(translations.keep_original_hint());
                });

                ui.separator();
                let smart_title = if translations.language == Language::Chinese {
                    "🎯 智能编码器推荐"
                } else {
                    "🎯 Smart Encoder Recommendation"
                };
                ui.collapsing(smart_title, |ui| {
                    ui.horizontal(|ui| {
                        let quality_label = if translations.language == Language::Chinese {
                            "质量预设:"
                        } else {
                            "Quality Preset:"
                        };
                        ui.label(quality_label);
                        
                        egui::ComboBox::from_id_salt("quality_preset")
                            .selected_text(&settings.quality_preset)
                            .show_ui(ui, |ui| {
                                if translations.language == Language::Chinese {
                                    ui.selectable_value(&mut settings.quality_preset, "高质量".to_string(), "🌟 高质量 (最佳画质)");
                                    ui.selectable_value(&mut settings.quality_preset, "平衡".to_string(), "⚖ 平衡 (质量与速度兼顾)");
                                    ui.selectable_value(&mut settings.quality_preset, "快速".to_string(), "⚡ 快速 (优先编码速度)");
                                } else {
                                    ui.selectable_value(&mut settings.quality_preset, "High Quality".to_string(), "🌟 High Quality (Best Picture)");
                                    ui.selectable_value(&mut settings.quality_preset, "Balanced".to_string(), "⚖ Balanced (Quality & Speed)");
                                    ui.selectable_value(&mut settings.quality_preset, "Fast".to_string(), "⚡ Fast (Speed Priority)");
                                }
                            });
                    });
                    
                    ui.horizontal(|ui| {
                        let speed_checkbox_text = if translations.language == Language::Chinese {
                            "🏃 优先编码速度而非质量"
                        } else {
                            "🏃 Prioritize encoding speed over quality"
                        };
                        ui.checkbox(&mut settings.speed_priority, speed_checkbox_text);
                    });
                    
                    {
                            
                        let (recommended_codec, reason) = crate::codec_manager::CodecManager::get_smart_encoder_recommendation(
                            &settings.container_format,
                            &settings.quality_preset,
                            settings.speed_priority,
                            cached_hw_encoders,
                            translations.language == crate::language::Language::Chinese
                        );
                            
                        
                            ui.horizontal(|ui| {
                                let recommend_label = if translations.language == Language::Chinese {
                                    "💡 推荐编码器:"
                                } else {
                                    "💡 Recommended Encoder:"
                                };
                                ui.label(recommend_label);
                                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), &recommended_codec);
                                
                                let apply_button_text = if translations.language == Language::Chinese {
                                    "应用"
                                } else {
                                    "Apply"
                                };
                                if ui.small_button(apply_button_text).clicked() {
                                    settings.codec = recommended_codec.clone();
                                }
                            });
                            
                            ui.horizontal(|ui| {
                                let reason_label = if translations.language == Language::Chinese {
                                    "📝 推荐理由:"
                                } else {
                                    "📝 Reason:"
                                };
                                ui.label(reason_label);
                                ui.colored_label(egui::Color32::from_rgb(180, 180, 180), &reason);
                            });
                    }
                });

                ui.separator();
                ui.label(translations.custom_parameters());
                ui.text_edit_multiline(&mut settings.custom_args);
                ui.label(translations.advanced_users_hint());
            });
        });
    }

    pub fn show_audio_settings(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(format!("🎵 {}", translations.audio_settings()));
                
                // Audio format selection (for audio-only operations)
                
                let supports_bitrate = Self::codec_supports_bitrate(&settings.codec);

                ui.horizontal(|ui| {
                    ui.label(translations.bitrate());
                    ui.add_enabled_ui(supports_bitrate, |ui| {
                        egui::ComboBox::from_id_salt("audio_bitrate")
                            .selected_text(&settings.bitrate)
                            .width(150.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut settings.bitrate, "auto".to_string(), "🎯 自动");
                                if supports_bitrate {
                                    ui.separator();
                                    ui.selectable_value(&mut settings.bitrate, "32k".to_string(), "32 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "64k".to_string(), "64 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "96k".to_string(), "96 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "128k".to_string(), if translations.language == Language::Chinese { "128 kbps (推荐)" } else { "128 kbps (Recommended)" });
                                    ui.selectable_value(&mut settings.bitrate, "160k".to_string(), "160 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "192k".to_string(), "192 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "224k".to_string(), "224 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "256k".to_string(), "256 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "320k".to_string(), if translations.language == Language::Chinese { "320 kbps (高质量)" } else { "320 kbps (High Quality)" });
                                    ui.selectable_value(&mut settings.bitrate, "448k".to_string(), "448 kbps");
                                    ui.selectable_value(&mut settings.bitrate, "512k".to_string(), "512 kbps");
                                }
                            });
                        
                        if supports_bitrate {
                            ui.add_space(10.0);
                            let mut custom_bitrate = if settings.bitrate.ends_with("k") || settings.bitrate == "auto" {
                                String::new()
                            } else {
                                settings.bitrate.clone()
                            };
                            
                            ui.label(translations.custom());
                            ui.text_edit_singleline(&mut custom_bitrate);
                            if !custom_bitrate.is_empty() && custom_bitrate != settings.bitrate {
                                settings.bitrate = custom_bitrate;
                            }
                        }
                    });
                    
                    if !supports_bitrate {
                        ui.label(translations.lossless_encoding());
                        settings.bitrate = "auto".to_string();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(translations.sample_rate());
                    egui::ComboBox::from_id_salt("audio_sample_rate")
                        .selected_text(&settings.sample_rate)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.sample_rate, "auto".to_string(), if translations.language == Language::Chinese { "🎯 自动" } else { "🎯 Auto" });
                            ui.separator();
                            ui.selectable_value(&mut settings.sample_rate, "8000".to_string(), if translations.language == Language::Chinese { "8 kHz (电话质量)" } else { "8 kHz (Phone Quality)" });
                            ui.selectable_value(&mut settings.sample_rate, "11025".to_string(), "11.025 kHz");
                            ui.selectable_value(&mut settings.sample_rate, "16000".to_string(), if translations.language == Language::Chinese { "16 kHz (语音)" } else { "16 kHz (Voice)" });
                            ui.selectable_value(&mut settings.sample_rate, "22050".to_string(), "22.05 kHz");
                            ui.selectable_value(&mut settings.sample_rate, "32000".to_string(), "32 kHz");
                            ui.selectable_value(&mut settings.sample_rate, "44100".to_string(), if translations.language == Language::Chinese { "🎵 44.1 kHz (CD质量)" } else { "🎵 44.1 kHz (CD Quality)" });
                            ui.selectable_value(&mut settings.sample_rate, "48000".to_string(), if translations.language == Language::Chinese { "⭐ 48 kHz (推荐)" } else { "⭐ 48 kHz (Recommended)" });
                            ui.selectable_value(&mut settings.sample_rate, "88200".to_string(), if translations.language == Language::Chinese { "88.2 kHz (高质量)" } else { "88.2 kHz (High Quality)" });
                            ui.selectable_value(&mut settings.sample_rate, "96000".to_string(), if translations.language == Language::Chinese { "💎 96 kHz (专业级)" } else { "💎 96 kHz (Professional)" });
                            ui.selectable_value(&mut settings.sample_rate, "176400".to_string(), "176.4 kHz");
                            ui.selectable_value(&mut settings.sample_rate, "192000".to_string(), if translations.language == Language::Chinese { "🏆 192 kHz (最高质量)" } else { "🏆 192 kHz (Highest Quality)" });
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(translations.channels());
                    egui::ComboBox::from_id_salt("audio_channels")
                        .selected_text(&settings.channels)
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.channels, "auto".to_string(), if translations.language == Language::Chinese { "🎯 自动" } else { "🎯 Auto" });
                            ui.separator();
                            ui.selectable_value(&mut settings.channels, "1".to_string(), if translations.language == Language::Chinese { "🔊 单声道" } else { "🔊 Mono" });
                            ui.selectable_value(&mut settings.channels, "2".to_string(), if translations.language == Language::Chinese { "🎧 立体声" } else { "🎧 Stereo" });
                            ui.selectable_value(&mut settings.channels, "3".to_string(), if translations.language == Language::Chinese { "🔊 2.1声道" } else { "🔊 2.1 Channel" });
                            ui.selectable_value(&mut settings.channels, "4".to_string(), if translations.language == Language::Chinese { "🔊 4.0声道" } else { "🔊 4.0 Channel" });
                            ui.selectable_value(&mut settings.channels, "5".to_string(), if translations.language == Language::Chinese { "🔊 4.1声道" } else { "🔊 4.1 Channel" });
                            ui.selectable_value(&mut settings.channels, "6".to_string(), if translations.language == Language::Chinese { "🏠 5.1声道" } else { "🏠 5.1 Channel" });
                            ui.selectable_value(&mut settings.channels, "7".to_string(), if translations.language == Language::Chinese { "🔊 6.1声道" } else { "🔊 6.1 Channel" });
                            ui.selectable_value(&mut settings.channels, "8".to_string(), if translations.language == Language::Chinese { "🎬 7.1声道" } else { "🎬 7.1 Channel" });
                        });
                });

                // VBR quality settings (only show for supported codecs)
                let supports_vbr = Self::codec_supports_vbr_quality(&settings.codec);
                ui.horizontal(|ui| {
                    ui.label(translations.vbr_quality());
                    ui.add_enabled_ui(supports_vbr, |ui| {
                        ui.text_edit_singleline(&mut settings.quality);
                        if supports_vbr {
                            let hint = match settings.codec.as_str() {
                                "libmp3lame" => "(0-9, 0=最高质量)",
                                "aac" => "(1-5, 1=最高质量)",
                                "libvorbis" | "libopus" => "(0-10, 10=最高质量)",
                                _ => "(auto)",
                            };
                            ui.label(hint);
                        } else {
                            ui.label(translations.vbr_not_supported());
                            settings.quality = "auto".to_string();
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(if translations.language == Language::Chinese {
                        "🔊 音量:"
                    } else {
                        "🔊 Volume:"
                    });
                    let volume_value = settings.volume; // Save current value first
                    ui.add(egui::Slider::new(&mut settings.volume, 0.0..=3.0)
                        .text(format!("{:.2}x", volume_value))
                        .step_by(0.1));
                    if volume_value != 1.0 {
                        ui.label(format!("({:+.1} dB)", 20.0 * volume_value.log10()));
                    }
                });

                ui.separator();
                ui.label(translations.custom_parameters());
                ui.text_edit_multiline(&mut settings.custom_args);
                ui.label(translations.advanced_users_hint());
            });
        });
    }

    pub fn show_preset_selector(
        ui: &mut egui::Ui, 
        video_settings: &mut VideoSettings, 
        audio_settings: &mut AudioSettings,
        translations: &Translations
    ) -> bool {
        let mut preset_applied = false;
        
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(translations.encoding_presets());
                ui.label(translations.select_preset_hint());
                ui.separator();
                
                let _presets = PresetManager::get_builtin_presets();
                let categories = PresetManager::get_all_categories();
                
                for category in categories {
                    ui.collapsing(category.display_name(translations), |ui| {
                        let category_presets = PresetManager::get_presets_by_category(&category);
                        
                        for preset in category_presets {
                            ui.horizontal(|ui| {
                                if ui.button(&preset.name).clicked() {
                                    PresetManager::apply_preset_to_settings(
                                        &preset, 
                                        video_settings, 
                                        audio_settings
                                    );
                                    preset_applied = true;
                                }
                                
                                ui.label(&preset.description);
                                
                                if !preset.recommended_formats.is_empty() {
                                    ui.label(format!("({})", preset.recommended_formats.join(", ")));
                                }
                            });
                        }
                    });
                }
                
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(translations.save_current_as_preset()).clicked() {
                    }
                    
                    if ui.button(translations.load_custom_preset()).clicked() {
                    }
                });
            });
        });
        
        preset_applied
    }

    pub fn show_smart_recommendations(
        ui: &mut egui::Ui,
        output_format: &str,
        video_settings: &mut VideoSettings,
        audio_settings: &mut AudioSettings,
        translations: &Translations,
    ) -> bool {
        let mut settings_applied = false;
        
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.heading(translations.smart_recommendations());
                ui.label(format!("{} '{}' {}", 
                    translations.recommended_settings_for(),
                    output_format,
                    translations.recommended_settings_suffix()));
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button(translations.high_quality()).clicked() {
                        let (video, audio) = CodecManager::get_recommended_settings("", output_format, "high");
                        *video_settings = video;
                        *audio_settings = audio;
                        settings_applied = true;
                    }
                    
                    if ui.button(translations.balanced()).clicked() {
                        let (video, audio) = CodecManager::get_recommended_settings("", output_format, "medium");
                        *video_settings = video;
                        *audio_settings = audio;
                        settings_applied = true;
                    }
                    
                    if ui.button(translations.fast()).clicked() {
                        let (video, audio) = CodecManager::get_recommended_settings("", output_format, "fast");
                        *video_settings = video;
                        *audio_settings = audio;
                        settings_applied = true;
                    }
                });
                
                let format_presets = PresetManager::recommend_presets_for_format(output_format);
                if !format_presets.is_empty() {
                    ui.separator();
                    ui.label(translations.format_specific_presets());
                    
                    for preset in format_presets.iter().take(3) {
                        if ui.button(&preset.name).clicked() {
                            PresetManager::apply_preset_to_settings(preset, video_settings, audio_settings);
                            settings_applied = true;
                        }
                    }
                }
            });
        });
        
        settings_applied
    }
    
    fn codec_supports_bitrate(codec: &str) -> bool {
        match codec {
            "flac" | "pcm_s16le" | "pcm_s24le" | "copy" => false,
            "auto" => true,
            _ => true,
        }
    }
    
    fn codec_supports_vbr_quality(codec: &str) -> bool {
        match codec {
            "libmp3lame" | "aac" | "libvorbis" | "libopus" => true,
            _ => false,
        }
    }

    fn show_video_codec_option(ui: &mut egui::Ui, current_codec: &mut String, codec: &str, display_name: &str, _translations: &Translations) {
        let is_selected = current_codec == codec;
        let quality_rating = CodecManager::get_codec_quality_rating(codec, false);
        let speed_rating = CodecManager::get_codec_speed_rating(codec, false);
        let description = CodecManager::get_video_codec_description(codec);
        
        let quality_stars = "★".repeat(quality_rating as usize) + &"☆".repeat(5 - quality_rating as usize);
        let speed_stars = "🏃".repeat(speed_rating as usize) + &"🐌".repeat(5 - speed_rating as usize);
        
        let label_text = format!("{} | Q:{} | S:{}", display_name, quality_stars, speed_stars);
        
        let response = ui.selectable_label(is_selected, label_text)
            .on_hover_text(format!("{}\n\nQuality: {}/5 stars\nSpeed: {}/5 stars\n\n{}", 
                display_name, quality_rating, speed_rating, description));
        
        if response.clicked() {
            *current_codec = codec.to_string();
        }
    }

    fn show_audio_codec_option(ui: &mut egui::Ui, current_codec: &mut String, codec: &str, display_name: &str, _translations: &Translations) {
        let is_selected = current_codec == codec;
        let quality_rating = CodecManager::get_codec_quality_rating(codec, true);
        let speed_rating = CodecManager::get_codec_speed_rating(codec, true);
        let description = CodecManager::get_audio_codec_description(codec);
        
        let quality_stars = "★".repeat(quality_rating as usize) + &"☆".repeat(5 - quality_rating as usize);
        let speed_stars = "🏃".repeat(speed_rating as usize) + &"🐌".repeat(5 - speed_rating as usize);
        
        let label_text = format!("{} | Q:{} | S:{}", display_name, quality_stars, speed_stars);
        
        let response = ui.selectable_label(is_selected, label_text)
            .on_hover_text(format!("{}\n\nQuality: {}/5 stars\nSpeed: {}/5 stars\n\n{}", 
                display_name, quality_rating, speed_rating, description));
        
        if response.clicked() {
            *current_codec = codec.to_string();
        }
    }
}

pub struct TaskPanel;

impl TaskPanel {
    pub fn show(ui: &mut egui::Ui, tasks: &mut Vec<ProcessingTask>, translations: &Translations) {
        ui.vertical(|ui| {
            if tasks.is_empty() {
                ui.label(translations.no_tasks());
                return;
            }

            {
                        let mut to_remove = Vec::new();
                        
                        for (i, task) in tasks.iter_mut().enumerate() {
                            ui.group(|ui| {
                                // First row: Task title and cancel/delete button on the same line
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} {}: {}", translations.task(), i + 1, task.operation.display_name(translations)));
                                    
                                    // Push button to the right
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let button_clicked = if task.status == TaskStatus::Running {
                                            ui.small_button(egui::RichText::new(translations.cancel()).color(egui::Color32::RED)).clicked()
                                        } else {
                                            ui.small_button(egui::RichText::new(translations.delete()).color(egui::Color32::RED)).clicked()
                                        };
                                        
                                        if button_clicked {
                                            if task.status == TaskStatus::Running {
                                                task.status = TaskStatus::Cancelled;
                                                log_info!("🛑 User clicked cancel, terminating FFmpeg processes...");
                                                Self::terminate_ffmpeg_processes();
                                            } else {
                                                to_remove.push(i);
                                            }
                                        }
                                    });
                                });
                                
                                // Second row: Status
                                ui.label(format!("{}: {:?}", translations.status(), task.status));
                                
                                // Progress bar and timing information
                                if task.status == TaskStatus::Running || task.status == TaskStatus::Completed {
                                    let progress_bar = egui::ProgressBar::new(task.progress)
                                        .text(format!("{:.1}%", task.progress * 100.0));
                                    ui.add_sized([ui.available_width(), 20.0], progress_bar);
                                    
                                    // Show time information for running tasks
                                    if task.status == TaskStatus::Running {
                                        if let Some(start_time) = task.start_time {
                                            let elapsed = start_time.elapsed();
                                            let elapsed_secs = elapsed.as_secs();
                                            
                                            ui.horizontal(|ui| {
                                                ui.label(format!("⏱ {}: {}:{:02}", 
                                                    if translations.language == crate::language::Language::Chinese { "已用时间" } else { "Elapsed" },
                                                    elapsed_secs / 60, elapsed_secs % 60
                                                ));
                                                
                                                if task.progress > 0.0 {
                                                    // Simple time estimation based on progress
                                                    let estimated_total_secs = (elapsed_secs as f32 / task.progress) as u64;
                                                    let remaining_secs = estimated_total_secs.saturating_sub(elapsed_secs);
                                                    
                                                    ui.separator();
                                                    ui.label(format!("⏳ {}: {}:{:02}", 
                                                        if translations.language == crate::language::Language::Chinese { "剩余时间" } else { "Remaining" },
                                                        remaining_secs / 60, remaining_secs % 60
                                                    ));
                                                } else {
                                                    ui.separator();
                                                    ui.label(if translations.language == crate::language::Language::Chinese { 
                                                        "⏳ 计算中..." 
                                                    } else { 
                                                        "⏳ Calculating..." 
                                                    });
                                                }
                                            });
                                        }
                                    } else if task.status == TaskStatus::Completed {
                                        if let Some(completion_time) = task.completion_time {
                                            let total_secs = completion_time.as_secs();
                                            ui.label(format!("✅ {}: {}:{:02}", 
                                                if translations.language == crate::language::Language::Chinese { "总用时" } else { "Total Time" },
                                                total_secs / 60, total_secs % 60
                                            ));
                                        } else if let Some(start_time) = task.start_time {
                                            // Fallback for tasks completed before this fix
                                            let total_time = start_time.elapsed();
                                            let total_secs = total_time.as_secs();
                                            ui.label(format!("✅ {}: {}:{:02}", 
                                                if translations.language == crate::language::Language::Chinese { "总用时" } else { "Total Time" },
                                                total_secs / 60, total_secs % 60
                                            ));
                                        }
                                    }
                                }
                                
                                if let Some(error) = &task.error_message {
                                    ui.colored_label(egui::Color32::RED, format!("{}: {}", translations.error(), error));
                                }
                            });
                        }
                        
                        for &i in to_remove.iter().rev() {
                            tasks.remove(i);
                        }
            }
        });
    }
    
    fn terminate_ffmpeg_processes() {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/IM", "ffmpeg.exe"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("pkill")
                .args(&["-f", "ffmpeg"])
                .output();
        }
    }
}