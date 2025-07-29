use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Comprehensive codec and format registry for FFmpeg
/// This module provides complete support for all major FFmpeg codecs and container formats

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub codec_type: CodecType,
    pub supported_formats: Vec<String>,
    pub supported_sample_rates: Vec<u32>,
    pub supported_bit_rates: Vec<String>,
    pub supported_pixel_formats: Vec<String>,
    pub supports_vbr: bool,
    pub supports_cbr: bool,
    pub supports_hardware: bool,
    pub default_bitrate: String,
    pub default_sample_rate: u32,
    pub quality_range: Option<(u32, u32)>,
    pub preset_options: Vec<String>,
    pub profile_options: Vec<String>,
    pub level_options: Vec<String>,
    pub category: CodecCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub extension: String,
    pub display_name: String,
    pub description: String,
    pub mime_type: String,
    pub preferred_audio_codecs: Vec<String>,
    pub preferred_video_codecs: Vec<String>,
    pub supported_audio_codecs: Vec<String>,
    pub supported_video_codecs: Vec<String>,
    pub container_features: Vec<String>,
    pub max_streams: Option<u32>,
    pub supports_chapters: bool,
    pub supports_metadata: bool,
    pub supports_subtitles: bool,
    pub supports_attachments: bool,
    pub category: FormatCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodecType {
    Audio,
    Video,
    Subtitle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CodecCategory {
    // Video categories
    H264Family,
    H265Family,
    VP8VP9Family,
    AV1Family,
    HardwareAccelerated,
    LegacyVideo,
    LosslessVideo,
    
    // Audio categories
    MP3Family,
    AACFamily,
    VorbisFamily,
    OpusFamily,
    LosslessAudio,
    ProfessionalAudio,
    SpeechCodecs,
    LegacyAudio,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FormatCategory {
    // Video container categories
    ModernVideo,
    WebOptimized,
    Professional,
    LegacyVideo,
    Broadcast,
    
    // Audio container categories
    LosslessAudio,
    CompressedAudio,
    ProfessionalAudio,
    WebAudio,
    LegacyAudio,
    
    // Specialized
    StreamingFormats,
    ArchivalFormats,
}

pub struct ComprehensiveCodecRegistry;

impl ComprehensiveCodecRegistry {
    /// Get all video codecs with comprehensive support
    pub fn get_video_codecs() -> HashMap<String, CodecInfo> {
        let mut codecs = HashMap::new();
        
        // === H.264 Family ===
        codecs.insert("libx264".to_string(), CodecInfo {
            name: "libx264".to_string(),
            display_name: "H.264 / AVC (x264)".to_string(),
            description: "High-quality H.264 encoder with excellent compression".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string(), "m2ts".to_string(), "3gp".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["100k".to_string(), "250k".to_string(), "500k".to_string(), "1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "5M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string(), "20M".to_string(), "25M".to_string(), "30M".to_string(), "50M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuvj420p".to_string(), "yuv422p".to_string(), "yuv444p".to_string(), "yuv420p10le".to_string(), "yuv422p10le".to_string(), "yuv444p10le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
            preset_options: vec!["ultrafast".to_string(), "superfast".to_string(), "veryfast".to_string(), "faster".to_string(), "fast".to_string(), "medium".to_string(), "slow".to_string(), "slower".to_string(), "veryslow".to_string(), "placebo".to_string()],
            profile_options: vec!["baseline".to_string(), "main".to_string(), "high".to_string(), "high10".to_string(), "high422".to_string(), "high444".to_string()],
            level_options: vec!["3.0".to_string(), "3.1".to_string(), "3.2".to_string(), "4.0".to_string(), "4.1".to_string(), "4.2".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string()],
            category: CodecCategory::H264Family,
        });

        codecs.insert("h264_nvenc".to_string(), CodecInfo {
            name: "h264_nvenc".to_string(),
            display_name: "H.264 NVENC (NVIDIA Hardware)".to_string(),
            description: "NVIDIA hardware-accelerated H.264 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string(), "20M".to_string(), "25M".to_string(), "30M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "nv12".to_string(), "p010le".to_string(), "yuv444p".to_string(), "p016le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "4M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
            preset_options: vec!["default".to_string(), "slow".to_string(), "medium".to_string(), "fast".to_string(), "hp".to_string(), "hq".to_string(), "bd".to_string(), "llhq".to_string(), "llhp".to_string(), "p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string(), "p6".to_string(), "p7".to_string()],
            profile_options: vec!["baseline".to_string(), "main".to_string(), "high".to_string(), "high444p".to_string()],
            level_options: vec!["auto".to_string(), "3.0".to_string(), "3.1".to_string(), "3.2".to_string(), "4.0".to_string(), "4.1".to_string(), "4.2".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        codecs.insert("h264_qsv".to_string(), CodecInfo {
            name: "h264_qsv".to_string(),
            display_name: "H.264 QuickSync (Intel Hardware)".to_string(),
            description: "Intel QuickSync hardware-accelerated H.264 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string(), "20M".to_string()],
            supported_pixel_formats: vec!["nv12".to_string(), "qsv".to_string(), "yuv420p".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "4M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
            preset_options: vec!["veryfast".to_string(), "faster".to_string(), "fast".to_string(), "medium".to_string(), "slow".to_string(), "slower".to_string(), "veryslow".to_string()],
            profile_options: vec!["baseline".to_string(), "main".to_string(), "high".to_string()],
            level_options: vec!["auto".to_string(), "3.0".to_string(), "3.1".to_string(), "3.2".to_string(), "4.0".to_string(), "4.1".to_string(), "4.2".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        // === H.265 Family ===
        codecs.insert("libx265".to_string(), CodecInfo {
            name: "libx265".to_string(),
            display_name: "H.265 / HEVC (x265)".to_string(),
            description: "Next-generation H.265 encoder with superior compression".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string(), "m2ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["100k".to_string(), "200k".to_string(), "500k".to_string(), "1M".to_string(), "1.5M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "5M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string(), "20M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuvj420p".to_string(), "yuv422p".to_string(), "yuv444p".to_string(), "yuv420p10le".to_string(), "yuv422p10le".to_string(), "yuv444p10le".to_string(), "yuv420p12le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "1.5M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
            preset_options: vec!["ultrafast".to_string(), "superfast".to_string(), "veryfast".to_string(), "faster".to_string(), "fast".to_string(), "medium".to_string(), "slow".to_string(), "slower".to_string(), "veryslow".to_string(), "placebo".to_string()],
            profile_options: vec!["main".to_string(), "main10".to_string(), "main12".to_string(), "main422-10".to_string(), "main422-12".to_string(), "main444-8".to_string(), "main444-10".to_string(), "main444-12".to_string()],
            level_options: vec!["3.0".to_string(), "3.1".to_string(), "4.0".to_string(), "4.1".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string(), "6.0".to_string(), "6.1".to_string(), "6.2".to_string()],
            category: CodecCategory::H265Family,
        });

        codecs.insert("hevc_nvenc".to_string(), CodecInfo {
            name: "hevc_nvenc".to_string(),
            display_name: "H.265 NVENC (NVIDIA Hardware)".to_string(),
            description: "NVIDIA hardware-accelerated H.265 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string(), "20M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "nv12".to_string(), "p010le".to_string(), "yuv444p".to_string(), "p016le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
            preset_options: vec!["default".to_string(), "slow".to_string(), "medium".to_string(), "fast".to_string(), "hp".to_string(), "hq".to_string(), "bd".to_string(), "llhq".to_string(), "llhp".to_string(), "p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string(), "p6".to_string(), "p7".to_string()],
            profile_options: vec!["main".to_string(), "main10".to_string(), "rext".to_string()],
            level_options: vec!["auto".to_string(), "3.0".to_string(), "3.1".to_string(), "4.0".to_string(), "4.1".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string(), "6.0".to_string(), "6.1".to_string(), "6.2".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        codecs.insert("hevc_qsv".to_string(), CodecInfo {
            name: "hevc_qsv".to_string(),
            display_name: "H.265 QuickSync (Intel Hardware)".to_string(),
            description: "Intel QuickSync hardware-accelerated H.265 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string()],
            supported_pixel_formats: vec!["nv12".to_string(), "qsv".to_string(), "p010le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
            preset_options: vec!["veryfast".to_string(), "faster".to_string(), "fast".to_string(), "medium".to_string(), "slow".to_string(), "slower".to_string(), "veryslow".to_string()],
            profile_options: vec!["main".to_string(), "main10".to_string()],
            level_options: vec!["auto".to_string(), "3.0".to_string(), "3.1".to_string(), "4.0".to_string(), "4.1".to_string(), "5.0".to_string(), "5.1".to_string(), "5.2".to_string(), "6.0".to_string(), "6.1".to_string(), "6.2".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        // === VP8/VP9 Family ===
        codecs.insert("libvpx".to_string(), CodecInfo {
            name: "libvpx".to_string(),
            display_name: "VP8".to_string(),
            description: "Google VP8 codec for web video".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["webm".to_string(), "mkv".to_string(), "avi".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "5M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuva420p".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
            preset_options: vec!["realtime".to_string(), "good".to_string(), "best".to_string()],
            profile_options: vec!["0".to_string(), "1".to_string(), "2".to_string(), "3".to_string()],
            level_options: vec![],
            category: CodecCategory::VP8VP9Family,
        });

        codecs.insert("libvpx-vp9".to_string(), CodecInfo {
            name: "libvpx-vp9".to_string(),
            display_name: "VP9".to_string(),
            description: "Google VP9 codec for efficient web video".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "5M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string(), "15M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuva420p".to_string(), "yuv422p".to_string(), "yuv444p".to_string(), "yuv420p10le".to_string(), "yuv422p10le".to_string(), "yuv444p10le".to_string(), "yuv420p12le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
            preset_options: vec!["realtime".to_string(), "good".to_string(), "best".to_string()],
            profile_options: vec!["0".to_string(), "1".to_string(), "2".to_string(), "3".to_string()],
            level_options: vec![],
            category: CodecCategory::VP8VP9Family,
        });

        // === AV1 Family ===
        codecs.insert("libaom-av1".to_string(), CodecInfo {
            name: "libaom-av1".to_string(),
            display_name: "AV1 (libaom)".to_string(),
            description: "Alliance for Open Media AV1 codec - next generation compression".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), "1.5M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "5M".to_string(), "6M".to_string(), "8M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuv422p".to_string(), "yuv444p".to_string(), "yuv420p10le".to_string(), "yuv422p10le".to_string(), "yuv444p10le".to_string(), "yuv420p12le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "1.5M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
            preset_options: vec!["0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(), "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string()],
            profile_options: vec!["main".to_string(), "high".to_string(), "professional".to_string()],
            level_options: vec![],
            category: CodecCategory::AV1Family,
        });

        codecs.insert("av1_nvenc".to_string(), CodecInfo {
            name: "av1_nvenc".to_string(),
            display_name: "AV1 NVENC (NVIDIA Hardware)".to_string(),
            description: "NVIDIA hardware-accelerated AV1 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string(), "10M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "nv12".to_string(), "p010le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 255)),
            preset_options: vec!["p1".to_string(), "p2".to_string(), "p3".to_string(), "p4".to_string(), "p5".to_string(), "p6".to_string(), "p7".to_string()],
            profile_options: vec!["main".to_string(), "high".to_string()],
            level_options: vec![],
            category: CodecCategory::HardwareAccelerated,
        });

        // === AMD AMF Hardware Encoders ===
        codecs.insert("h264_amf".to_string(), CodecInfo {
            name: "h264_amf".to_string(),
            display_name: "H.264 AMF (AMD Hardware)".to_string(),
            description: "AMD hardware-accelerated H.264 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["500k".to_string(), "1M".to_string(), "2M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "nv12".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
            preset_options: vec!["speed".to_string(), "balanced".to_string(), "quality".to_string()],
            profile_options: vec!["baseline".to_string(), "main".to_string(), "high".to_string()],
            level_options: vec!["auto".to_string(), "3.0".to_string(), "3.1".to_string(), "4.0".to_string(), "4.1".to_string(), "5.0".to_string(), "5.1".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        codecs.insert("hevc_amf".to_string(), CodecInfo {
            name: "hevc_amf".to_string(),
            display_name: "H.265 AMF (AMD Hardware)".to_string(),
            description: "AMD hardware-accelerated H.265 encoder".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["500k".to_string(), "1M".to_string(), "2M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuv420p10le".to_string(), "p010le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
            preset_options: vec!["speed".to_string(), "balanced".to_string(), "quality".to_string()],
            profile_options: vec!["main".to_string(), "main10".to_string()],
            level_options: vec!["auto".to_string(), "4.0".to_string(), "4.1".to_string(), "5.0".to_string(), "5.1".to_string(), "6.0".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });

        codecs.insert("av1_amf".to_string(), CodecInfo {
            name: "av1_amf".to_string(),
            display_name: "AV1 AMF (AMD Hardware)".to_string(),
            description: "AMD hardware-accelerated AV1 encoder (RDNA2+ GPUs)".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string(), "yuv420p10le".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 63)),
            preset_options: vec!["speed".to_string(), "balanced".to_string(), "quality".to_string()],
            profile_options: vec!["main".to_string()],
            level_options: vec!["auto".to_string()],
            category: CodecCategory::HardwareAccelerated,
        });
        
        // === Legacy Video Codecs ===
        codecs.insert("wmv2".to_string(), CodecInfo {
            name: "wmv2".to_string(),
            display_name: "WMV2 (Windows Media Video 8)".to_string(),
            description: "Windows Media Video version 8 codec".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["wmv".to_string(), "avi".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["256k".to_string(), "512k".to_string(), "768k".to_string(), "1M".to_string(), "2M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "1M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)),
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::LegacyVideo,
        });
        
        codecs.insert("flv1".to_string(), CodecInfo {
            name: "flv1".to_string(),
            display_name: "FLV1 (Flash Video)".to_string(),
            description: "Sorenson Spark codec used in Flash Video".to_string(),
            codec_type: CodecType::Video,
            supported_formats: vec!["flv".to_string(), "avi".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["256k".to_string(), "384k".to_string(), "512k".to_string(), "768k".to_string(), "1M".to_string()],
            supported_pixel_formats: vec!["yuv420p".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "512k".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)),
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::LegacyVideo,
        });

        codecs
    }

    /// Get all audio codecs with comprehensive support
    pub fn get_audio_codecs() -> HashMap<String, CodecInfo> {
        let mut codecs = HashMap::new();
        
        // === MP3 Family ===
        codecs.insert("libmp3lame".to_string(), CodecInfo {
            name: "libmp3lame".to_string(),
            display_name: "MP3 (LAME)".to_string(),
            description: "High-quality MP3 encoder with wide compatibility".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["mp3".to_string(), "avi".to_string(), "mkv".to_string(), "mp4".to_string(), "mov".to_string()],
            supported_sample_rates: vec![8000, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000],
            supported_bit_rates: vec!["8k".to_string(), "16k".to_string(), "24k".to_string(), "32k".to_string(), "40k".to_string(), "48k".to_string(), "56k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "112k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "224k".to_string(), "256k".to_string(), "320k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 9)),
            preset_options: vec!["insane".to_string(), "extreme".to_string(), "standard".to_string(), "medium".to_string()],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::MP3Family,
        });

        // === AAC Family ===
        codecs.insert("aac".to_string(), CodecInfo {
            name: "aac".to_string(),
            display_name: "AAC (Advanced Audio Coding)".to_string(),
            description: "High-efficiency AAC encoder for modern audio".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["aac".to_string(), "m4a".to_string(), "mp4".to_string(), "mkv".to_string(), "mov".to_string(), "avi".to_string(), "ts".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000],
            supported_bit_rates: vec!["16k".to_string(), "24k".to_string(), "32k".to_string(), "48k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "224k".to_string(), "256k".to_string(), "320k".to_string(), "384k".to_string(), "448k".to_string(), "512k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((1, 5)),
            preset_options: vec![],
            profile_options: vec!["aac_low".to_string(), "aac_he".to_string(), "aac_he_v2".to_string(), "aac_ld".to_string(), "aac_eld".to_string()],
            level_options: vec![],
            category: CodecCategory::AACFamily,
        });

        codecs.insert("libfdk_aac".to_string(), CodecInfo {
            name: "libfdk_aac".to_string(),
            display_name: "AAC (FDK)".to_string(),
            description: "High-quality AAC encoder using Fraunhofer FDK library".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["aac".to_string(), "m4a".to_string(), "mp4".to_string(), "mkv".to_string(), "mov".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000],
            supported_bit_rates: vec!["16k".to_string(), "24k".to_string(), "32k".to_string(), "48k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "224k".to_string(), "256k".to_string(), "320k".to_string(), "384k".to_string(), "448k".to_string(), "512k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((1, 5)),
            preset_options: vec![],
            profile_options: vec!["aac_low".to_string(), "aac_he".to_string(), "aac_he_v2".to_string(), "aac_ld".to_string(), "aac_eld".to_string()],
            level_options: vec![],
            category: CodecCategory::AACFamily,
        });

        // === Opus Family ===
        codecs.insert("libopus".to_string(), CodecInfo {
            name: "libopus".to_string(),
            display_name: "Opus".to_string(),
            description: "Modern, low-latency audio codec with excellent quality".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["opus".to_string(), "ogg".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![8000, 12000, 16000, 24000, 48000],
            supported_bit_rates: vec!["6k".to_string(), "8k".to_string(), "12k".to_string(), "16k".to_string(), "24k".to_string(), "32k".to_string(), "48k".to_string(), "64k".to_string(), "96k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "256k".to_string(), "320k".to_string(), "450k".to_string(), "510k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 48000,
            quality_range: Some((0, 10)),
            preset_options: vec![],
            profile_options: vec!["voip".to_string(), "audio".to_string(), "restricted_lowdelay".to_string()],
            level_options: vec![],
            category: CodecCategory::OpusFamily,
        });

        // === Vorbis Family ===
        codecs.insert("libvorbis".to_string(), CodecInfo {
            name: "libvorbis".to_string(),
            display_name: "Vorbis".to_string(),
            description: "Open-source audio codec with good compression".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["ogg".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000],
            supported_bit_rates: vec!["32k".to_string(), "48k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "112k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "224k".to_string(), "256k".to_string(), "320k".to_string(), "384k".to_string(), "448k".to_string(), "500k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 10)),
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::VorbisFamily,
        });

        // === Lossless Audio ===
        codecs.insert("flac".to_string(), CodecInfo {
            name: "flac".to_string(),
            display_name: "FLAC (Free Lossless Audio Codec)".to_string(),
            description: "Open-source lossless audio compression".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["flac".to_string(), "mkv".to_string(), "ogg".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 12)),
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::LosslessAudio,
        });

        codecs.insert("alac".to_string(), CodecInfo {
            name: "alac".to_string(),
            display_name: "ALAC (Apple Lossless Audio Codec)".to_string(),
            description: "Apple's lossless audio codec".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["m4a".to_string(), "mp4".to_string(), "mov".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::LosslessAudio,
        });

        // === PCM Family (Professional Audio) ===
        codecs.insert("pcm_s16le".to_string(), CodecInfo {
            name: "pcm_s16le".to_string(),
            display_name: "PCM 16-bit (WAV)".to_string(),
            description: "Uncompressed 16-bit PCM audio".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::ProfessionalAudio,
        });

        codecs.insert("pcm_s24le".to_string(), CodecInfo {
            name: "pcm_s24le".to_string(),
            display_name: "PCM 24-bit (WAV)".to_string(),
            description: "Uncompressed 24-bit PCM audio for professional use".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::ProfessionalAudio,
        });

        codecs.insert("pcm_s32le".to_string(), CodecInfo {
            name: "pcm_s32le".to_string(),
            display_name: "PCM 32-bit (WAV)".to_string(),
            description: "Uncompressed 32-bit PCM audio for high-end professional use".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::ProfessionalAudio,
        });

        codecs.insert("pcm_f32le".to_string(), CodecInfo {
            name: "pcm_f32le".to_string(),
            display_name: "PCM 32-bit Float (WAV)".to_string(),
            description: "Uncompressed 32-bit floating-point PCM audio".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: false,
            supports_hardware: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::ProfessionalAudio,
        });
        
        // === Legacy Audio Codecs ===
        codecs.insert("wmav2".to_string(), CodecInfo {
            name: "wmav2".to_string(),
            display_name: "WMA v2 (Windows Media Audio)".to_string(),
            description: "Windows Media Audio version 2 codec".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["wma".to_string(), "wmv".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "48k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: true,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 10)),
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::LegacyAudio,
        });
        
        codecs.insert("ac3".to_string(), CodecInfo {
            name: "ac3".to_string(),
            display_name: "AC-3 (Dolby Digital)".to_string(),
            description: "Dolby Digital surround sound codec".to_string(),
            codec_type: CodecType::Audio,
            supported_formats: vec!["ac3".to_string(), "avi".to_string(), "mkv".to_string(), "mp4".to_string(), "mov".to_string()],
            supported_sample_rates: vec![32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "40k".to_string(), "48k".to_string(), "56k".to_string(), "64k".to_string(), "80k".to_string(), "96k".to_string(), "112k".to_string(), "128k".to_string(), "160k".to_string(), "192k".to_string(), "224k".to_string(), "256k".to_string(), "320k".to_string(), "384k".to_string(), "448k".to_string(), "512k".to_string(), "576k".to_string(), "640k".to_string()],
            supported_pixel_formats: vec![],
            supports_vbr: false,
            supports_cbr: true,
            supports_hardware: false,
            default_bitrate: "448k".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
            preset_options: vec![],
            profile_options: vec![],
            level_options: vec![],
            category: CodecCategory::ProfessionalAudio,
        });

        codecs
    }

    /// Get comprehensive container format information
    pub fn get_container_formats() -> HashMap<String, FormatInfo> {
        let mut formats = HashMap::new();
        
        // === Modern Video Containers ===
        formats.insert("mp4".to_string(), FormatInfo {
            extension: "mp4".to_string(),
            display_name: "MP4 (MPEG-4 Part 14)".to_string(),
            description: "Most widely supported modern video container format".to_string(),
            mime_type: "video/mp4".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string(), "ac3".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libaom-av1".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string(), "ac3".to_string(), "eac3".to_string(), "libopus".to_string(), "alac".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libaom-av1".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "av1_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "subtitles".to_string(), "fast_start".to_string(), "streaming".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::ModernVideo,
        });

        formats.insert("mkv".to_string(), FormatInfo {
            extension: "mkv".to_string(),
            display_name: "Matroska Video".to_string(),
            description: "Open-source container format with extensive codec support".to_string(),
            mime_type: "video/x-matroska".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "libvorbis".to_string(), "libopus".to_string(), "flac".to_string(), "ac3".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libvpx-vp9".to_string(), "libaom-av1".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string(), "libvorbis".to_string(), "libopus".to_string(), "flac".to_string(), "ac3".to_string(), "eac3".to_string(), "dts".to_string(), "alac".to_string(), "pcm_s16le".to_string(), "pcm_s24le".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libvpx".to_string(), "libvpx-vp9".to_string(), "libaom-av1".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "av1_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "subtitles".to_string(), "attachments".to_string(), "multiple_tracks".to_string(), "variable_fps".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: true,
            category: FormatCategory::Professional,
        });

        formats.insert("webm".to_string(), FormatInfo {
            extension: "webm".to_string(),
            display_name: "WebM".to_string(),
            description: "Web-optimized container format based on Matroska".to_string(),
            mime_type: "video/webm".to_string(),
            preferred_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string()],
            preferred_video_codecs: vec!["libvpx-vp9".to_string(), "libvpx".to_string(), "libaom-av1".to_string()],
            supported_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string()],
            supported_video_codecs: vec!["libvpx".to_string(), "libvpx-vp9".to_string(), "libaom-av1".to_string(), "av1_nvenc".to_string()],
            container_features: vec!["metadata".to_string(), "web_optimized".to_string(), "streaming".to_string(), "open_source".to_string()],
            max_streams: None,
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::WebOptimized,
        });

        // === Legacy Video Containers ===
        formats.insert("avi".to_string(), FormatInfo {
            extension: "avi".to_string(),
            display_name: "AVI (Audio Video Interleave)".to_string(),
            description: "Legacy Microsoft container format with wide compatibility".to_string(),
            mime_type: "video/x-msvideo".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string(), "aac".to_string(), "ac3".to_string()],
            preferred_video_codecs: vec!["libx264".to_string()],
            supported_audio_codecs: vec!["libmp3lame".to_string(), "aac".to_string(), "ac3".to_string(), "pcm_s16le".to_string(), "pcm_s24le".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libvpx".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["legacy_support".to_string(), "wide_compatibility".to_string()],
            max_streams: Some(2),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::LegacyVideo,
        });

        formats.insert("mov".to_string(), FormatInfo {
            extension: "mov".to_string(),
            display_name: "QuickTime Movie".to_string(),
            description: "Apple's QuickTime container format".to_string(),
            mime_type: "video/quicktime".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "alac".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "alac".to_string(), "libmp3lame".to_string(), "ac3".to_string(), "pcm_s16le".to_string(), "pcm_s24le".to_string(), "pcm_f32le".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "quicktime_features".to_string(), "pro_res_support".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::Professional,
        });

        // === Broadcast/Professional Formats ===
        formats.insert("ts".to_string(), FormatInfo {
            extension: "ts".to_string(),
            display_name: "MPEG Transport Stream".to_string(),
            description: "Broadcast standard transport stream format".to_string(),
            mime_type: "video/mp2t".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "eac3".to_string(), "libmp3lame".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["broadcast".to_string(), "streaming".to_string(), "error_recovery".to_string(), "multiple_programs".to_string()],
            max_streams: None,
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::Broadcast,
        });

        formats.insert("m2ts".to_string(), FormatInfo {
            extension: "m2ts".to_string(),
            display_name: "BDAV MPEG-2 Transport Stream".to_string(),
            description: "Blu-ray disc transport stream format".to_string(),
            mime_type: "video/mp2t".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "eac3".to_string(), "dts".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "eac3".to_string(), "dts".to_string(), "pcm_s16le".to_string(), "pcm_s24le".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "h264_nvenc".to_string(), "hevc_nvenc".to_string(), "h264_qsv".to_string(), "hevc_qsv".to_string()],
            container_features: vec!["blu_ray".to_string(), "high_bitrate".to_string(), "professional".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::Professional,
        });

        // === Audio-Only Formats ===
        formats.insert("mp3".to_string(), FormatInfo {
            extension: "mp3".to_string(),
            display_name: "MP3 Audio".to_string(),
            description: "Most widely supported compressed audio format".to_string(),
            mime_type: "audio/mpeg".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["libmp3lame".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["id3_tags".to_string(), "widely_supported".to_string(), "streaming".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: true,
            category: FormatCategory::CompressedAudio,
        });

        formats.insert("flac".to_string(), FormatInfo {
            extension: "flac".to_string(),
            display_name: "FLAC Audio".to_string(),
            description: "Free lossless audio codec".to_string(),
            mime_type: "audio/flac".to_string(),
            preferred_audio_codecs: vec!["flac".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["flac".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["lossless".to_string(), "metadata".to_string(), "cue_sheets".to_string(), "open_source".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: true,
            category: FormatCategory::LosslessAudio,
        });

        formats.insert("wav".to_string(), FormatInfo {
            extension: "wav".to_string(),
            display_name: "WAV Audio".to_string(),
            description: "Uncompressed audio format for professional use".to_string(),
            mime_type: "audio/wav".to_string(),
            preferred_audio_codecs: vec!["pcm_s16le".to_string(), "pcm_s24le".to_string(), "pcm_f32le".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["pcm_s16le".to_string(), "pcm_s24le".to_string(), "pcm_s32le".to_string(), "pcm_f32le".to_string(), "pcm_alaw".to_string(), "pcm_mulaw".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["uncompressed".to_string(), "professional".to_string(), "broadcast_wave".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::ProfessionalAudio,
        });

        formats.insert("aac".to_string(), FormatInfo {
            extension: "aac".to_string(),
            display_name: "AAC Audio".to_string(),
            description: "Advanced Audio Coding format".to_string(),
            mime_type: "audio/aac".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "libfdk_aac".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["aac".to_string(), "libfdk_aac".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["high_quality".to_string(), "efficient".to_string(), "streaming".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::CompressedAudio,
        });

        formats.insert("ogg".to_string(), FormatInfo {
            extension: "ogg".to_string(),
            display_name: "OGG Container".to_string(),
            description: "Open-source multimedia container".to_string(),
            mime_type: "application/ogg".to_string(),
            preferred_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string(), "flac".to_string()],
            preferred_video_codecs: vec!["libtheora".to_string()],
            supported_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string(), "flac".to_string()],
            supported_video_codecs: vec!["libtheora".to_string()],
            container_features: vec!["open_source".to_string(), "streaming".to_string(), "metadata".to_string(), "multiple_streams".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::WebAudio,
        });
        
        // Legacy video formats
        formats.insert("wmv".to_string(), FormatInfo {
            extension: "wmv".to_string(),
            display_name: "Windows Media Video".to_string(),
            description: "Microsoft's proprietary video format".to_string(),
            mime_type: "video/x-ms-wmv".to_string(),
            preferred_audio_codecs: vec!["wmav2".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["wmv2".to_string(), "libx264".to_string()],
            supported_audio_codecs: vec!["wmav2".to_string(), "aac".to_string(), "libmp3lame".to_string()],
            supported_video_codecs: vec!["wmv2".to_string(), "libx264".to_string()],
            container_features: vec!["drm_support".to_string(), "windows_media".to_string()],
            max_streams: Some(2),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::LegacyVideo,
        });
        
        formats.insert("flv".to_string(), FormatInfo {
            extension: "flv".to_string(),
            display_name: "Flash Video".to_string(),
            description: "Adobe Flash video format (legacy)".to_string(),
            mime_type: "video/x-flv".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "flv1".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "flv1".to_string()],
            container_features: vec!["streaming".to_string(), "flash_compatible".to_string()],
            max_streams: Some(2),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::LegacyVideo,
        });
        
        formats.insert("3gp".to_string(), FormatInfo {
            extension: "3gp".to_string(),
            display_name: "3GPP Mobile Video".to_string(),
            description: "Mobile phone video format".to_string(),
            mime_type: "video/3gpp".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "amr_nb".to_string()],
            preferred_video_codecs: vec!["libx264".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "amr_nb".to_string(), "amr_wb".to_string()],
            supported_video_codecs: vec!["libx264".to_string()],
            container_features: vec!["mobile_optimized".to_string(), "small_size".to_string()],
            max_streams: Some(2),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::LegacyVideo,
        });
        
        formats.insert("m4v".to_string(), FormatInfo {
            extension: "m4v".to_string(),
            display_name: "MPEG-4 Video".to_string(),
            description: "Apple's MPEG-4 video format".to_string(),
            mime_type: "video/x-m4v".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "alac".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            supported_audio_codecs: vec!["aac".to_string(), "alac".to_string(), "libmp3lame".to_string()],
            supported_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "h264_videotoolbox".to_string()],
            container_features: vec!["itunes_compatible".to_string(), "apple_devices".to_string()],
            max_streams: None,
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: true,
            supports_attachments: false,
            category: FormatCategory::ModernVideo,
        });
        
        // Legacy audio formats
        formats.insert("wma".to_string(), FormatInfo {
            extension: "wma".to_string(),
            display_name: "Windows Media Audio".to_string(),
            description: "Microsoft's proprietary audio format".to_string(),
            mime_type: "audio/x-ms-wma".to_string(),
            preferred_audio_codecs: vec!["wmav2".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["wmav2".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["drm_support".to_string(), "windows_media".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::LegacyAudio,
        });
        
        formats.insert("m4a".to_string(), FormatInfo {
            extension: "m4a".to_string(),
            display_name: "MPEG-4 Audio".to_string(),
            description: "Apple's audio format, supports AAC and ALAC".to_string(),
            mime_type: "audio/mp4".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "alac".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["aac".to_string(), "alac".to_string(), "libfdk_aac".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["itunes_compatible".to_string(), "apple_lossless".to_string(), "metadata".to_string()],
            max_streams: Some(1),
            supports_chapters: true,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: true,
            category: FormatCategory::CompressedAudio,
        });
        
        formats.insert("opus".to_string(), FormatInfo {
            extension: "opus".to_string(),
            display_name: "Opus Audio".to_string(),
            description: "Modern, high-quality audio codec".to_string(),
            mime_type: "audio/opus".to_string(),
            preferred_audio_codecs: vec!["libopus".to_string()],
            preferred_video_codecs: vec![],
            supported_audio_codecs: vec!["libopus".to_string()],
            supported_video_codecs: vec![],
            container_features: vec!["low_latency".to_string(), "high_quality".to_string(), "voip_optimized".to_string()],
            max_streams: Some(1),
            supports_chapters: false,
            supports_metadata: true,
            supports_subtitles: false,
            supports_attachments: false,
            category: FormatCategory::CompressedAudio,
        });

        formats
    }

    /// Get codec compatibility matrix
    pub fn get_codec_format_compatibility() -> HashMap<String, Vec<String>> {
        let mut compatibility = HashMap::new();
        
        // Video codecs compatibility
        compatibility.insert("libx264".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string(), "m2ts".to_string(), "3gp".to_string(), "flv".to_string(), "m4v".to_string()]);
        compatibility.insert("libx265".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string(), "m2ts".to_string(), "m4v".to_string()]);
        compatibility.insert("libvpx".to_string(), vec!["webm".to_string(), "mkv".to_string(), "avi".to_string()]);
        compatibility.insert("libvpx-vp9".to_string(), vec!["webm".to_string(), "mkv".to_string()]);
        compatibility.insert("libaom-av1".to_string(), vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()]);
        
        // Legacy codecs for wmv and flv
        compatibility.insert("wmv2".to_string(), vec!["wmv".to_string(), "avi".to_string()]);
        compatibility.insert("flv1".to_string(), vec!["flv".to_string(), "avi".to_string()]);
        
        // Hardware accelerated codecs
        compatibility.insert("h264_nvenc".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string(), "flv".to_string(), "m4v".to_string()]);
        compatibility.insert("hevc_nvenc".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string(), "m4v".to_string()]);
        compatibility.insert("av1_nvenc".to_string(), vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()]);
        compatibility.insert("vp9_nvenc".to_string(), vec!["webm".to_string(), "mkv".to_string()]);
        
        // AMD AMF encoders
        compatibility.insert("h264_amf".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string(), "flv".to_string(), "m4v".to_string()]);
        compatibility.insert("hevc_amf".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string(), "m4v".to_string()]);
        compatibility.insert("av1_amf".to_string(), vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()]);
        
        // Intel QSV
        compatibility.insert("h264_qsv".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string(), "flv".to_string(), "m4v".to_string()]);
        compatibility.insert("hevc_qsv".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string(), "m4v".to_string()]);
        compatibility.insert("av1_qsv".to_string(), vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()]);
        compatibility.insert("vp9_qsv".to_string(), vec!["webm".to_string(), "mkv".to_string()]);
        
        // VA-API
        compatibility.insert("h264_vaapi".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string(), "ts".to_string()]);
        compatibility.insert("hevc_vaapi".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "ts".to_string()]);
        compatibility.insert("av1_vaapi".to_string(), vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()]);
        compatibility.insert("vp8_vaapi".to_string(), vec!["webm".to_string(), "mkv".to_string()]);
        compatibility.insert("vp9_vaapi".to_string(), vec!["webm".to_string(), "mkv".to_string()]);
        
        // VideoToolbox
        compatibility.insert("h264_videotoolbox".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "m4v".to_string()]);
        compatibility.insert("hevc_videotoolbox".to_string(), vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string(), "m4v".to_string()]);
        compatibility.insert("prores_videotoolbox".to_string(), vec!["mov".to_string(), "mkv".to_string()]);
        
        // Audio codecs compatibility
        compatibility.insert("libmp3lame".to_string(), vec!["mp3".to_string(), "avi".to_string(), "mkv".to_string(), "mp4".to_string(), "mov".to_string(), "flv".to_string()]);
        compatibility.insert("aac".to_string(), vec!["aac".to_string(), "m4a".to_string(), "mp4".to_string(), "mkv".to_string(), "mov".to_string(), "avi".to_string(), "ts".to_string(), "flv".to_string(), "3gp".to_string(), "m4v".to_string()]);
        compatibility.insert("libfdk_aac".to_string(), vec!["aac".to_string(), "m4a".to_string(), "mp4".to_string(), "mkv".to_string(), "mov".to_string(), "m4v".to_string()]);
        compatibility.insert("libopus".to_string(), vec!["opus".to_string(), "ogg".to_string(), "webm".to_string(), "mkv".to_string()]);
        compatibility.insert("libvorbis".to_string(), vec!["ogg".to_string(), "webm".to_string(), "mkv".to_string()]);
        compatibility.insert("flac".to_string(), vec!["flac".to_string(), "mkv".to_string(), "ogg".to_string()]);
        compatibility.insert("alac".to_string(), vec!["m4a".to_string(), "mp4".to_string(), "mov".to_string(), "m4v".to_string()]);
        compatibility.insert("pcm_s16le".to_string(), vec!["wav".to_string(), "mov".to_string(), "avi".to_string()]);
        compatibility.insert("pcm_s24le".to_string(), vec!["wav".to_string(), "mov".to_string(), "avi".to_string()]);
        compatibility.insert("pcm_f32le".to_string(), vec!["wav".to_string(), "mov".to_string(), "avi".to_string()]);
        
        // Legacy audio for wma/wmv
        compatibility.insert("wmav2".to_string(), vec!["wma".to_string(), "wmv".to_string(), "avi".to_string()]);
        compatibility.insert("ac3".to_string(), vec!["ac3".to_string(), "avi".to_string(), "mkv".to_string(), "mp4".to_string(), "mov".to_string(), "ts".to_string()]);
        
        // Mobile audio codecs
        compatibility.insert("amr_nb".to_string(), vec!["3gp".to_string(), "mp4".to_string()]);
        compatibility.insert("amr_wb".to_string(), vec!["3gp".to_string(), "mp4".to_string()]);
        
        compatibility
    }

    /// Validate codec and format combination
    pub fn is_compatible(codec: &str, format: &str) -> bool {
        if let Some(compatible_formats) = Self::get_codec_format_compatibility().get(codec) {
            compatible_formats.contains(&format.to_string())
        } else {
            false
        }
    }

    /// Get recommended codecs for a format
    pub fn get_recommended_codecs_for_format(format: &str) -> (Vec<String>, Vec<String>) {
        let formats = Self::get_container_formats();
        if let Some(format_info) = formats.get(format) {
            (format_info.preferred_video_codecs.clone(), format_info.preferred_audio_codecs.clone())
        } else {
            (vec![], vec![])
        }
    }
    
    /// Get the best default audio codec for a format
    pub fn get_default_audio_codec_for_format(format: &str) -> String {
        let formats = Self::get_container_formats();
        if let Some(format_info) = formats.get(format) {
            // Return the first preferred audio codec, or "aac" as fallback
            format_info.preferred_audio_codecs.first()
                .cloned()
                .unwrap_or_else(|| "aac".to_string())
        } else {
            // Default fallback
            match format {
                "mp4" | "m4v" | "mov" => "aac".to_string(),
                "webm" => "libopus".to_string(),
                "ogg" => "libvorbis".to_string(),
                "mkv" => "aac".to_string(),
                "avi" => "libmp3lame".to_string(),
                "wmv" => "wmav2".to_string(),
                "flv" => "aac".to_string(),
                "3gp" => "aac".to_string(),
                _ => "aac".to_string(),
            }
        }
    }

    /// Get all available formats grouped by category
    pub fn get_formats_by_category() -> HashMap<FormatCategory, Vec<String>> {
        let mut by_category = HashMap::new();
        let formats = Self::get_container_formats();
        
        for (format_name, format_info) in formats {
            by_category.entry(format_info.category.clone())
                .or_insert_with(Vec::new)
                .push(format_name);
        }
        
        by_category
    }

    /// Get all available codecs grouped by category
    pub fn get_codecs_by_category() -> HashMap<CodecCategory, Vec<String>> {
        let mut by_category = HashMap::new();
        let mut all_codecs = Self::get_video_codecs();
        all_codecs.extend(Self::get_audio_codecs());
        
        for (codec_name, codec_info) in all_codecs {
            by_category.entry(codec_info.category.clone())
                .or_insert_with(Vec::new)
                .push(codec_name);
        }
        
        by_category
    }
}