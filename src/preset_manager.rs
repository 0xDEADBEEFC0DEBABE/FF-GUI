use serde::{Serialize, Deserialize};
use crate::app_state::*;
use crate::codec_manager::*;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingPreset {
    pub name: String,
    pub description: String,
    pub category: PresetCategory,
    pub video_settings: VideoSettings,
    pub audio_settings: AudioSettings,
    pub recommended_formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresetCategory {
    WebOptimized,
    HighQuality,
    FastEncoding,
    MobileOptimized,
    Archive,
    Streaming,
    Custom,
}

impl PresetCategory {
    pub fn display_name(&self, translations: &crate::language::Translations) -> &'static str {
        match self {
            PresetCategory::WebOptimized => {
                if translations.language == crate::language::Language::Chinese {
                    "网络优化"
                } else {
                    "Web Optimized"
                }
            },
            PresetCategory::HighQuality => {
                if translations.language == crate::language::Language::Chinese {
                    "高质量"
                } else {
                    "High Quality"
                }
            },
            PresetCategory::FastEncoding => {
                if translations.language == crate::language::Language::Chinese {
                    "快速编码"
                } else {
                    "Fast Encoding"
                }
            },
            PresetCategory::MobileOptimized => {
                if translations.language == crate::language::Language::Chinese {
                    "移动设备优化"
                } else {
                    "Mobile Optimized"
                }
            },
            PresetCategory::Archive => {
                if translations.language == crate::language::Language::Chinese {
                    "存档压缩"
                } else {
                    "Archive"
                }
            },
            PresetCategory::Streaming => {
                if translations.language == crate::language::Language::Chinese {
                    "流媒体"
                } else {
                    "Streaming"
                }
            },
            PresetCategory::Custom => {
                if translations.language == crate::language::Language::Chinese {
                    "自定义"
                } else {
                    "Custom"
                }
            },
        }
    }
}

pub struct PresetManager;

impl PresetManager {
    pub fn get_builtin_presets() -> Vec<EncodingPreset> {
        vec![
            EncodingPreset {
                name: "YouTube 1080p".to_string(),
                description: "Optimized settings for YouTube 1080p uploads".to_string(),
                category: PresetCategory::WebOptimized,
                video_settings: VideoSettings {
                    codec: "libx264".to_string(),
                    preset: "slow".to_string(),
                    profile: "high".to_string(),
                    tune: "film".to_string(),
                    quality: 21,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (1920, 1080),
                    use_hardware_acceleration: false,
                    custom_args: "-movflags +faststart -maxrate 8M -bufsize 12M".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "192k".to_string(),
                    sample_rate: "48000".to_string(),
                    channels: "2".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string()],
            },
            
            EncodingPreset {
                name: "Web H.265 4K".to_string(),
                description: "Efficient 4K web video using H.265 encoding".to_string(),
                category: PresetCategory::WebOptimized,
                video_settings: VideoSettings {
                    codec: "libx265".to_string(),
                    preset: "medium".to_string(),
                    profile: "main".to_string(),
                    tune: "auto".to_string(),
                    quality: 24,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (3840, 2160),
                    use_hardware_acceleration: false,
                    custom_args: "-movflags +faststart -tag:v hvc1".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "256k".to_string(),
                    sample_rate: "48000".to_string(),
                    channels: "2".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string()],
            },

            EncodingPreset {
                name: "Master Quality".to_string(),
                description: "Highest quality settings for master storage".to_string(),
                category: PresetCategory::HighQuality,
                video_settings: VideoSettings {
                    codec: "libx264".to_string(),
                    preset: "veryslow".to_string(),
                    profile: "high".to_string(),
                    tune: "film".to_string(),
                    quality: 16,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (0, 0),
                    use_hardware_acceleration: false,
                    custom_args: "-movflags +faststart".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "flac".to_string(),
                    bitrate: "lossless".to_string(),
                    sample_rate: "48000".to_string(),
                    channels: "auto".to_string(),
                    volume: 1.0,
                    quality: "8".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mkv".to_string(), "mov".to_string()],
            },

            EncodingPreset {
                name: "ProRes Proxy".to_string(),
                description: "Professional proxy files suitable for editing".to_string(),
                category: PresetCategory::HighQuality,
                video_settings: VideoSettings {
                    codec: "prores_ks".to_string(),
                    preset: "auto".to_string(),
                    profile: "proxy".to_string(),
                    tune: "auto".to_string(),
                    quality: 0,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (0, 0),
                    use_hardware_acceleration: false,
                    custom_args: "-profile:v 0".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "pcm_s16le".to_string(),
                    bitrate: "lossless".to_string(),
                    sample_rate: "48000".to_string(),
                    channels: "auto".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mov".to_string()],
            },

            EncodingPreset {
                name: "Fast H.264".to_string(),
                description: "Fast H.264 encoding for batch file conversion".to_string(),
                category: PresetCategory::FastEncoding,
                video_settings: VideoSettings {
                    codec: "libx264".to_string(),
                    preset: "ultrafast".to_string(),
                    profile: "main".to_string(),
                    tune: "fastdecode".to_string(),
                    quality: 28,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (0, 0),
                    use_hardware_acceleration: true,
                    custom_args: "-movflags +faststart".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "128k".to_string(),
                    sample_rate: "44100".to_string(),
                    channels: "auto".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string()],
            },

            EncodingPreset {
                name: "NVENC Fast".to_string(),
                description: "NVIDIA hardware accelerated fast encoding".to_string(),
                category: PresetCategory::FastEncoding,
                video_settings: VideoSettings {
                    codec: "h264_nvenc".to_string(),
                    preset: "fast".to_string(),
                    profile: "high".to_string(),
                    tune: "auto".to_string(),
                    quality: 25,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (0, 0),
                    use_hardware_acceleration: true,
                    custom_args: "-rc vbr_hq -surfaces 32".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "128k".to_string(),
                    sample_rate: "48000".to_string(),
                    channels: "auto".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string()],
            },

            EncodingPreset {
                name: "Mobile H.264".to_string(),
                description: "Mobile device compatible H.264 encoding".to_string(),
                category: PresetCategory::MobileOptimized,
                video_settings: VideoSettings {
                    codec: "libx264".to_string(),
                    preset: "medium".to_string(),
                    profile: "baseline".to_string(),
                    tune: "fastdecode".to_string(),
                    quality: 26,
                    bitrate: "auto".to_string(),
                    fps: "30".to_string(),
                    resolution: (1280, 720),
                    use_hardware_acceleration: false,
                    custom_args: "-movflags +faststart -level 3.1".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "96k".to_string(),
                    sample_rate: "44100".to_string(),
                    channels: "2".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string()],
            },

            EncodingPreset {
                name: "Archive H.265".to_string(),
                description: "High compression archive settings using H.265".to_string(),
                category: PresetCategory::Archive,
                video_settings: VideoSettings {
                    codec: "libx265".to_string(),
                    preset: "veryslow".to_string(),
                    profile: "main".to_string(),
                    tune: "auto".to_string(),
                    quality: 28,
                    bitrate: "auto".to_string(),
                    fps: "auto".to_string(),
                    resolution: (0, 0),
                    use_hardware_acceleration: false,
                    custom_args: "-x265-params log-level=error".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "96k".to_string(),
                    sample_rate: "44100".to_string(),
                    channels: "auto".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string(), "mkv".to_string()],
            },

            EncodingPreset {
                name: "Twitch Stream".to_string(),
                description: "Recommended settings for Twitch streaming".to_string(),
                category: PresetCategory::Streaming,
                video_settings: VideoSettings {
                    codec: "libx264".to_string(),
                    preset: "veryfast".to_string(),
                    profile: "main".to_string(),
                    tune: "zerolatency".to_string(),
                    quality: 0,
                    bitrate: "6000k".to_string(),
                    fps: "60".to_string(),
                    resolution: (1920, 1080),
                    use_hardware_acceleration: false,
                    custom_args: "-keyint_min 60 -g 120".to_string(),
                    ..Default::default()
                },
                audio_settings: AudioSettings {
                    codec: "aac".to_string(),
                    bitrate: "160k".to_string(),
                    sample_rate: "44100".to_string(),
                    channels: "2".to_string(),
                    volume: 1.0,
                    quality: "auto".to_string(),
                    custom_args: String::new(),
                    ..Default::default()
                },
                recommended_formats: vec!["mp4".to_string(), "flv".to_string()],
            },
        ]
    }

    pub fn get_presets_by_category(category: &PresetCategory) -> Vec<EncodingPreset> {
        Self::get_builtin_presets()
            .into_iter()
            .filter(|preset| std::mem::discriminant(&preset.category) == std::mem::discriminant(category))
            .collect()
    }

    pub fn get_all_categories() -> Vec<PresetCategory> {
        vec![
            PresetCategory::WebOptimized,
            PresetCategory::HighQuality,
            PresetCategory::FastEncoding,
            PresetCategory::MobileOptimized,
            PresetCategory::Archive,
            PresetCategory::Streaming,
            PresetCategory::Custom,
        ]
    }

    pub fn recommend_presets_for_format(format: &str) -> Vec<EncodingPreset> {
        Self::get_builtin_presets()
            .into_iter()
            .filter(|preset| preset.recommended_formats.contains(&format.to_string()))
            .collect()
    }

    pub fn create_custom_preset(
        name: String,
        description: String,
        video_settings: VideoSettings,
        audio_settings: AudioSettings,
    ) -> EncodingPreset {
        EncodingPreset {
            name,
            description,
            category: PresetCategory::Custom,
            video_settings,
            audio_settings,
            recommended_formats: vec!["mp4".to_string()],
        }
    }

    pub fn save_custom_preset(preset: &EncodingPreset, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(preset)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    pub fn load_custom_preset(filename: &str) -> Result<EncodingPreset> {
        let json = std::fs::read_to_string(filename)?;
        let preset: EncodingPreset = serde_json::from_str(&json)?;
        Ok(preset)
    }

    pub fn validate_preset(preset: &EncodingPreset) -> Result<Vec<String>> {
        let mut warnings = Vec::new();
        
        if let Err(e) = CodecManager::validate_codec_format_compatibility(
            &preset.video_settings.codec, 
            &preset.recommended_formats.first().unwrap_or(&"mp4".to_string()), 
            false
        ) {
            warnings.push(format!("Video codec issue: {}", e));
        }
        
        if let Err(e) = CodecManager::validate_codec_format_compatibility(
            &preset.audio_settings.codec, 
            &preset.recommended_formats.first().unwrap_or(&"mp4".to_string()), 
            true
        ) {
            warnings.push(format!("Audio codec issue: {}", e));
        }
        
        if let Ok(sample_rate) = preset.audio_settings.sample_rate.parse::<u32>() {
            if let Err(e) = CodecManager::validate_sample_rate(&preset.audio_settings.codec, sample_rate) {
                warnings.push(format!("Sample rate issue: {}", e));
            }
        }
        
        if let Err(e) = CodecManager::validate_bitrate(&preset.audio_settings.codec, &preset.audio_settings.bitrate) {
            warnings.push(format!("Bitrate issue: {}", e));
        }
        
        Ok(warnings)
    }

    pub fn apply_preset_to_settings(
        preset: &EncodingPreset,
        video_settings: &mut VideoSettings,
        audio_settings: &mut AudioSettings,
    ) {
        *video_settings = preset.video_settings.clone();
        *audio_settings = preset.audio_settings.clone();
    }
}