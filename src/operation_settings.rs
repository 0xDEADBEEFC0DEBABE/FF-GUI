use crate::app_state::{OperationType, VideoSettings, AudioSettings};
use crate::language::Translations;
use crate::comprehensive_codec_registry::CodecType;
use crate::comprehensive_ui_components::{ComprehensiveUIComponents, FormatPurpose};
use egui;

pub struct OperationSettings;

impl OperationSettings {
    pub fn show(ctx: &egui::Context, ui: &mut egui::Ui, operation: &OperationType, video_settings: &mut VideoSettings, audio_settings: &mut AudioSettings, translations: &Translations, detected_resolution: Option<(u32, u32)>, is_portrait: Option<bool>, cached_hw_encoders: &[String]) {
        match operation {
            // Video operations
            OperationType::VideoConvert => Self::show_video_convert(ui, video_settings, translations, cached_hw_encoders),
            OperationType::VideoCompress => Self::show_video_compress(ui, video_settings, translations, cached_hw_encoders),
            OperationType::VideoResize => Self::show_video_resize(ui, video_settings, translations, detected_resolution, is_portrait),
            OperationType::VideoCrop => Self::show_video_crop(ui, video_settings, translations),
            OperationType::VideoRotate => Self::show_video_rotate(ui, video_settings, translations),
            OperationType::VideoFilter => Self::show_video_filter(ui, video_settings, translations),
            
            // Audio operations
            OperationType::AudioConvert => Self::show_audio_convert(ui, audio_settings, translations, cached_hw_encoders),
            OperationType::AudioCompress => Self::show_audio_compress(ui, audio_settings, translations),
            OperationType::AudioResample => Self::show_audio_resample(ui, audio_settings, translations),
            OperationType::AudioVolume => Self::show_audio_volume(ui, audio_settings, translations),
            OperationType::AudioTrim => Self::show_audio_trim(ui, audio_settings, translations),
            OperationType::AudioMerge => Self::show_audio_merge(ui, audio_settings, translations),
            
            // Combined operations
            OperationType::VideoAudioMerge => Self::show_video_audio_merge(ui, video_settings, audio_settings, translations),
            OperationType::VideoAudioSplit => Self::show_video_audio_split(ui, video_settings, audio_settings, translations),
            OperationType::ExtractAudio => Self::show_extract_audio(ui, audio_settings, translations),
            OperationType::ExtractVideo => Self::show_extract_video(ui, video_settings, translations),
            
            // Advanced operations
            OperationType::BatchConvert => Self::show_batch_convert(ui, video_settings, audio_settings, translations),
            OperationType::AddSubtitle => Self::show_add_subtitle(ctx, ui, video_settings, translations),
            OperationType::AddWatermark => Self::show_add_watermark(ctx, ui, video_settings, translations),
            OperationType::FrameExtract => Self::show_frame_extract(ui, video_settings, translations),
            OperationType::VideoToGif => Self::show_video_to_gif(ui, video_settings, translations),
            OperationType::GifResize => Self::show_gif_resize(ui, video_settings, translations),
        }
    }
    
    // Video Convert - Format conversion with codec selection
    fn show_video_convert(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations, cached_hw_encoders: &[String]) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎬 视频格式转换设置"
            } else {
                "🎬 Video Format Conversion Settings"
            });
            ui.separator();
            
            // Comprehensive output format selection
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "输出格式:"
                } else {
                    "Output Format:"
                });
            });
            
            ComprehensiveUIComponents::show_comprehensive_format_selector(
                ui,
                &mut settings.container_format,
                FormatPurpose::Video,
                translations
            );
            
            ui.add_space(10.0);
            
            // Comprehensive video codec selection
            if !settings.copy_video {
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "视频编码器:"
                    } else {
                        "Video Codec:"
                    });
                });
                
                ComprehensiveUIComponents::show_comprehensive_codec_selector(
                    ui,
                    CodecType::Video,
                    &mut settings.codec,
                    &settings.container_format,
                    translations,
                    cached_hw_encoders
                );
                
                // Show advanced codec settings
                let codec_name = settings.codec.clone();
                ComprehensiveUIComponents::show_advanced_codec_settings(
                    ui,
                    &codec_name,
                    settings,
                    translations
                );
                
                ui.add_space(10.0);
                
                // Show codec-format compatibility check
                ComprehensiveUIComponents::show_codec_format_compatibility_matrix(
                    ui,
                    &settings.codec,
                    &settings.container_format,
                    translations
                );
                
                ui.add_space(10.0);
            }
            
            // Conversion mode selection
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "转换模式:"
                } else {
                    "Conversion Mode:"
                });
                ui.radio_value(&mut settings.copy_video, false, if translations.language == crate::language::Language::Chinese {
                    "重新编码 (调整质量/大小)"
                } else {
                    "Re-encode (adjust quality/size)"
                });
                ui.radio_value(&mut settings.copy_video, true, if translations.language == crate::language::Language::Chinese {
                    "快速转换 (仅改变容器)"
                } else {
                    "Fast convert (container only)"
                });
            });
            
            ui.add_space(5.0);
            
            // Format-specific recommendations
            if !settings.copy_video {
                ui.horizontal(|ui| {
                    ui.label("💡");
                    ui.label(match settings.container_format.as_str() {
                        "mp4" => if translations.language == crate::language::Language::Chinese {
                            "MP4: 最佳兼容性，适合播放和分享"
                        } else {
                            "MP4: Best compatibility for playback and sharing"
                        },
                        "mkv" => if translations.language == crate::language::Language::Chinese {
                            "MKV: 支持多种编码器和字幕"
                        } else {
                            "MKV: Supports multiple codecs and subtitles"
                        },
                        "webm" => if translations.language == crate::language::Language::Chinese {
                            "WebM: 适合网页播放，开源格式"
                        } else {
                            "WebM: Optimized for web playback, open source"
                        },
                        "avi" => if translations.language == crate::language::Language::Chinese {
                            "AVI: 传统格式，广泛支持"
                        } else {
                            "AVI: Legacy format with wide support"
                        },
                        "mov" => if translations.language == crate::language::Language::Chinese {
                            "MOV: Apple格式，适合Mac和专业编辑"
                        } else {
                            "MOV: Apple format, ideal for Mac and pro editing"
                        },
                        _ => if translations.language == crate::language::Language::Chinese {
                            "选择适合您需求的格式"
                        } else {
                            "Choose format that suits your needs"
                        }
                    });
                });
            }
        });
    }
    
    // Video Compress - Bitrate and quality controls
    fn show_video_compress(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations, _cached_hw_encoders: &[String]) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "📊 视频压缩设置"
            } else {
                "📊 Video Compression Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    translations.compression_quality()
                } else {
                    "Compression Quality:"
                });
                ui.radio_value(&mut settings.crf, 18, &format!("{} ({})", translations.high_quality(), translations.large_file()));
                ui.radio_value(&mut settings.crf, 23, translations.balanced());
                ui.radio_value(&mut settings.crf, 28, &format!("{} ({})", translations.high_compression(), translations.small_file()));
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "目标文件大小 (MB):"
                } else {
                    "Target File Size (MB):"
                });
                ui.add(egui::DragValue::new(&mut settings.target_size_mb)
                    .range(1..=10000)
                    .suffix(" MB"));
            });
            
            if settings.target_size_mb > 0 {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "⚠ 启用目标文件大小会进行两遍编码"
                } else {
                    "⚠ Target size enables two-pass encoding"
                });
            }
        });
    }
    
    // Video Resize - Resolution adjustment
    fn show_video_resize(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations, detected_resolution: Option<(u32, u32)>, is_portrait: Option<bool>) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "📐 视频分辨率调整"
            } else {
                "📐 Video Resolution Adjustment"
            });
            ui.separator();
            
            // Show detected video information and smart suggestions
            if let Some((width, height)) = detected_resolution {
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        format!("🎯 检测到视频: {}x{} {}", width, height, 
                            if is_portrait == Some(true) { "(竖屏)" } else { "(横屏)" })
                    } else {
                        format!("🎯 Detected: {}x{} {}", width, height,
                            if is_portrait == Some(true) { "(Portrait)" } else { "(Landscape)" })
                    });
                });
                
                if is_portrait == Some(true) {
                    ui.colored_label(egui::Color32::from_rgb(70, 130, 180), 
                        if translations.language == crate::language::Language::Chinese {
                            "💡 竖屏视频检测：建议使用下方竖屏预设或手动调整"
                        } else {
                            "💡 Portrait video detected: Use portrait presets below or adjust manually"
                        }
                    );
                }
                ui.add_space(5.0);
            }
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "横屏预设:"
                } else {
                    "Landscape Presets:"
                });
                if ui.button("720p").clicked() {
                    // Auto-adjust based on detected orientation
                    if is_portrait == Some(true) {
                        settings.width = Some(720);
                        settings.height = Some(1280);
                        settings.resolution = (720, 1280);
                    } else {
                        settings.width = Some(1280);
                        settings.height = Some(720);
                        settings.resolution = (1280, 720);
                    }
                }
                if ui.button("1080p").clicked() {
                    if is_portrait == Some(true) {
                        settings.width = Some(1080);
                        settings.height = Some(1920);
                        settings.resolution = (1080, 1920);
                    } else {
                        settings.width = Some(1920);
                        settings.height = Some(1080);
                        settings.resolution = (1920, 1080);
                    }
                }
                if ui.button("4K").clicked() {
                    if is_portrait == Some(true) {
                        settings.width = Some(2160);
                        settings.height = Some(3840);
                        settings.resolution = (2160, 3840);
                    } else {
                        settings.width = Some(3840);
                        settings.height = Some(2160);
                        settings.resolution = (3840, 2160);
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "竖屏预设:"
                } else {
                    "Portrait Presets:"
                });
                if ui.button("720p Portrait").clicked() {
                    settings.width = Some(720);
                    settings.height = Some(1280);
                    settings.resolution = (720, 1280);
                }
                if ui.button("1080p Portrait").clicked() {
                    settings.width = Some(1080);
                    settings.height = Some(1920);
                    settings.resolution = (1080, 1920);
                }
                if ui.button("4K Portrait").clicked() {
                    settings.width = Some(2160);
                    settings.height = Some(3840);
                    settings.resolution = (2160, 3840);
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(translations.custom_width());
                let mut width = settings.width.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut width).range(0..=7680)).changed() {
                    settings.width = if width > 0 { Some(width as u32) } else { None };
                    // Sync with resolution field
                    if let (Some(w), Some(h)) = (settings.width, settings.height) {
                        settings.resolution = (w, h);
                    }
                }
                
                ui.label(translations.height());
                let mut height = settings.height.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut height).range(0..=4320)).changed() {
                    settings.height = if height > 0 { Some(height as u32) } else { None };
                    // Sync with resolution field
                    if let (Some(w), Some(h)) = (settings.width, settings.height) {
                        settings.resolution = (w, h);
                    }
                }
            });
            
            ui.checkbox(&mut settings.maintain_aspect_ratio, 
                if translations.language == crate::language::Language::Chinese {
                    translations.maintain_aspect_ratio()
                } else {
                    "Maintain Aspect Ratio"
                });
        });
    }
    
    // Video Crop - Cropping controls
    fn show_video_crop(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔲 视频裁剪设置"
            } else {
                "🔲 Video Crop Settings"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "裁剪区域 (像素):"
            } else {
                "Crop Area (pixels):"
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese { "上:" } else { "Top:" });
                let mut top = settings.crop_top.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut top).range(0..=2160)).changed() {
                    settings.crop_top = if top > 0 { Some(top as u32) } else { None };
                }
                
                ui.label(if translations.language == crate::language::Language::Chinese { "下:" } else { "Bottom:" });
                let mut bottom = settings.crop_bottom.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut bottom).range(0..=2160)).changed() {
                    settings.crop_bottom = if bottom > 0 { Some(bottom as u32) } else { None };
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese { "左:" } else { "Left:" });
                let mut left = settings.crop_left.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut left).range(0..=3840)).changed() {
                    settings.crop_left = if left > 0 { Some(left as u32) } else { None };
                }
                
                ui.label(if translations.language == crate::language::Language::Chinese { "右:" } else { "Right:" });
                let mut right = settings.crop_right.unwrap_or(0) as i32;
                if ui.add(egui::DragValue::new(&mut right).range(0..=3840)).changed() {
                    settings.crop_right = if right > 0 { Some(right as u32) } else { None };
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "常用比例:"
                } else {
                    "Common Ratios:"
                });
                if ui.button("16:9").clicked() {
                    // TODO: Calculate crop based on input video
                }
                if ui.button("4:3").clicked() {
                    // TODO: Calculate crop based on input video
                }
                if ui.button("1:1").clicked() {
                    // TODO: Calculate crop based on input video
                }
            });
        });
    }
    
    // Video Rotate - Rotation options
    fn show_video_rotate(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔄 视频旋转设置"
            } else {
                "🔄 Video Rotation Settings"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "旋转角度:"
            } else {
                "Rotation Angle:"
            });
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut settings.use_custom_rotation, false, if translations.language == crate::language::Language::Chinese { "预设角度" } else { "Preset Angles" });
                ui.radio_value(&mut settings.use_custom_rotation, true, if translations.language == crate::language::Language::Chinese { "自定义角度" } else { "Custom Angle" });
            });
            
            ui.add_space(5.0);
            
            if !settings.use_custom_rotation {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut settings.rotation, 0, if translations.language == crate::language::Language::Chinese { "不旋转" } else { "No Rotation" });
                    ui.radio_value(&mut settings.rotation, 90, "90°");
                    ui.radio_value(&mut settings.rotation, 180, "180°");
                    ui.radio_value(&mut settings.rotation, 270, "270°");
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "角度 (度):"
                    } else {
                        "Angle (degrees):"
                    });
                    ui.add(egui::DragValue::new(&mut settings.custom_rotation_angle)
                        .speed(0.1)
                        .range(-360.0..=360.0)
                        .suffix("°"));
                });
                ui.horizontal(|ui| {
                    ui.label("💡");
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "正值为顺时针旋转，负值为逆时针旋转"
                    } else {
                        "Positive values rotate clockwise, negative values rotate counter-clockwise"
                    });
                });
            }
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.flip_horizontal, 
                if translations.language == crate::language::Language::Chinese {
                    "水平翻转"
                } else {
                    "Flip Horizontal"
                });
            ui.checkbox(&mut settings.flip_vertical, 
                if translations.language == crate::language::Language::Chinese {
                    "垂直翻转"
                } else {
                    "Flip Vertical"
                });
        });
    }
    
    // Video Filter - Various video filters
    fn show_video_filter(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎨 视频滤镜设置"
            } else {
                "🎨 Video Filter Settings"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "选择滤镜:"
            } else {
                "Select Filter:"
            });
            
            ui.checkbox(&mut settings.denoise, 
                if translations.language == crate::language::Language::Chinese {
                    "降噪"
                } else {
                    "Denoise"
                });
            ui.checkbox(&mut settings.deinterlace, 
                if translations.language == crate::language::Language::Chinese {
                    "去隔行"
                } else {
                    "Deinterlace"
                });
            ui.checkbox(&mut settings.stabilize, 
                if translations.language == crate::language::Language::Chinese {
                    "视频稳定"
                } else {
                    "Stabilize"
                });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "亮度:"
                } else {
                    "Brightness:"
                });
                ui.add(egui::Slider::new(&mut settings.brightness, -1.0..=1.0));
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "对比度:"
                } else {
                    "Contrast:"
                });
                ui.add(egui::Slider::new(&mut settings.contrast, -2.0..=2.0));
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "饱和度:"
                } else {
                    "Saturation:"
                });
                ui.add(egui::Slider::new(&mut settings.saturation, 0.0..=3.0));
            });
        });
    }
    
    // Audio Convert - Audio format conversion
    fn show_audio_convert(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations, cached_hw_encoders: &[String]) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎵 音频格式转换"
            } else {
                "🎵 Audio Format Conversion"
            });
            ui.separator();
            
            // Comprehensive audio format selection
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "输出格式:"
                } else {
                    "Output Format:"
                });
            });
            
            ComprehensiveUIComponents::show_comprehensive_format_selector(
                ui,
                &mut settings.format,
                FormatPurpose::Audio,
                translations
            );
            
            ui.add_space(10.0);
            
            // Comprehensive audio codec selection
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "音频编码器:"
                } else {
                    "Audio Codec:"  
                });
            });
            
            ComprehensiveUIComponents::show_comprehensive_codec_selector(
                ui,
                CodecType::Audio,
                &mut settings.codec,
                &settings.format,
                translations,
                cached_hw_encoders
            );
            
            ui.add_space(10.0);
            
            // Show codec-format compatibility check
            ComprehensiveUIComponents::show_codec_format_compatibility_matrix(
                ui,
                &settings.codec,
                &settings.format,
                translations
            );
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.copy_audio, 
                if translations.language == crate::language::Language::Chinese {
                    "保持原始音频流"
                } else {
                    "Keep Original Audio Stream"
                });
        });
    }
    
    // Audio Compress - Bitrate control
    fn show_audio_compress(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "📊 音频压缩设置"
            } else {
                "📊 Audio Compression Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "音频比特率:"
                } else {
                    "Audio Bitrate:"
                });
                ui.radio_value(&mut settings.bitrate, "64k".to_string(), "64 kbps");
                ui.radio_value(&mut settings.bitrate, "128k".to_string(), "128 kbps");
                ui.radio_value(&mut settings.bitrate, "192k".to_string(), "192 kbps");
                ui.radio_value(&mut settings.bitrate, "256k".to_string(), "256 kbps");
                ui.radio_value(&mut settings.bitrate, "320k".to_string(), "320 kbps");
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                let vbr_label = if translations.language == crate::language::Language::Chinese {
                    format!("{} (0-9):", translations.vbr_quality())
                } else {
                    "VBR Quality (0-9):".to_string()
                };
                ui.label(&vbr_label);
                ui.add(egui::DragValue::new(&mut settings.vbr_quality)
                    .range(0..=9));
            });
        });
    }
    
    // Audio Resample - Sample rate conversion
    fn show_audio_resample(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎵 音频重采样设置"
            } else {
                "🎵 Audio Resampling Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    translations.sample_rate()
                } else {
                    "Sample Rate:"
                });
                ui.radio_value(&mut settings.sample_rate, "22050".to_string(), "22.05 kHz");
                ui.radio_value(&mut settings.sample_rate, "44100".to_string(), "44.1 kHz");
                ui.radio_value(&mut settings.sample_rate, "48000".to_string(), "48 kHz");
                ui.radio_value(&mut settings.sample_rate, "96000".to_string(), "96 kHz");
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "重采样算法:"
                } else {
                    "Resampling Algorithm:"
                });
                egui::ComboBox::from_label("")
                    .selected_text(&settings.resample_method)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.resample_method, "swr".to_string(), "SWR (Default)");
                        ui.selectable_value(&mut settings.resample_method, "soxr".to_string(), "SoX (High Quality)");
                    });
            });
        });
    }
    
    // Audio Volume - Volume adjustment
    fn show_audio_volume(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔊 音量调整设置"
            } else {
                "🔊 Volume Adjustment Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "音量调整:"
                } else {
                    "Volume Adjustment:"
                });
                ui.add(egui::Slider::new(&mut settings.volume, 0.0..=2.0)
                    .suffix("x")
                    .logarithmic(true));
            });
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.normalize, 
                if translations.language == crate::language::Language::Chinese {
                    "音频标准化"
                } else {
                    "Normalize Audio"
                });
            
            if settings.normalize {
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "目标响度 (LUFS):"
                    } else {
                        "Target Loudness (LUFS):"
                    });
                    ui.add(egui::DragValue::new(&mut settings.target_lufs)
                        .range(-70.0..=0.0)
                        .suffix(" LUFS"));
                });
            }
        });
    }
    
    // Audio Trim - Cut audio segments
    fn show_audio_trim(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔍 音频裁剪设置"
            } else {
                "🔍 Audio Trim Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "开始时间:"
                } else {
                    "Start Time:"
                });
                ui.text_edit_singleline(&mut settings.start_time);
                ui.label("(HH:MM:SS)");
            });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "结束时间:"
                } else {
                    "End Time:"
                });
                ui.text_edit_singleline(&mut settings.end_time);
                ui.label("(HH:MM:SS)");
            });
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.fade_in, 
                if translations.language == crate::language::Language::Chinese {
                    "淡入效果"
                } else {
                    "Fade In"
                });
            ui.checkbox(&mut settings.fade_out, 
                if translations.language == crate::language::Language::Chinese {
                    "淡出效果"
                } else {
                    "Fade Out"
                });
        });
    }
    
    // Audio Merge - Combine multiple audio files
    fn show_audio_merge(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎶 音频合并设置"
            } else {
                "🎶 Audio Merge Settings"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "合并模式:"
                } else {
                    "Merge Mode:"
                });
                ui.radio_value(&mut settings.merge_mode, "concat".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "顺序连接" } else { "Concatenate" });
                ui.radio_value(&mut settings.merge_mode, "mix".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "混音" } else { "Mix" });
            });
            
            ui.add_space(10.0);
            
            if settings.merge_mode == "concat" {
                ui.checkbox(&mut settings.add_silence, 
                    if translations.language == crate::language::Language::Chinese {
                        "在音频之间添加静音"
                    } else {
                        "Add Silence Between Tracks"
                    });
                
                if settings.add_silence {
                    ui.horizontal(|ui| {
                        ui.label(if translations.language == crate::language::Language::Chinese {
                            "静音时长 (秒):"
                        } else {
                            "Silence Duration (seconds):"
                        });
                        ui.add(egui::DragValue::new(&mut settings.silence_duration)
                            .range(0.0..=10.0)
                            .suffix(" s"));
                    });
                }
            }
        });
    }
    
    // Video/Audio Merge - Combine video and audio
    fn show_video_audio_merge(ui: &mut egui::Ui, _video_settings: &mut VideoSettings, audio_settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎬🎵 视频音频合并"
            } else {
                "🎬🎵 Video/Audio Merge"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "请在文件选择区域选择视频文件和音频文件"
            } else {
                "Please select video and audio files in the file selection area"
            });
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut audio_settings.sync_audio, 
                if translations.language == crate::language::Language::Chinese {
                    "自动同步音频"
                } else {
                    "Auto-sync Audio"
                });
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "音频延迟 (秒):"
                } else {
                    "Audio Delay (seconds):"
                });
                ui.add(egui::DragValue::new(&mut audio_settings.audio_delay)
                    .range(-10.0..=10.0)
                    .speed(0.1)
                    .suffix(" s"));
            });
        });
    }
    
    // Video/Audio Split - Separate video and audio
    fn show_video_audio_split(ui: &mut egui::Ui, video_settings: &mut VideoSettings, audio_settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎬🎵 音视频分离"
            } else {
                "🎬🎵 Video/Audio Split"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "此操作将从视频文件中分离出音频和视频流到指定文件夹。"
            } else {
                "This operation will separate audio and video streams from the video file to the specified folder."
            });
            
            ui.add_space(10.0);
            
            // Video encoding settings
            ui.group(|ui| {
                ui.heading(if translations.language == crate::language::Language::Chinese {
                    "🎬 视频编码设置"
                } else {
                    "🎬 Video Encoding Settings"
                });
                
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "视频编码器:"
                    } else {
                        "Video Codec:"
                    });
                    egui::ComboBox::from_id_salt("video_split_codec")
                        .selected_text(&video_settings.codec)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut video_settings.codec, "copy".to_string(), "Copy (保持原格式)");
                            ui.selectable_value(&mut video_settings.codec, "libx264".to_string(), "H.264 (libx264)");
                            ui.selectable_value(&mut video_settings.codec, "libx265".to_string(), "H.265 (libx265)");
                            ui.selectable_value(&mut video_settings.codec, "libvpx".to_string(), "VP8 (libvpx)");
                            ui.selectable_value(&mut video_settings.codec, "libvpx-vp9".to_string(), "VP9 (libvpx-vp9)");
                            ui.selectable_value(&mut video_settings.codec, "libav1".to_string(), "AV1 (libav1)");
                            ui.selectable_value(&mut video_settings.codec, "mkv".to_string(), "MKV Container (原始编码)");
                        });
                });
                
                // Show CRF/Quality settings if not copy or mkv
                if video_settings.codec != "copy" && video_settings.codec != "mkv" {
                    ui.horizontal(|ui| {
                        ui.label(if translations.language == crate::language::Language::Chinese {
                            "视频质量 (CRF):"
                        } else {
                            "Video Quality (CRF):"
                        });
                        ui.add(egui::Slider::new(&mut video_settings.crf, 0..=51).suffix(" (0=无损, 23=默认, 51=最差)"));
                    });
                    
                    if video_settings.codec.contains("libx26") {
                        ui.horizontal(|ui| {
                            ui.label(if translations.language == crate::language::Language::Chinese {
                                "编码预设:"
                            } else {
                                "Encoding Preset:"
                            });
                            egui::ComboBox::from_id_salt("video_split_preset")
                                .selected_text(&video_settings.preset)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut video_settings.preset, "ultrafast".to_string(), "ultrafast (最快)");
                                    ui.selectable_value(&mut video_settings.preset, "fast".to_string(), "fast (快速)");
                                    ui.selectable_value(&mut video_settings.preset, "medium".to_string(), "medium (平衡)");
                                    ui.selectable_value(&mut video_settings.preset, "slow".to_string(), "slow (高质量)");
                                    ui.selectable_value(&mut video_settings.preset, "veryslow".to_string(), "veryslow (最高质量)");
                                });
                        });
                    }
                }
            });
            
            ui.add_space(5.0);
            
            // Audio encoding settings
            ui.group(|ui| {
                ui.heading(if translations.language == crate::language::Language::Chinese {
                    "🎵 音频编码设置"
                } else {
                    "🎵 Audio Encoding Settings"
                });
                
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "音频编码器:"
                    } else {
                        "Audio Codec:"
                    });
                    egui::ComboBox::from_id_salt("audio_split_codec")
                        .selected_text(&audio_settings.codec)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut audio_settings.codec, "copy".to_string(), "Copy (保持原格式)");
                            ui.selectable_value(&mut audio_settings.codec, "pcm_s16le".to_string(), "PCM 16-bit (无损WAV)");
                            ui.selectable_value(&mut audio_settings.codec, "pcm_s24le".to_string(), "PCM 24-bit (无损WAV)");
                            ui.selectable_value(&mut audio_settings.codec, "flac".to_string(), "FLAC (无损压缩)");
                            ui.selectable_value(&mut audio_settings.codec, "libmp3lame".to_string(), "MP3 (libmp3lame)");
                            ui.selectable_value(&mut audio_settings.codec, "aac".to_string(), "AAC");
                            ui.selectable_value(&mut audio_settings.codec, "libvorbis".to_string(), "Ogg Vorbis");
                            ui.selectable_value(&mut audio_settings.codec, "libopus".to_string(), "Opus");
                        });
                });
                
                // Show bitrate/quality settings if not copy or PCM
                if audio_settings.codec != "copy" && !audio_settings.codec.starts_with("pcm_") && audio_settings.codec != "flac" {
                    ui.horizontal(|ui| {
                        ui.label(if translations.language == crate::language::Language::Chinese {
                            "音频码率:"
                        } else {
                            "Audio Bitrate:"
                        });
                        egui::ComboBox::from_id_salt("audio_split_bitrate")
                            .selected_text(&audio_settings.bitrate)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut audio_settings.bitrate, "96k".to_string(), "96 kbps");
                                ui.selectable_value(&mut audio_settings.bitrate, "128k".to_string(), "128 kbps");
                                ui.selectable_value(&mut audio_settings.bitrate, "192k".to_string(), "192 kbps");
                                ui.selectable_value(&mut audio_settings.bitrate, "256k".to_string(), "256 kbps");
                                ui.selectable_value(&mut audio_settings.bitrate, "320k".to_string(), "320 kbps");
                            });
                    });
                }
                
                // Sample rate setting
                ui.horizontal(|ui| {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "采样率:"
                    } else {
                        "Sample Rate:"
                    });
                    egui::ComboBox::from_id_salt("audio_split_sample_rate")
                        .selected_text(&audio_settings.sample_rate)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut audio_settings.sample_rate, "auto".to_string(), "Auto (保持原始)");
                            ui.selectable_value(&mut audio_settings.sample_rate, "22050".to_string(), "22.05 kHz");
                            ui.selectable_value(&mut audio_settings.sample_rate, "44100".to_string(), "44.1 kHz");
                            ui.selectable_value(&mut audio_settings.sample_rate, "48000".to_string(), "48 kHz");
                            ui.selectable_value(&mut audio_settings.sample_rate, "96000".to_string(), "96 kHz");
                        });
                });
            });
            
            ui.add_space(10.0);
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "输出文件将命名为: 文件名_video.扩展名 和 文件名_audio.扩展名"
            } else {
                "Output files will be named: filename_video.ext and filename_audio.ext"
            });
        });
    }
    
    // Extract Audio - Extract audio from video
    fn show_extract_audio(ui: &mut egui::Ui, settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎵 提取音频"
            } else {
                "🎵 Extract Audio"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "提取格式:"
                } else {
                    "Extract Format:"
                });
                ui.radio_value(&mut settings.format, "original".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "保持原格式" } else { "Keep Original" });
                ui.radio_value(&mut settings.format, "mp3".to_string(), "MP3");
                ui.radio_value(&mut settings.format, "aac".to_string(), "AAC");
                ui.radio_value(&mut settings.format, "wav".to_string(), "WAV");
            });
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.extract_all_tracks, 
                if translations.language == crate::language::Language::Chinese {
                    "提取所有音轨"
                } else {
                    "Extract All Audio Tracks"
                });
        });
    }
    
    // Extract Video - Remove audio from video
    fn show_extract_video(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎬 提取视频"
            } else {
                "🎬 Extract Video"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "此操作将移除视频中的所有音频流。"
            } else {
                "This operation will remove all audio streams from the video."
            });
            
            ui.add_space(10.0);
            
            ui.checkbox(&mut settings.copy_video, 
                if translations.language == crate::language::Language::Chinese {
                    "无损提取 (不重新编码)"
                } else {
                    "Lossless Extraction (No Re-encoding)"
                });
        });
    }
    
    // Batch Convert - Multiple file conversion
    fn show_batch_convert(ui: &mut egui::Ui, video_settings: &mut VideoSettings, _audio_settings: &mut AudioSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "📦 批量转换设置"
            } else {
                "📦 Batch Conversion Settings"
            });
            ui.separator();
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "选择要进行的批量操作类型，所有文件将使用当前的对应设置进行转换。"
            } else {
                "Select the batch operation type. All files will be converted using current settings for the selected operation."
            });
            
            ui.add_space(10.0);
            
            // Batch operation type selection
            ui.label(if translations.language == crate::language::Language::Chinese {
                "批量操作类型:"
            } else {
                "Batch Operation Type:"
            });
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut video_settings.batch_operation_type, "convert".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "格式转换" } else { "Format Convert" });
                ui.radio_value(&mut video_settings.batch_operation_type, "compress".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "视频压缩" } else { "Video Compress" });
                ui.radio_value(&mut video_settings.batch_operation_type, "resize".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "尺寸调整" } else { "Resize" });
            });
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut video_settings.batch_operation_type, "rotate".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "视频旋转" } else { "Video Rotate" });
                ui.radio_value(&mut video_settings.batch_operation_type, "crop".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "视频裁剪" } else { "Video Crop" });
                ui.radio_value(&mut video_settings.batch_operation_type, "filter".to_string(), 
                    if translations.language == crate::language::Language::Chinese { "视频滤镜" } else { "Video Filter" });
            });
            
            ui.add_space(10.0);
            
            // Show current settings info based on selected operation
            ui.group(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "当前设置预览:"
                } else {
                    "Current Settings Preview:"
                });
                
                match video_settings.batch_operation_type.as_str() {
                    "convert" => {
                        ui.label(format!("📄 {} → {}", 
                            if translations.language == crate::language::Language::Chinese { "输出格式" } else { "Output Format" },
                            video_settings.container_format
                        ));
                        ui.label(format!("📄 {} → {}", 
                            if translations.language == crate::language::Language::Chinese { "编码器" } else { "Codec" },
                            video_settings.codec
                        ));
                    },
                    "compress" => {
                        ui.label(format!("🎯 CRF: {}", video_settings.crf));
                        if video_settings.target_size_mb > 0 {
                            ui.label(format!("📏 {} {} MB", 
                                if translations.language == crate::language::Language::Chinese { "目标大小:" } else { "Target Size:" },
                                video_settings.target_size_mb
                            ));
                        }
                    },
                    "resize" => {
                        if let (Some(w), Some(h)) = (video_settings.width, video_settings.height) {
                            ui.label(format!("📐 {} {}x{}", 
                                if translations.language == crate::language::Language::Chinese { "新尺寸:" } else { "New Size:" },
                                w, h
                            ));
                        } else {
                            ui.label(if translations.language == crate::language::Language::Chinese {
                                "⚠ 请先在\"视频尺寸调整\"中设置目标尺寸"
                            } else {
                                "⚠ Please set target size in \"Video Resize\" first"
                            });
                        }
                    },
                    "rotate" => {
                        if video_settings.use_custom_rotation {
                            ui.label(format!("🔄 {} {}°", 
                                if translations.language == crate::language::Language::Chinese { "自定义角度:" } else { "Custom Angle:" },
                                video_settings.custom_rotation_angle
                            ));
                        } else {
                            ui.label(format!("🔄 {} {}°", 
                                if translations.language == crate::language::Language::Chinese { "旋转角度:" } else { "Rotation:" },
                                video_settings.rotation
                            ));
                        }
                        if video_settings.flip_horizontal || video_settings.flip_vertical {
                            let flip_text = match (video_settings.flip_horizontal, video_settings.flip_vertical) {
                                (true, true) => if translations.language == crate::language::Language::Chinese { "水平+垂直翻转" } else { "Flip H+V" },
                                (true, false) => if translations.language == crate::language::Language::Chinese { "水平翻转" } else { "Flip H" },
                                (false, true) => if translations.language == crate::language::Language::Chinese { "垂直翻转" } else { "Flip V" },
                                _ => ""
                            };
                            ui.label(format!("↔️ {}", flip_text));
                        }
                    },
                    "crop" => {
                        let has_crop = video_settings.crop_top.is_some() || video_settings.crop_bottom.is_some() || 
                                      video_settings.crop_left.is_some() || video_settings.crop_right.is_some();
                        if has_crop {
                            ui.label(format!("✂️ {} T:{} B:{} L:{} R:{}", 
                                if translations.language == crate::language::Language::Chinese { "裁剪:" } else { "Crop:" },
                                video_settings.crop_top.unwrap_or(0),
                                video_settings.crop_bottom.unwrap_or(0),
                                video_settings.crop_left.unwrap_or(0),
                                video_settings.crop_right.unwrap_or(0)
                            ));
                        } else {
                            ui.label(if translations.language == crate::language::Language::Chinese {
                                "⚠ 请先在\"视频裁剪\"中设置裁剪参数"
                            } else {
                                "⚠ Please set crop parameters in \"Video Crop\" first"
                            });
                        }
                    },
                    "filter" => {
                        let filters = [
                            (video_settings.denoise, if translations.language == crate::language::Language::Chinese { "降噪" } else { "Denoise" }),
                            (video_settings.deinterlace, if translations.language == crate::language::Language::Chinese { "去隔行" } else { "Deinterlace" }),
                            (video_settings.stabilize, if translations.language == crate::language::Language::Chinese { "视频稳定" } else { "Stabilize" }),
                        ].iter().filter(|(enabled, _)| *enabled).map(|(_, name)| *name).collect::<Vec<_>>();
                        
                        if !filters.is_empty() {
                            ui.label(format!("🎨 {}: {}", 
                                if translations.language == crate::language::Language::Chinese { "滤镜" } else { "Filters" },
                                filters.join(", ")
                            ));
                        }
                        
                        if video_settings.brightness != 0.0 || video_settings.contrast != 1.0 || video_settings.saturation != 1.0 {
                            ui.label(format!("⚡ {} B:{:.1} C:{:.1} S:{:.1}", 
                                if translations.language == crate::language::Language::Chinese { "色彩调整:" } else { "Color Adjust:" },
                                video_settings.brightness, video_settings.contrast, video_settings.saturation
                            ));
                        }
                        
                        if filters.is_empty() && video_settings.brightness == 0.0 && video_settings.contrast == 1.0 && video_settings.saturation == 1.0 {
                            ui.label(if translations.language == crate::language::Language::Chinese {
                                "⚠ 请先在\"视频滤镜\"中设置滤镜参数"
                            } else {
                                "⚠ Please set filter parameters in \"Video Filter\" first"
                            });
                        }
                    },
                    _ => {}
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "输出命名模式:"
                } else {
                    "Output Naming Pattern:"
                });
                ui.text_edit_singleline(&mut video_settings.batch_naming_pattern);
            });
            
            ui.label(if translations.language == crate::language::Language::Chinese {
                "支持的变量: {name}, {index}, {date}"
            } else {
                "Supported variables: {name}, {index}, {date}"
            });
        });
    }
    
    // Add Subtitle - Subtitle embedding
    fn show_add_subtitle(ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "📝 添加字幕"
            } else {
                "📝 Add Subtitle"
            });
            ui.separator();
            
            // Subtitle file selector with drag-and-drop support
            ui.group(|ui| {
                ui.label("📝 Subtitle File (Required):");
                
                // Create drag-and-drop area for subtitle file
                let subtitle_area_height = 60.0;
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), subtitle_area_height),
                    egui::Sense::click()
                );
                
                // Handle drag and drop for subtitle file
                let is_being_dragged = !ctx.input(|i| i.raw.hovered_files.is_empty());
                let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
                
                // Draw background
                let bg_color = if can_accept_drop {
                    egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
                } else if settings.subtitle_file.is_empty() {
                    egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
                } else {
                    egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
                };
                
                ui.painter().rect_filled(rect, 5.0, bg_color);
                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                
                // Handle click to open file dialog
                if response.clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Subtitle files", &["srt", "ass", "vtt", "sub"])
                        .pick_file() {
                        settings.subtitle_file = path.display().to_string();
                    }
                }
                
                // Process dropped files for subtitle
                if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                    let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                    if let Some(file) = dropped_files.first() {
                        if let Some(path) = &file.path {
                            let ext = path.extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if ["srt", "ass", "vtt", "sub"].contains(&ext.as_str()) {
                                settings.subtitle_file = path.display().to_string();
                            }
                        }
                    }
                }
                
                // Draw text in the drop area
                let text = if settings.subtitle_file.is_empty() {
                    if can_accept_drop {
                        "📝 Drop subtitle file here"
                    } else {
                        "📝 Click to select subtitle file or drag & drop here"
                    }
                } else {
                    // Show file name
                    let file_name = std::path::Path::new(&settings.subtitle_file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    &format!("📝 {}", file_name)
                };
                
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::default(),
                    if settings.subtitle_file.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
                );
                
                ui.add_space(5.0);
                
                // Show controls and status
                ui.horizontal(|ui| {
                    if !settings.subtitle_file.is_empty() {
                        if ui.button("🚮 Clear").clicked() {
                            settings.subtitle_file.clear();
                        }
                        if ui.button("📁 Change").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Subtitle files", &["srt", "ass", "vtt", "sub"])
                                .pick_file() {
                                settings.subtitle_file = path.display().to_string();
                            }
                        }
                    }
                });
                
                // Show status for subtitle file
                if settings.subtitle_file.is_empty() {
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                        "⚠ Please select a subtitle file first");
                } else {
                    let is_subtitle = settings.subtitle_file.to_lowercase().ends_with(".srt") ||
                                     settings.subtitle_file.to_lowercase().ends_with(".ass") ||
                                     settings.subtitle_file.to_lowercase().ends_with(".vtt") ||
                                     settings.subtitle_file.to_lowercase().ends_with(".sub");
                    
                    if is_subtitle {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ Subtitle file ready");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                            "⚠ Selected file is not a subtitle file");
                    }
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(translations.subtitle_mode());
                ui.radio_value(&mut settings.subtitle_mode, "soft".to_string(), translations.soft_subtitle());
                ui.radio_value(&mut settings.subtitle_mode, "hard".to_string(), translations.hard_subtitle());
            });
            
            if settings.subtitle_mode == "hard" {
                ui.add_space(5.0);
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "💡 硬字幕会永久嵌入视频，无法关闭"
                } else {
                    "💡 Hard subtitles are permanently embedded and cannot be turned off"
                });
            }
            
            ui.add_space(10.0);
            
            if settings.subtitle_mode == "hard" {
                ui.collapsing(if translations.language == crate::language::Language::Chinese {
                    "🎨 字体样式设置"
                } else {
                    "🎨 Font Style Settings"
                }, |ui| {
                    
                    // Font family dropdown
                    ui.horizontal(|ui| {
                        ui.label(translations.font_family());
                        egui::ComboBox::from_label("")
                            .selected_text(&settings.subtitle_font_family)
                            .show_ui(ui, |ui| {
                                let fonts = ["Arial", "Times New Roman", "Helvetica", "Verdana", "Calibri", "Tahoma", "Georgia", "Impact", "Comic Sans MS"];
                                for font in fonts {
                                    ui.selectable_value(&mut settings.subtitle_font_family, font.to_string(), font);
                                }
                            });
                    });
                    
                    // Font size
                    ui.horizontal(|ui| {
                        ui.label(translations.font_size());
                        ui.add(egui::Slider::new(&mut settings.subtitle_font_size, 8..=72).step_by(1.0));
                    });
                    
                    // Font color dropdown
                    ui.horizontal(|ui| {
                        ui.label(translations.font_color());
                        egui::ComboBox::from_label("")
                            .selected_text(&settings.subtitle_font_color)
                            .show_ui(ui, |ui| {
                                let colors = [("white", "白色"), ("black", "黑色"), ("red", "红色"), ("blue", "蓝色"), ("green", "绿色"), ("yellow", "黄色")];
                                for (color_en, color_zh) in colors {
                                    let display_name = if translations.language == crate::language::Language::Chinese { color_zh } else { color_en };
                                    ui.selectable_value(&mut settings.subtitle_font_color, color_en.to_string(), display_name);
                                }
                            });
                    });
                    
                    // Outline color
                    ui.horizontal(|ui| {
                        ui.label(translations.outline_color());
                        egui::ComboBox::from_label("")
                            .selected_text(&settings.subtitle_outline_color)
                            .show_ui(ui, |ui| {
                                let colors = [("black", "黑色"), ("white", "白色"), ("gray", "灰色"), ("none", "无")];
                                for (color_en, color_zh) in colors {
                                    let display_name = if translations.language == crate::language::Language::Chinese { color_zh } else { color_en };
                                    ui.selectable_value(&mut settings.subtitle_outline_color, color_en.to_string(), display_name);
                                }
                            });
                    });
                    
                    // Background color
                    ui.horizontal(|ui| {
                        ui.label(translations.background_color());
                        egui::ComboBox::from_label("")
                            .selected_text(&settings.subtitle_background_color)
                            .show_ui(ui, |ui| {
                                let colors = [("transparent", "透明"), ("black", "黑色"), ("white", "白色"), ("gray", "灰色")];
                                for (color_en, color_zh) in colors {
                                    let display_name = if translations.language == crate::language::Language::Chinese { color_zh } else { color_en };
                                    ui.selectable_value(&mut settings.subtitle_background_color, color_en.to_string(), display_name);
                                }
                            });
                    });
                    
                    // Alignment
                    ui.horizontal(|ui| {
                        ui.label(translations.alignment());
                        egui::ComboBox::from_label("")
                            .selected_text(&settings.subtitle_alignment)
                            .show_ui(ui, |ui| {
                                let alignments = [("center", "居中"), ("left", "左对齐"), ("right", "右对齐")];
                                for (align_en, align_zh) in alignments {
                                    let display_name = if translations.language == crate::language::Language::Chinese { align_zh } else { align_en };
                                    ui.selectable_value(&mut settings.subtitle_alignment, align_en.to_string(), display_name);
                                }
                            });
                    });
                });
            }
        });
    }
    
    // Add Watermark - Watermark overlay
    fn show_add_watermark(ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "💧 添加水印"
            } else {
                "💧 Add Watermark"
            });
            ui.separator();
            
            // Watermark file selector with drag-and-drop support
            ui.group(|ui| {
                ui.label("💧 Watermark Image File (Required):");
                
                // Create drag-and-drop area for watermark file
                let watermark_area_height = 60.0;
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), watermark_area_height),
                    egui::Sense::click()
                );
                
                // Handle drag and drop for watermark file
                let is_being_dragged = !ctx.input(|i| i.raw.hovered_files.is_empty());
                let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
                
                // Draw background
                let bg_color = if can_accept_drop {
                    egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
                } else if settings.watermark_file.is_empty() {
                    egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
                } else {
                    egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
                };
                
                ui.painter().rect_filled(rect, 5.0, bg_color);
                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                
                // Handle click to open file dialog
                if response.clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Image files", &["png", "jpg", "jpeg", "bmp", "gif"])
                        .pick_file() {
                        settings.watermark_file = path.display().to_string();
                    }
                }
                
                // Process dropped files for watermark
                if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                    let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
                    if let Some(file) = dropped_files.first() {
                        if let Some(path) = &file.path {
                            let ext = path.extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            if ["png", "jpg", "jpeg", "bmp", "gif"].contains(&ext.as_str()) {
                                settings.watermark_file = path.display().to_string();
                            }
                        }
                    }
                }
                
                // Draw text in the drop area
                let text = if settings.watermark_file.is_empty() {
                    if can_accept_drop {
                        "💧 Drop image file here"
                    } else {
                        "💧 Click to select image file or drag & drop here"
                    }
                } else {
                    // Show file name
                    let file_name = std::path::Path::new(&settings.watermark_file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    &format!("💧 {}", file_name)
                };
                
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::default(),
                    if settings.watermark_file.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
                );
                
                ui.add_space(5.0);
                
                // Show controls and status
                ui.horizontal(|ui| {
                    if !settings.watermark_file.is_empty() {
                        if ui.button("🚮 Clear").clicked() {
                            settings.watermark_file.clear();
                        }
                        if ui.button("📁 Change").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Image files", &["png", "jpg", "jpeg", "bmp", "gif"])
                                .pick_file() {
                                settings.watermark_file = path.display().to_string();
                            }
                        }
                    }
                });
                
                // Show status for watermark file
                if settings.watermark_file.is_empty() {
                    ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                        "⚠ Please select an image file first");
                } else {
                    let is_image = settings.watermark_file.to_lowercase().ends_with(".png") ||
                                   settings.watermark_file.to_lowercase().ends_with(".jpg") ||
                                   settings.watermark_file.to_lowercase().ends_with(".jpeg") ||
                                   settings.watermark_file.to_lowercase().ends_with(".bmp") ||
                                   settings.watermark_file.to_lowercase().ends_with(".gif");
                    
                    if is_image {
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ Image file ready");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                            "⚠ Selected file is not an image file");
                    }
                }
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(translations.position());
                egui::ComboBox::from_label("")
                    .selected_text(&settings.watermark_position)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.watermark_position, "top-left".to_string(), translations.top_left());
                        ui.selectable_value(&mut settings.watermark_position, "top-right".to_string(), translations.top_right());
                        ui.selectable_value(&mut settings.watermark_position, "bottom-left".to_string(), translations.bottom_left());
                        ui.selectable_value(&mut settings.watermark_position, "bottom-right".to_string(), translations.bottom_right());
                        ui.selectable_value(&mut settings.watermark_position, "center".to_string(), translations.center());
                    });
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label(translations.opacity());
                let opacity_percent = settings.watermark_opacity * 100.0;
                ui.add(egui::Slider::new(&mut settings.watermark_opacity, 0.0..=1.0)
                    .text(format!("{:.1}%", opacity_percent))
                    .step_by(0.1));
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.scale());
                let scale_value = settings.watermark_scale;
                ui.add(egui::Slider::new(&mut settings.watermark_scale, 0.1..=2.0)
                    .text(format!("{:.1}x", scale_value))
                    .step_by(0.1));
            });
            
            ui.add_space(5.0);
            ui.label(if translations.language == crate::language::Language::Chinese {
                "💡 PNG格式支持透明背景，效果最佳"
            } else {
                "💡 PNG format supports transparency for best results"
            });
        });
    }
    
    // Frame Extract - Extract frames from video
    fn show_frame_extract(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🖼 提取视频帧"
            } else {
                "🖼 Extract Video Frames"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(translations.extract_mode());
                ui.radio_value(&mut settings.frame_extract_mode, "all".to_string(), translations.all_frames());
                ui.radio_value(&mut settings.frame_extract_mode, "interval".to_string(), translations.every_n_frames());
                ui.radio_value(&mut settings.frame_extract_mode, "time".to_string(), translations.time_range());
            });
            
            ui.add_space(10.0);
            
            match settings.frame_extract_mode.as_str() {
                "interval" => {
                    ui.horizontal(|ui| {
                        ui.label(translations.frame_interval());
                        ui.add(egui::DragValue::new(&mut settings.frame_interval)
                            .range(1..=3600)
                            .suffix(" frames"));
                    });
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "每隔N帧提取一张图片"
                    } else {
                        "Extract one frame every N frames"
                    });
                }
                "time" => {
                    ui.horizontal(|ui| {
                        ui.label(translations.start_time());
                        ui.text_edit_singleline(&mut settings.frame_start_time);
                        ui.label("(HH:MM:SS)");
                    });
                    ui.horizontal(|ui| {
                        ui.label(translations.end_time());
                        ui.text_edit_singleline(&mut settings.frame_end_time);
                        ui.label("(HH:MM:SS, 可选)");
                    });
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "从指定时间范围提取帧"
                    } else {
                        "Extract frames from specified time range"
                    });
                }
                _ => {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "提取视频中的所有帧"
                    } else {
                        "Extract all frames from the video"
                    });
                }
            }
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label(translations.output_format());
                ui.radio_value(&mut settings.frame_format, "png".to_string(), "PNG");
                ui.radio_value(&mut settings.frame_format, "jpg".to_string(), "JPEG");
                ui.radio_value(&mut settings.frame_format, "bmp".to_string(), "BMP");
            });
            
            if settings.frame_format == "jpg" {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(translations.image_quality());
                    let quality_value = settings.frame_quality;
                    ui.add(egui::Slider::new(&mut settings.frame_quality, 1..=31)
                        .text(format!("{}", quality_value))
                        .step_by(1.0));
                });
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "数值越小质量越高 (1=最高质量, 31=最低质量)"
                } else {
                    "Lower values = higher quality (1=highest, 31=lowest)"
                });
            }
            
            ui.add_space(10.0);
            ui.label(if translations.language == crate::language::Language::Chinese {
                "💡 输出文件将保存为: frame_001.png, frame_002.png..."
            } else {
                "💡 Output files will be saved as: frame_001.png, frame_002.png..."
            });
        });
    }
    
    // Video to GIF conversion
    fn show_video_to_gif(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🎥 视频转GIF"
            } else {
                "🎥 Video to GIF"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_fps());
                let fps_value = settings.gif_fps;
                ui.add(egui::Slider::new(&mut settings.gif_fps, 1.0..=30.0)
                    .text(format!("{:.1} fps", fps_value))
                    .step_by(0.5));
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_scale());
                let scale_value = settings.gif_scale;
                ui.add(egui::Slider::new(&mut settings.gif_scale, 0.1..=2.0)
                    .text(format!("{:.1}x", scale_value))
                    .step_by(0.1));
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_colors());
                ui.add(egui::Slider::new(&mut settings.gif_colors, 2..=256)
                    .step_by(1.0));
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_dither());
                egui::ComboBox::from_label("")
                    .selected_text(&settings.gif_dither)
                    .show_ui(ui, |ui| {
                        let dithers = if translations.language == crate::language::Language::Chinese {
                            [("none", "无"), ("bayer", "拜耳抖动"), ("floyd_steinberg", "Floyd-Steinberg抖动")]
                        } else {
                            [("none", "None"), ("bayer", "Bayer"), ("floyd_steinberg", "Floyd-Steinberg")]
                        };
                        for (value, display) in dithers {
                            ui.selectable_value(&mut settings.gif_dither, value.to_string(), display);
                        }
                    });
            });
            
            ui.checkbox(&mut settings.gif_loop, translations.gif_loop());
            ui.checkbox(&mut settings.gif_optimize, translations.gif_optimize());
            
            ui.add_space(5.0);
            ui.label(if translations.language == crate::language::Language::Chinese {
                "💡 降低帧率和颜色数量可减小文件大小"
            } else {
                "💡 Lower frame rate and color count reduce file size"
            });
        });
    }
    
    // GIF resize
    fn show_gif_resize(ui: &mut egui::Ui, settings: &mut VideoSettings, translations: &Translations) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔄 GIF尺寸调整"
            } else {
                "🔄 GIF Resize"
            });
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label(translations.width());
                if let Some(ref mut width) = settings.width {
                    ui.add(egui::DragValue::new(width).range(1..=4096));
                } else {
                    settings.width = Some(320);
                }
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.height());
                if let Some(ref mut height) = settings.height {
                    ui.add(egui::DragValue::new(height).range(1..=4096));
                } else {
                    settings.height = Some(240);
                }
            });
            
            ui.checkbox(&mut settings.maintain_aspect_ratio, if translations.language == crate::language::Language::Chinese {
                "保持宽高比"
            } else {
                "Maintain aspect ratio"
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_scale());
                let scale_value = settings.gif_scale;
                ui.add(egui::Slider::new(&mut settings.gif_scale, 0.1..=5.0)
                    .text(format!("{:.1}x", scale_value))
                    .step_by(0.1));
            });
            
            ui.separator();
            ui.label(if translations.language == crate::language::Language::Chinese {
                "🗜️ 文件大小优化设置"
            } else {
                "🗜️ File Size Optimization"
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_colors());
                ui.add(egui::Slider::new(&mut settings.gif_colors, 2..=256)
                    .step_by(1.0));
            });
            
            ui.horizontal(|ui| {
                ui.label(translations.gif_dither());
                egui::ComboBox::from_label("")
                    .selected_text(&settings.gif_dither)
                    .show_ui(ui, |ui| {
                        let dithers = if translations.language == crate::language::Language::Chinese {
                            [("none", "无"), ("bayer", "拜耳抖动"), ("floyd_steinberg", "Floyd-Steinberg抖动")]
                        } else {
                            [("none", "None"), ("bayer", "Bayer"), ("floyd_steinberg", "Floyd-Steinberg")]
                        };
                        for (value, display) in dithers {
                            ui.selectable_value(&mut settings.gif_dither, value.to_string(), display);
                        }
                    });
            });
            
            ui.checkbox(&mut settings.gif_optimize, translations.gif_optimize());
            
            ui.add_space(5.0);
            ui.label(if translations.language == crate::language::Language::Chinese {
                "💡 缩放比例会覆盖宽高设置\n🎨 减少颜色数量和启用优化可显著减小文件大小"
            } else {
                "💡 Scale ratio overrides width/height settings\n🎨 Reducing colors and enabling optimization significantly reduces file size"
            });
        });
    }
}