use crate::comprehensive_codec_registry::*;
use crate::app_state::*;
use crate::language::*;
use egui;
use std::collections::HashMap;

/// Comprehensive UI components that use the full codec registry
pub struct ComprehensiveUIComponents;

impl ComprehensiveUIComponents {
    /// Show comprehensive codec selector with categorized display and hardware compatibility filtering
    pub fn show_comprehensive_codec_selector(
        ui: &mut egui::Ui, 
        codec_type: CodecType,
        current_codec: &mut String,
        selected_format: &str,
        translations: &Translations,
        cached_hw_encoders: &[String]  // Use async hardware detection like original UI
    ) {
        let codecs = match codec_type {
            CodecType::Video => ComprehensiveCodecRegistry::get_video_codecs(),
            CodecType::Audio => ComprehensiveCodecRegistry::get_audio_codecs(),
            CodecType::Subtitle => HashMap::new(), // TODO: Implement subtitle codecs
        };
        
        // Check if current codec is compatible with selected format
        if !selected_format.is_empty() && !current_codec.is_empty() && current_codec != "auto" {
            if !ComprehensiveCodecRegistry::is_compatible(current_codec, selected_format) {
                // Auto switch to a compatible codec
                let (recommended_video, recommended_audio) = ComprehensiveCodecRegistry::get_recommended_codecs_for_format(selected_format);
                let recommended = if codec_type == CodecType::Video { recommended_video } else { recommended_audio };
                if !recommended.is_empty() {
                    *current_codec = recommended[0].clone();
                } else {
                    *current_codec = "auto".to_string();
                }
            }
        }

        egui::ComboBox::from_id_salt(format!("{:?}_codec_comprehensive", codec_type))
            .selected_text(Self::get_codec_display_name(current_codec, &codecs))
            .width(ui.available_width().min(500.0).max(300.0))  // Adaptive width
            .show_ui(ui, |ui| {
                ui.set_min_width(400.0);
                ui.set_max_width(600.0);
                
                // Auto option first
                Self::show_codec_option(ui, current_codec, "auto", &translations.auto_recommended(), None, translations);
                ui.separator();
                
                if codec_type == CodecType::Video {
                    // Video codecs - follow original UI pattern
                    
                    // Software encoders first
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        "🎬 软件编码器"
                    } else {
                        "🎬 Software Encoders"
                    });
                    
                    // Only show codecs compatible with selected format
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libx264", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libx264", "🎥 H.264 (libx264)", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libx265", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libx265", "🎥 H.265 (libx265)", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libvpx", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libvpx", "🌐 VP8 (libvpx)", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libvpx-vp9", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libvpx-vp9", "🌐 VP9 (libvpx-vp9)", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libaom-av1", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libaom-av1", "🚀 AV1 (libaom-av1)", None, translations);
                    }
                    
                    // Legacy codecs for specific formats
                    if selected_format == "wmv" {
                        Self::show_codec_option(ui, current_codec, "wmv2", "📼 WMV2 (Windows Media)", None, translations);
                    }
                    if selected_format == "flv" {
                        Self::show_codec_option(ui, current_codec, "flv1", "📼 FLV1 (Flash Video)", None, translations);
                    }
                    
                    ui.separator();
                    
                    // Hardware encoders - using async detection pattern
                    if !cached_hw_encoders.is_empty() {
                        ui.label(if translations.language == crate::language::Language::Chinese {
                            "⚡ 硬件加速编码器"
                        } else {
                            "⚡ Hardware Accelerated Encoders"
                        });
                        
                        let nvenc_encoders: Vec<_> = cached_hw_encoders.iter().filter(|e| e.contains("nvenc")).collect();
                        let qsv_encoders: Vec<_> = cached_hw_encoders.iter().filter(|e| e.contains("qsv")).collect();
                        let amf_encoders: Vec<_> = cached_hw_encoders.iter().filter(|e| e.contains("amf")).collect();
                        let vaapi_encoders: Vec<_> = cached_hw_encoders.iter().filter(|e| e.contains("vaapi")).collect();
                        let videotoolbox_encoders: Vec<_> = cached_hw_encoders.iter().filter(|e| e.contains("videotoolbox")).collect();
                        
                        // NVIDIA NVENC
                        for encoder in nvenc_encoders {
                            if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible(encoder, selected_format) {
                                let display_name = match encoder.as_str() {
                                    "h264_nvenc" => format!("⚡ H.264 NVENC {}", translations.codec_hardware()),
                                    "hevc_nvenc" => format!("⚡ H.265 NVENC {}", translations.codec_hardware()),
                                    "av1_nvenc" => format!("⚡ AV1 NVENC {}", translations.codec_hardware()),
                                    "vp9_nvenc" => format!("⚡ VP9 NVENC {}", translations.codec_hardware()),
                                    _ => format!("⚡ {} NVENC {}", encoder.replace("_nvenc", "").to_uppercase(), translations.codec_hardware()),
                                };
                                Self::show_codec_option(ui, current_codec, encoder, &display_name, None, translations);
                            }
                        }
                        
                        // Intel QuickSync
                        for encoder in qsv_encoders {
                            if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible(encoder, selected_format) {
                                let display_name = match encoder.as_str() {
                                    "h264_qsv" => format!("💨 H.264 QuickSync {}", translations.codec_hardware()),
                                    "hevc_qsv" => format!("💨 H.265 QuickSync {}", translations.codec_hardware()),
                                    "av1_qsv" => format!("💨 AV1 QuickSync {}", translations.codec_hardware()),
                                    "vp9_qsv" => format!("💨 VP9 QuickSync {}", translations.codec_hardware()),
                                    _ => format!("💨 {} QuickSync {}", encoder.replace("_qsv", "").to_uppercase(), translations.codec_hardware()),
                                };
                                Self::show_codec_option(ui, current_codec, encoder, &display_name, None, translations);
                            }
                        }
                        
                        // AMD AMF
                        for encoder in amf_encoders {
                            if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible(encoder, selected_format) {
                                let display_name = match encoder.as_str() {
                                    "h264_amf" => format!("🔥 H.264 AMF {}", translations.codec_hardware()),
                                    "hevc_amf" => format!("🔥 H.265 AMF {}", translations.codec_hardware()),
                                    "av1_amf" => format!("🔥 AV1 AMF {}", translations.codec_hardware()),
                                    _ => format!("🔥 {} AMF {}", encoder.replace("_amf", "").to_uppercase(), translations.codec_hardware()),
                                };
                                Self::show_codec_option(ui, current_codec, encoder, &display_name, None, translations);
                            }
                        }
                        
                        // VA-API (Linux)
                        for encoder in vaapi_encoders {
                            if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible(encoder, selected_format) {
                                let display_name = match encoder.as_str() {
                                    "h264_vaapi" => format!("🐧 H.264 VA-API {}", translations.codec_hardware()),
                                    "hevc_vaapi" => format!("🐧 H.265 VA-API {}", translations.codec_hardware()),
                                    "av1_vaapi" => format!("🐧 AV1 VA-API {}", translations.codec_hardware()),
                                    "vp8_vaapi" => format!("🐧 VP8 VA-API {}", translations.codec_hardware()),
                                    "vp9_vaapi" => format!("🐧 VP9 VA-API {}", translations.codec_hardware()),
                                    _ => format!("🐧 {} VA-API {}", encoder.replace("_vaapi", "").to_uppercase(), translations.codec_hardware()),
                                };
                                Self::show_codec_option(ui, current_codec, encoder, &display_name, None, translations);
                            }
                        }
                        
                        // Apple VideoToolbox
                        for encoder in videotoolbox_encoders {
                            if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible(encoder, selected_format) {
                                let display_name = match encoder.as_str() {
                                    "h264_videotoolbox" => format!("🍎 H.264 VideoToolbox {}", translations.codec_hardware()),
                                    "hevc_videotoolbox" => format!("🍎 H.265 VideoToolbox {}", translations.codec_hardware()),
                                    "prores_videotoolbox" => format!("🍎 ProRes VideoToolbox {}", translations.codec_hardware()),
                                    _ => format!("🍎 {} VideoToolbox {}", encoder.replace("_videotoolbox", "").to_uppercase(), translations.codec_hardware()),
                                };
                                Self::show_codec_option(ui, current_codec, encoder, &display_name, None, translations);
                            }
                        }
                    } else {
                        let no_hw_text = if translations.language == crate::language::Language::Chinese {
                            "⚠ 未检测到硬件编码器"
                        } else {
                            "⚠ No hardware encoders detected"
                        };
                        ui.label(no_hw_text);
                    }
                } else {
                    // Audio codecs - simple list like video codecs
                    
                    // High Quality Audio
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("aac", selected_format) {
                        Self::show_codec_option(ui, current_codec, "aac", "🎵 AAC", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libfdk_aac", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libfdk_aac", "🎵 AAC (FDK)", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libmp3lame", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libmp3lame", "🎵 MP3", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libopus", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libopus", "🎵 Opus", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("libvorbis", selected_format) {
                        Self::show_codec_option(ui, current_codec, "libvorbis", "🎵 Vorbis", None, translations);
                    }
                    
                    ui.separator();
                    
                    // Lossless Audio
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("flac", selected_format) {
                        Self::show_codec_option(ui, current_codec, "flac", "💎 FLAC", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("alac", selected_format) {
                        Self::show_codec_option(ui, current_codec, "alac", "💎 ALAC", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("pcm_s16le", selected_format) {
                        Self::show_codec_option(ui, current_codec, "pcm_s16le", "💎 PCM 16-bit", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("pcm_s24le", selected_format) {
                        Self::show_codec_option(ui, current_codec, "pcm_s24le", "💎 PCM 24-bit", None, translations);
                    }
                    if selected_format.is_empty() || ComprehensiveCodecRegistry::is_compatible("pcm_f32le", selected_format) {
                        Self::show_codec_option(ui, current_codec, "pcm_f32le", "💎 PCM 32-bit", None, translations);
                    }
                    
                    // Legacy audio codecs
                    if selected_format == "wma" {
                        Self::show_codec_option(ui, current_codec, "wmav2", "🎵 WMA v2", None, translations);
                    }
                    if selected_format == "avi" || selected_format == "mkv" || selected_format == "mp4" || selected_format == "mov" {
                        Self::show_codec_option(ui, current_codec, "ac3", "🔊 AC-3 (Dolby Digital)", None, translations);
                    }
                }
            });
        
        // Show codec information panel
        if !current_codec.is_empty() && current_codec != "auto" {
            if let Some(codec_info) = codecs.get(current_codec) {
                Self::show_codec_info_panel(ui, codec_info, translations);
            }
        }
    }

    /// Show comprehensive format selector with categorized display
    pub fn show_comprehensive_format_selector(
        ui: &mut egui::Ui,
        current_format: &mut String,
        format_purpose: FormatPurpose,
        _translations: &Translations
    ) {
        let formats = ComprehensiveCodecRegistry::get_container_formats();
        
        egui::ComboBox::from_id_salt(format!("{:?}_format_comprehensive", format_purpose))
            .selected_text(Self::get_format_display_name(current_format, &formats))
            .width(ui.available_width().min(400.0).max(250.0))  // Adaptive width
            .show_ui(ui, |ui| {
                ui.set_min_width(300.0);
                ui.set_max_width(400.0);
                
                // Show formats in simple list like video codec
                if format_purpose == FormatPurpose::Video || format_purpose == FormatPurpose::Any {
                    // Video formats
                    Self::show_simple_format_option(ui, current_format, "mp4", "📹 .mp4");
                    Self::show_simple_format_option(ui, current_format, "mkv", "📹 .mkv");
                    Self::show_simple_format_option(ui, current_format, "avi", "📹 .avi");
                    Self::show_simple_format_option(ui, current_format, "mov", "📹 .mov");
                    Self::show_simple_format_option(ui, current_format, "webm", "🌐 .webm");
                    Self::show_simple_format_option(ui, current_format, "flv", "📹 .flv");
                    Self::show_simple_format_option(ui, current_format, "wmv", "📹 .wmv");
                    Self::show_simple_format_option(ui, current_format, "3gp", "📱 .3gp");
                    Self::show_simple_format_option(ui, current_format, "ts", "📺 .ts");
                    Self::show_simple_format_option(ui, current_format, "m4v", "📹 .m4v");
                }
                
                if format_purpose == FormatPurpose::Audio || format_purpose == FormatPurpose::Any {
                    if format_purpose == FormatPurpose::Any {
                        ui.separator();
                    }
                    // Audio formats
                    Self::show_simple_format_option(ui, current_format, "mp3", "🎵 .mp3");
                    Self::show_simple_format_option(ui, current_format, "aac", "🎵 .aac");
                    Self::show_simple_format_option(ui, current_format, "m4a", "🎵 .m4a");
                    Self::show_simple_format_option(ui, current_format, "wav", "🎵 .wav");
                    Self::show_simple_format_option(ui, current_format, "flac", "💎 .flac");
                    Self::show_simple_format_option(ui, current_format, "ogg", "🎵 .ogg");
                    Self::show_simple_format_option(ui, current_format, "wma", "🎵 .wma");
                    Self::show_simple_format_option(ui, current_format, "opus", "🎵 .opus");
                }
            });
    }
    
    /// Show simple format option (like codec option)
    fn show_simple_format_option(
        ui: &mut egui::Ui,
        current_format: &mut String,
        format_name: &str,
        display_name: &str
    ) {
        let is_selected = current_format == format_name;
        let response = ui.selectable_label(is_selected, display_name);
        
        if response.clicked() {
            *current_format = format_name.to_string();
        }
    }

    /// Show advanced codec settings based on selected codec
    pub fn show_advanced_codec_settings(
        ui: &mut egui::Ui,
        codec_name: &str,
        settings: &mut VideoSettings, // or AudioSettings
        _translations: &Translations
    ) {
        let video_codecs = ComprehensiveCodecRegistry::get_video_codecs();
        let audio_codecs = ComprehensiveCodecRegistry::get_audio_codecs();
        
        let codec_info = video_codecs.get(codec_name)
            .or_else(|| audio_codecs.get(codec_name));
        
        if let Some(codec_info) = codec_info {
            ui.collapsing(format!("⚙️ Advanced {} Settings", codec_info.display_name), |ui| {
                // Quality/CRF settings
                if let Some((min_quality, max_quality)) = codec_info.quality_range {
                    ui.horizontal(|ui| {
                        ui.label(if codec_info.codec_type == CodecType::Video {
                            "Quality (CRF):"
                        } else {
                            "Quality:"
                        });
                        ui.add(egui::Slider::new(&mut settings.crf, min_quality as i32..=max_quality as i32)
                            .step_by(1.0)
                            .suffix(if codec_info.codec_type == CodecType::Video {
                                " (lower = better)"
                            } else {
                                ""
                            }));
                    });
                }
                
                // Preset settings
                if !codec_info.preset_options.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Preset:");
                        egui::ComboBox::from_id_salt(format!("{}_preset", codec_name))
                            .selected_text(&settings.preset)
                            .show_ui(ui, |ui| {
                                for preset in &codec_info.preset_options {
                                    ui.selectable_value(&mut settings.preset, preset.clone(), preset);
                                }
                            });
                    });
                }
                
                // Profile settings
                if !codec_info.profile_options.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Profile:");
                        egui::ComboBox::from_id_salt(format!("{}_profile", codec_name))
                            .selected_text(&settings.profile)
                            .show_ui(ui, |ui| {
                                for profile in &codec_info.profile_options {
                                    ui.selectable_value(&mut settings.profile, profile.clone(), profile);
                                }
                            });
                    });
                }
                
                // Level settings
                if !codec_info.level_options.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Level:");
                        egui::ComboBox::from_id_salt(format!("{}_level", codec_name))
                            .selected_text(&settings.level)
                            .show_ui(ui, |ui| {
                                for level in &codec_info.level_options {
                                    ui.selectable_value(&mut settings.level, level.clone(), level);
                                }
                            });
                    });
                }
                
                // Bitrate settings
                if !codec_info.supported_bit_rates.is_empty() && codec_info.supports_cbr {
                    ui.horizontal(|ui| {
                        ui.label("Target Bitrate:");
                        egui::ComboBox::from_id_salt(format!("{}_bitrate", codec_name))
                            .selected_text(&settings.bitrate)
                            .show_ui(ui, |ui| {
                                for bitrate in &codec_info.supported_bit_rates {
                                    ui.selectable_value(&mut settings.bitrate, bitrate.clone(), bitrate);
                                }
                            });
                    });
                }
                
                // Hardware acceleration info
                if codec_info.supports_hardware {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 100),
                        "⚡ This codec supports hardware acceleration"
                    );
                }
                
                // Pixel format settings for video codecs
                if codec_info.codec_type == CodecType::Video && !codec_info.supported_pixel_formats.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Pixel Format:");
                        egui::ComboBox::from_id_salt(format!("{}_pixel_format", codec_name))
                            .selected_text(&settings.pixel_format)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut settings.pixel_format, "auto".to_string(), "Auto");
                                ui.separator();
                                for pix_fmt in &codec_info.supported_pixel_formats {
                                    ui.selectable_value(&mut settings.pixel_format, pix_fmt.clone(), pix_fmt);
                                }
                            });
                    });
                }
            });
        }
    }

    /// Show compatibility matrix between selected codec and format
    pub fn show_codec_format_compatibility_matrix(
        ui: &mut egui::Ui,
        current_codec: &str,
        current_format: &str,
        translations: &Translations
    ) {
        ui.group(|ui| {
            ui.heading(if translations.language == crate::language::Language::Chinese {
                "🔗 编码器-格式兼容性检查"
            } else {
                "🔗 Codec-Format Compatibility"
            });
            
            if current_codec.is_empty() || current_format.is_empty() || current_codec == "auto" {
                ui.label(if translations.language == crate::language::Language::Chinese {
                    "选择编码器和格式以查看兼容性信息"
                } else {
                    "Select both codec and format to see compatibility information"
                });
                return;
            }
            
            let is_compatible = ComprehensiveCodecRegistry::is_compatible(current_codec, current_format);
            
            if is_compatible {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 100),
                    if translations.language == crate::language::Language::Chinese {
                        format!("✅ {} 与 .{} 格式兼容", current_codec, current_format)
                    } else {
                        format!("✅ {} is compatible with .{} format", current_codec, current_format)
                    }
                );
                
                // Show additional compatibility info
                let formats = ComprehensiveCodecRegistry::get_container_formats();
                if let Some(format_info) = formats.get(current_format) {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        format!("📋 容器特性: {}", format_info.container_features.join(", "))
                    } else {
                        format!("📋 Features: {}", format_info.container_features.join(", "))
                    });
                }
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    if translations.language == crate::language::Language::Chinese {
                        format!("❌ {} 与 .{} 格式不兼容", current_codec, current_format)
                    } else {
                        format!("❌ {} is NOT compatible with .{} format", current_codec, current_format)
                    }
                );
                
                // Suggest compatible formats
                let compatibility = ComprehensiveCodecRegistry::get_codec_format_compatibility();
                if let Some(compatible_formats) = compatibility.get(current_codec) {
                    ui.label(if translations.language == crate::language::Language::Chinese {
                        format!("💡 建议的格式: .{}", compatible_formats.join(", ."))
                    } else {
                        format!("💡 Suggested formats: .{}", compatible_formats.join(", ."))
                    });
                }
            }
        });
    }

    // Helper functions
    
    fn group_codecs_by_category(codecs: &HashMap<String, CodecInfo>) -> Vec<(CodecCategory, Vec<String>)> {
        let mut by_category: HashMap<CodecCategory, Vec<String>> = HashMap::new();
        
        for (codec_name, codec_info) in codecs {
            by_category.entry(codec_info.category.clone())
                .or_insert_with(Vec::new)
                .push(codec_name.clone());
        }
        
        // Sort categories by priority
        let mut result: Vec<_> = by_category.into_iter().collect();
        result.sort_by_key(|(category, _)| Self::get_category_priority(category));
        
        // Sort codecs within each category
        for (_, codecs) in &mut result {
            codecs.sort();
        }
        
        result
    }
    
    fn group_formats_by_category(formats: &HashMap<String, FormatInfo>, purpose: FormatPurpose) -> Vec<(FormatCategory, Vec<String>)> {
        let mut by_category: HashMap<FormatCategory, Vec<String>> = HashMap::new();
        
        for (format_name, format_info) in formats {
            // Filter by purpose
            if Self::format_matches_purpose(format_info, purpose) {
                by_category.entry(format_info.category.clone())
                    .or_insert_with(Vec::new)
                    .push(format_name.clone());
            }
        }
        
        let mut result: Vec<_> = by_category.into_iter().collect();
        result.sort_by_key(|(category, _)| Self::get_format_category_priority(category));
        
        for (_, formats) in &mut result {
            formats.sort();
        }
        
        result
    }
    
    fn show_codec_option(
        ui: &mut egui::Ui,
        current_codec: &mut String,
        codec_name: &str,
        display_name: &str,
        codec_info: Option<&CodecInfo>,
        _translations: &Translations
    ) {
        let is_selected = current_codec == codec_name;
        
        let mut rich_text = egui::RichText::new(display_name);
        if is_selected {
            rich_text = rich_text.color(egui::Color32::from_rgb(100, 200, 255));
        }
        
        let mut response = ui.selectable_label(is_selected, rich_text);
        
        if let Some(info) = codec_info {
            response = response.on_hover_text(&info.description);
        }
        
        if response.clicked() {
            *current_codec = codec_name.to_string();
        }
        
        // Show codec badges
        if let Some(info) = codec_info {
            ui.horizontal(|ui| {
                if info.supports_hardware {
                    ui.small_button("⚡ HW");
                }
                if info.supports_vbr {
                    ui.small_button("📊 VBR");
                }
                if codec_name.contains("lossless") || info.default_bitrate == "lossless" {
                    ui.small_button("💎 Lossless");
                }
            });
        }
    }
    
    fn show_format_option(
        ui: &mut egui::Ui,
        current_format: &mut String,
        format_name: &str,
        format_info: &FormatInfo,
        _translations: &Translations
    ) {
        let is_selected = current_format == format_name;
        
        let mut rich_text = egui::RichText::new(&format!(".{}", format_info.extension));
        if is_selected {
            rich_text = rich_text.color(egui::Color32::from_rgb(100, 200, 255));
        }
        
        let response = ui.selectable_label(is_selected, rich_text);
        let response = response.on_hover_text(&format_info.description);
        
        if response.clicked() {
            *current_format = format_name.to_string();
        }
        
        // Show format badges
        ui.horizontal(|ui| {
            if format_info.supports_subtitles {
                ui.small_button("📝 Subs");
            }
            if format_info.supports_chapters {
                ui.small_button("📚 Chapters");
            }
            if format_info.supports_attachments {
                ui.small_button("📎 Attachments");
            }
        });
    }
    
    fn show_codec_info_panel(ui: &mut egui::Ui, codec_info: &CodecInfo, _translations: &Translations) {
        ui.group(|ui| {
            ui.heading(format!("ℹ️ {} Information", codec_info.display_name));
            
            ui.label(&codec_info.description);
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Supported formats:");
                ui.label(codec_info.supported_formats.join(", "));
            });
            
            if !codec_info.supported_bit_rates.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Bitrates:");
                    ui.label(format!("{} options", codec_info.supported_bit_rates.len()));
                });
            }
            
            if codec_info.supports_hardware {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "⚡ Hardware acceleration supported");
            }
        });
    }
    
    fn show_format_info_panel(ui: &mut egui::Ui, format_info: &FormatInfo, _translations: &Translations) {
        ui.group(|ui| {
            ui.heading(format!("ℹ️ {} Information", format_info.display_name));
            
            ui.label(&format_info.description);
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("MIME type:");
                ui.label(&format_info.mime_type);
            });
            
            ui.horizontal(|ui| {
                ui.label("Preferred video codecs:");
                ui.label(format_info.preferred_video_codecs.join(", "));
            });
            
            ui.horizontal(|ui| {
                ui.label("Preferred audio codecs:");
                ui.label(format_info.preferred_audio_codecs.join(", "));
            });
            
            ui.horizontal(|ui| {
                ui.label("Features:");
                ui.label(format_info.container_features.join(", "));
            });
        });
    }
    
    fn show_format_codec_compatibility_info(ui: &mut egui::Ui, current_format: &str, _translations: &Translations) {
        if current_format.is_empty() {
            return;
        }
        
        let (recommended_video, recommended_audio) = ComprehensiveCodecRegistry::get_recommended_codecs_for_format(current_format);
        
        if !recommended_video.is_empty() || !recommended_audio.is_empty() {
            ui.group(|ui| {
                ui.heading("💡 Recommended Codecs");
                
                if !recommended_video.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Video:");
                        ui.label(recommended_video.join(", "));
                    });
                }
                
                if !recommended_audio.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Audio:");
                        ui.label(recommended_audio.join(", "));
                    });
                }
            });
        }
    }
    
    fn get_codec_display_name(codec_name: &str, codecs: &HashMap<String, CodecInfo>) -> String {
        if codec_name == "auto" {
            "🤖 Auto (Recommended)".to_string()
        } else if let Some(info) = codecs.get(codec_name) {
            info.display_name.clone()
        } else {
            codec_name.to_string()
        }
    }
    
    fn get_format_display_name(format_name: &str, _formats: &HashMap<String, FormatInfo>) -> String {
        if format_name.is_empty() {
            "Choose Format".to_string()
        } else {
            // Simple display: add emoji based on format type and show extension
            match format_name {
                // Video formats
                "mp4" | "mkv" | "avi" | "mov" | "flv" | "wmv" | "m4v" => format!("📹 .{}", format_name),
                "webm" => "🌐 .webm".to_string(),
                "3gp" => "📱 .3gp".to_string(),
                "ts" => "📺 .ts".to_string(),
                // Audio formats
                "mp3" | "aac" | "m4a" | "wav" | "ogg" | "wma" | "opus" => format!("🎵 .{}", format_name),
                "flac" => "💎 .flac".to_string(),
                // Default
                _ => format!(".{}", format_name),
            }
        }
    }
    
    fn get_category_display_name(category: &CodecCategory, translations: &Translations) -> String {
        match category {
            CodecCategory::H264Family => format!("🎬 {}", translations.codec_category_h264_family()),
            CodecCategory::H265Family => format!("🎬 {}", translations.codec_category_h265_family()),
            CodecCategory::VP8VP9Family => format!("🌐 {}", translations.codec_category_vp8vp9_family()),
            CodecCategory::AV1Family => format!("🚀 {}", translations.codec_category_av1_family()),
            CodecCategory::HardwareAccelerated => format!("⚡ {}", translations.codec_category_hardware_encoders()),
            CodecCategory::LegacyVideo => format!("📼 {}", translations.codec_category_legacy_video()),
            CodecCategory::LosslessVideo => "💎 Lossless Video".to_string(), // Keep original for now
            CodecCategory::MP3Family => "🎵 MP3 Family".to_string(), // Keep original for now
            CodecCategory::AACFamily => "🎵 AAC Family".to_string(), // Keep original for now
            CodecCategory::VorbisFamily => "🎵 Vorbis Family".to_string(), // Keep original for now
            CodecCategory::OpusFamily => "🎵 Opus Family".to_string(), // Keep original for now
            CodecCategory::LosslessAudio => format!("💎 {}", translations.codec_category_lossless_audio()),
            CodecCategory::ProfessionalAudio => format!("🎧 {}", translations.codec_category_high_quality_audio()),
            CodecCategory::SpeechCodecs => format!("🗣️ {}", translations.codec_category_speech_codecs()),
            CodecCategory::LegacyAudio => format!("📻 {}", translations.codec_category_legacy_audio()),
        }
    }
    
    fn get_format_category_display_name(category: &FormatCategory, translations: &Translations) -> String {
        match category {
            FormatCategory::ModernVideo => format!("🎬 {}", translations.format_category_modern_video()),
            FormatCategory::WebOptimized => format!("🌐 {}", translations.format_category_web_optimized()),
            FormatCategory::Professional => format!("🎯 {}", translations.format_category_professional()),
            FormatCategory::LegacyVideo => format!("📼 {}", translations.format_category_legacy_video()),
            FormatCategory::Broadcast => "📺 Broadcast".to_string(), // Keep original for now
            FormatCategory::LosslessAudio => format!("💎 {}", translations.format_category_lossless_audio()),
            FormatCategory::CompressedAudio => format!("🎵 {}", translations.format_category_compressed_audio()),
            FormatCategory::ProfessionalAudio => format!("🎧 {}", translations.format_category_high_quality_audio()),
            FormatCategory::WebAudio => "🌐 Web Audio".to_string(), // Keep original for now
            FormatCategory::LegacyAudio => "📻 Legacy Audio".to_string(), // Keep original for now
            FormatCategory::StreamingFormats => "📡 Streaming".to_string(), // Keep original for now
            FormatCategory::ArchivalFormats => "🗄️ Archival".to_string(), // Keep original for now
        }
    }
    
    fn get_category_priority(category: &CodecCategory) -> u8 {
        match category {
            CodecCategory::H264Family => 1,
            CodecCategory::H265Family => 2,
            CodecCategory::AV1Family => 3,
            CodecCategory::VP8VP9Family => 4,
            CodecCategory::HardwareAccelerated => 5,
            CodecCategory::AACFamily => 10,
            CodecCategory::MP3Family => 11,
            CodecCategory::OpusFamily => 12,
            CodecCategory::VorbisFamily => 13,
            CodecCategory::LosslessAudio => 20,
            CodecCategory::ProfessionalAudio => 21,
            CodecCategory::LosslessVideo => 30,
            CodecCategory::SpeechCodecs => 40,
            CodecCategory::LegacyVideo => 50,
            CodecCategory::LegacyAudio => 51,
        }
    }
    
    fn get_format_category_priority(category: &FormatCategory) -> u8 {
        match category {
            FormatCategory::ModernVideo => 1,
            FormatCategory::WebOptimized => 2,
            FormatCategory::Professional => 3,
            FormatCategory::CompressedAudio => 10,
            FormatCategory::LosslessAudio => 11,
            FormatCategory::ProfessionalAudio => 12,
            FormatCategory::WebAudio => 13,
            FormatCategory::Broadcast => 20,
            FormatCategory::StreamingFormats => 30,
            FormatCategory::ArchivalFormats => 40,
            FormatCategory::LegacyVideo => 50,
            FormatCategory::LegacyAudio => 51,
        }
    }
    
    fn format_matches_purpose(format_info: &FormatInfo, purpose: FormatPurpose) -> bool {
        match purpose {
            FormatPurpose::Video => !format_info.preferred_video_codecs.is_empty(),
            FormatPurpose::Audio => format_info.preferred_video_codecs.is_empty(),
            FormatPurpose::Any => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatPurpose {
    Video,
    Audio,
    Any,
}