use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::app_state::*;
use crate::{log_debug, log_info};
use crate::comprehensive_codec_registry::ComprehensiveCodecRegistry;

/// Encoder compatibility manager
pub struct CodecManager;

#[derive(Debug, Clone)]
pub struct CodecInfo {
    pub name: String,
    pub display_name: String,
    pub supported_formats: Vec<String>,
    pub supported_sample_rates: Vec<u32>,
    pub supported_bit_rates: Vec<String>,
    pub supports_vbr: bool,
    pub supports_cbr: bool,
    pub default_bitrate: String,
    pub default_sample_rate: u32,
    pub quality_range: Option<(u32, u32)>, // VBR quality range
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub extension: String,
    pub display_name: String,
    pub preferred_audio_codecs: Vec<String>,
    pub preferred_video_codecs: Vec<String>,
    pub container_features: Vec<String>,
}

impl CodecManager {
    /// Get all supported audio encoders
    pub fn get_audio_codecs() -> HashMap<String, CodecInfo> {
        let mut codecs = HashMap::new();
        
        // MP3 (LAME)
        codecs.insert("libmp3lame".to_string(), CodecInfo {
            name: "libmp3lame".to_string(),
            display_name: "MP3 (LAME)".to_string(),
            supported_formats: vec!["mp3".to_string(), "avi".to_string(), "mkv".to_string(), "mp4".to_string(), "mov".to_string()],
            supported_sample_rates: vec![8000, 11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "64k".to_string(), "96k".to_string(), 
                                    "128k".to_string(), "160k".to_string(), "192k".to_string(), 
                                    "224k".to_string(), "256k".to_string(), "320k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 9)), // VBR quality: 0=highest quality, 9=lowest quality
        });

        // AAC
        codecs.insert("aac".to_string(), CodecInfo {
            name: "aac".to_string(),
            display_name: "AAC (Advanced Audio Coding)".to_string(),
            supported_formats: vec!["aac".to_string(), "m4a".to_string(), "mp4".to_string(), "mkv".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000],
            supported_bit_rates: vec!["32k".to_string(), "64k".to_string(), "96k".to_string(), 
                                    "128k".to_string(), "160k".to_string(), "192k".to_string(), 
                                    "224k".to_string(), "256k".to_string(), "320k".to_string(), 
                                    "384k".to_string(), "448k".to_string(), "512k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((1, 5)), // VBR quality: 1=highest quality, 5=lowest quality
        });

        // FLAC
        codecs.insert("flac".to_string(), CodecInfo {
            name: "flac".to_string(),
            display_name: "FLAC (Free Lossless Audio Codec)".to_string(),
            supported_formats: vec!["flac".to_string(), "mkv".to_string(), "ogg".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 64000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![], // FLAC is lossless, doesn't use bitrate
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 12)), // Compression level: 0=fastest, 12=smallest file
        });

        // Opus
        codecs.insert("libopus".to_string(), CodecInfo {
            name: "libopus".to_string(),
            display_name: "Opus".to_string(),
            supported_formats: vec!["opus".to_string(), "ogg".to_string(), "webm".to_string()],
            supported_sample_rates: vec![8000, 12000, 16000, 24000, 48000],
            supported_bit_rates: vec!["6k".to_string(), "8k".to_string(), "16k".to_string(), 
                                    "24k".to_string(), "32k".to_string(), "48k".to_string(), 
                                    "64k".to_string(), "96k".to_string(), "128k".to_string(), 
                                    "160k".to_string(), "192k".to_string(), "256k".to_string(), 
                                    "320k".to_string(), "450k".to_string(), "510k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 48000,
            quality_range: Some((0, 10)), // VBR quality: 0=lowest quality, 10=highest quality
        });

        // Vorbis
        codecs.insert("libvorbis".to_string(), CodecInfo {
            name: "libvorbis".to_string(),
            display_name: "Vorbis".to_string(),
            supported_formats: vec!["ogg".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "64k".to_string(), "96k".to_string(), 
                                    "128k".to_string(), "160k".to_string(), "192k".to_string(), 
                                    "224k".to_string(), "256k".to_string(), "320k".to_string(), 
                                    "384k".to_string(), "448k".to_string(), "500k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 10)), // VBR quality: 0=lowest quality, 10=highest quality
        });

        // WAV PCM 16-bit
        codecs.insert("pcm_s16le".to_string(), CodecInfo {
            name: "pcm_s16le".to_string(),
            display_name: "PCM 16-bit (WAV)".to_string(),
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![], // PCM does not use bitrate
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
        });

        // WAV PCM 24-bit
        codecs.insert("pcm_s24le".to_string(), CodecInfo {
            name: "pcm_s24le".to_string(),
            display_name: "PCM 24-bit (WAV)".to_string(),
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![], // PCM does not use bitrate
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
        });

        // AC-3
        codecs.insert("ac3".to_string(), CodecInfo {
            name: "ac3".to_string(),
            display_name: "AC-3 (Dolby Digital)".to_string(),
            supported_formats: vec!["ac3".to_string(), "mp4".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "40k".to_string(), "48k".to_string(), 
                                    "56k".to_string(), "64k".to_string(), "80k".to_string(), 
                                    "96k".to_string(), "112k".to_string(), "128k".to_string(), 
                                    "160k".to_string(), "192k".to_string(), "224k".to_string(), 
                                    "256k".to_string(), "320k".to_string(), "384k".to_string(), 
                                    "448k".to_string(), "512k".to_string(), "576k".to_string(), 
                                    "640k".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "192k".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
        });

        // E-AC-3 (Enhanced AC-3)
        codecs.insert("eac3".to_string(), CodecInfo {
            name: "eac3".to_string(),
            display_name: "E-AC-3 (Dolby Digital Plus)".to_string(),
            supported_formats: vec!["eac3".to_string(), "mp4".to_string(), "mkv".to_string(), "ts".to_string()],
            supported_sample_rates: vec![32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "64k".to_string(), "96k".to_string(), 
                                    "128k".to_string(), "192k".to_string(), "256k".to_string(), 
                                    "320k".to_string(), "384k".to_string(), "512k".to_string(), 
                                    "640k".to_string(), "768k".to_string(), "1024k".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "256k".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
        });

        // DTS
        codecs.insert("dts".to_string(), CodecInfo {
            name: "dts".to_string(),
            display_name: "DTS (Digital Theater Systems)".to_string(),
            supported_formats: vec!["dts".to_string(), "mkv".to_string(), "wav".to_string()],
            supported_sample_rates: vec![44100, 48000, 88200, 96000],
            supported_bit_rates: vec!["754k".to_string(), "1411k".to_string(), "1509k".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "1411k".to_string(),
            default_sample_rate: 48000,
            quality_range: None,
        });

        // ALAC (Apple Lossless)
        codecs.insert("alac".to_string(), CodecInfo {
            name: "alac".to_string(),
            display_name: "ALAC (Apple Lossless Audio Codec)".to_string(),
            supported_formats: vec!["m4a".to_string(), "mp4".to_string(), "mov".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000],
            supported_bit_rates: vec![], // Lossless
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
        });

        // WMA (Windows Media Audio)
        codecs.insert("wmav2".to_string(), CodecInfo {
            name: "wmav2".to_string(),
            display_name: "WMA v2 (Windows Media Audio)".to_string(),
            supported_formats: vec!["wma".to_string(), "asf".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000],
            supported_bit_rates: vec!["32k".to_string(), "48k".to_string(), "64k".to_string(), 
                                    "80k".to_string(), "96k".to_string(), "128k".to_string(), 
                                    "160k".to_string(), "192k".to_string(), "256k".to_string(), 
                                    "320k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "128k".to_string(),
            default_sample_rate: 44100,
            quality_range: Some((0, 100)),
        });

        // Speex
        codecs.insert("libspeex".to_string(), CodecInfo {
            name: "libspeex".to_string(),
            display_name: "Speex (Speech Codec)".to_string(),
            supported_formats: vec!["spx".to_string(), "ogg".to_string()],
            supported_sample_rates: vec![8000, 16000, 32000],
            supported_bit_rates: vec!["2.15k".to_string(), "5.95k".to_string(), "8k".to_string(), 
                                    "11k".to_string(), "15k".to_string(), "18.2k".to_string(), 
                                    "24.6k".to_string(), "42.2k".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "15k".to_string(),
            default_sample_rate: 16000,
            quality_range: Some((0, 10)),
        });

        // PCM variants
        codecs.insert("pcm_s32le".to_string(), CodecInfo {
            name: "pcm_s32le".to_string(),
            display_name: "PCM 32-bit (WAV)".to_string(),
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
        });

        codecs.insert("pcm_f32le".to_string(), CodecInfo {
            name: "pcm_f32le".to_string(),
            display_name: "PCM 32-bit Float (WAV)".to_string(),
            supported_formats: vec!["wav".to_string(), "mov".to_string(), "avi".to_string()],
            supported_sample_rates: vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000],
            supported_bit_rates: vec![],
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 44100,
            quality_range: None,
        });

        // G.711 A-law
        codecs.insert("pcm_alaw".to_string(), CodecInfo {
            name: "pcm_alaw".to_string(),
            display_name: "PCM A-law (G.711)".to_string(),
            supported_formats: vec!["wav".to_string(), "au".to_string()],
            supported_sample_rates: vec![8000],
            supported_bit_rates: vec!["64k".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "64k".to_string(),
            default_sample_rate: 8000,
            quality_range: None,
        });

        // G.711 μ-law
        codecs.insert("pcm_mulaw".to_string(), CodecInfo {
            name: "pcm_mulaw".to_string(),
            display_name: "PCM μ-law (G.711)".to_string(),
            supported_formats: vec!["wav".to_string(), "au".to_string()],
            supported_sample_rates: vec![8000],
            supported_bit_rates: vec!["64k".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "64k".to_string(),
            default_sample_rate: 8000,
            quality_range: None,
        });

        codecs
    }

    /// Get all supported video encoders
    pub fn get_video_codecs() -> HashMap<String, CodecInfo> {
        let mut codecs = HashMap::new();
        
        // H.264
        codecs.insert("libx264".to_string(), CodecInfo {
            name: "libx264".to_string(),
            display_name: "H.264 (x264)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string()],
            supported_sample_rates: vec![], // Video encoders do not use sample rates
            supported_bit_rates: vec!["500k".to_string(), "1M".to_string(), "2M".to_string(), 
                                    "3M".to_string(), "4M".to_string(), "5M".to_string(), 
                                    "6M".to_string(), "8M".to_string(), "10M".to_string(), 
                                    "15M".to_string(), "20M".to_string(), "25M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)), // CRF quality: 0=lossless, 51=worst quality
        });

        // H.265
        codecs.insert("libx265".to_string(), CodecInfo {
            name: "libx265".to_string(),
            display_name: "H.265 (x265)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["300k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "1.5M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "5M".to_string(), "6M".to_string(), 
                                    "8M".to_string(), "10M".to_string(), "15M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "1.5M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
        });

        // VP8
        codecs.insert("libvpx".to_string(), CodecInfo {
            name: "libvpx".to_string(),
            display_name: "VP8".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "2M".to_string(), "3M".to_string(), "4M".to_string(), 
                                    "5M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)), // CRF quality range
        });

        // VP9
        codecs.insert("libvpx-vp9".to_string(), CodecInfo {
            name: "libvpx-vp9".to_string(),
            display_name: "VP9".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "2M".to_string(), "3M".to_string(), "4M".to_string(), 
                                    "5M".to_string(), "6M".to_string(), "8M".to_string(), 
                                    "10M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)), // CRF quality range
        });

        // AV1
        codecs.insert("libaom-av1".to_string(), CodecInfo {
            name: "libaom-av1".to_string(),
            display_name: "AV1".to_string(),
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "1.5M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "5M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "1.5M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
        });

        // Hardware encoders - NVIDIA NVENC
        codecs.insert("h264_nvenc".to_string(), CodecInfo {
            name: "h264_nvenc".to_string(),
            display_name: "H.264 NVENC (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), 
                                    "6M".to_string(), "8M".to_string(), "10M".to_string(), 
                                    "15M".to_string(), "20M".to_string(), "25M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "4M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
        });

        codecs.insert("hevc_nvenc".to_string(), CodecInfo {
            name: "hevc_nvenc".to_string(),
            display_name: "H.265 NVENC (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string(), 
                                    "10M".to_string(), "15M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
        });

        codecs.insert("av1_nvenc".to_string(), CodecInfo {
            name: "av1_nvenc".to_string(),
            display_name: "AV1 NVENC (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 255)),
        });

        // Hardware encoders - Intel QuickSync
        codecs.insert("h264_qsv".to_string(), CodecInfo {
            name: "h264_qsv".to_string(),
            display_name: "H.264 QuickSync (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "avi".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), 
                                    "6M".to_string(), "8M".to_string(), "10M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "4M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
        });

        codecs.insert("hevc_qsv".to_string(), CodecInfo {
            name: "hevc_qsv".to_string(),
            display_name: "H.265 QuickSync (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "mkv".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
        });

        // Hardware encoders - Additional NVENC
        codecs.insert("vp9_nvenc".to_string(), CodecInfo {
            name: "vp9_nvenc".to_string(),
            display_name: "VP9 NVENC (Hardware)".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string(), 
                                    "10M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
        });

        // Hardware encoders - Additional QuickSync
        codecs.insert("vp9_qsv".to_string(), CodecInfo {
            name: "vp9_qsv".to_string(),
            display_name: "VP9 QuickSync (Hardware)".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 51)),
        });

        codecs.insert("av1_qsv".to_string(), CodecInfo {
            name: "av1_qsv".to_string(),
            display_name: "AV1 QuickSync (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 255)),
        });

        // Hardware encoders - AMD AMF
        codecs.insert("av1_amf".to_string(), CodecInfo {
            name: "av1_amf".to_string(),
            display_name: "AV1 AMF (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 51)),
        });

        // Hardware encoders - Additional VA-API
        codecs.insert("vp8_vaapi".to_string(), CodecInfo {
            name: "vp8_vaapi".to_string(),
            display_name: "VP8 VA-API (Hardware)".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["500k".to_string(), "1M".to_string(), "2M".to_string(), 
                                    "3M".to_string(), "4M".to_string(), "6M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
        });

        codecs.insert("vp9_vaapi".to_string(), CodecInfo {
            name: "vp9_vaapi".to_string(),
            display_name: "VP9 VA-API (Hardware)".to_string(),
            supported_formats: vec!["webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 63)),
        });

        codecs.insert("av1_vaapi".to_string(), CodecInfo {
            name: "av1_vaapi".to_string(),
            display_name: "AV1 VA-API (Hardware)".to_string(),
            supported_formats: vec!["mp4".to_string(), "webm".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "3M".to_string(), 
                                    "4M".to_string(), "6M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "3M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 255)),
        });

        // Apple VideoToolbox additional
        codecs.insert("prores_videotoolbox".to_string(), CodecInfo {
            name: "prores_videotoolbox".to_string(),
            display_name: "ProRes VideoToolbox (Hardware)".to_string(),
            supported_formats: vec!["mov".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec![], // ProRes uses quality levels, not bitrates
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "auto".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 4)), // ProRes quality levels
        });

        // Additional software codecs
        
        // MJPEG (Motion JPEG)
        codecs.insert("mjpeg".to_string(), CodecInfo {
            name: "mjpeg".to_string(),
            display_name: "MJPEG (Motion JPEG)".to_string(),
            supported_formats: vec!["avi".to_string(), "mov".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "5M".to_string(), 
                                    "10M".to_string(), "15M".to_string(), "20M".to_string(), 
                                    "25M".to_string(), "30M".to_string(), "50M".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "10M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)), // JPEG quality
        });

        // FFV1 (Lossless)
        codecs.insert("ffv1".to_string(), CodecInfo {
            name: "ffv1".to_string(),
            display_name: "FFV1 (Lossless)".to_string(),
            supported_formats: vec!["mkv".to_string(), "avi".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec![], // Lossless
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 9)), // Compression level
        });

        // Huffyuv (Lossless)
        codecs.insert("huffyuv".to_string(), CodecInfo {
            name: "huffyuv".to_string(),
            display_name: "HuffYUV (Lossless)".to_string(),
            supported_formats: vec!["avi".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec![], // Lossless
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "lossless".to_string(),
            default_sample_rate: 0,
            quality_range: None,
        });

        // Theora
        codecs.insert("libtheora".to_string(), CodecInfo {
            name: "libtheora".to_string(),
            display_name: "Theora".to_string(),
            supported_formats: vec!["ogg".to_string(), "ogv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "2M".to_string(), "3M".to_string(), "4M".to_string(), 
                                    "5M".to_string(), "6M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 10)),
        });

        // DV (Digital Video)
        codecs.insert("dvvideo".to_string(), CodecInfo {
            name: "dvvideo".to_string(),
            display_name: "DV (Digital Video)".to_string(),
            supported_formats: vec!["dv".to_string(), "avi".to_string(), "mov".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["25M".to_string()], // Fixed bitrate for DV
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "25M".to_string(),
            default_sample_rate: 0,
            quality_range: None,
        });

        // MPEG-2
        codecs.insert("mpeg2video".to_string(), CodecInfo {
            name: "mpeg2video".to_string(),
            display_name: "MPEG-2 Video".to_string(),
            supported_formats: vec!["mpg".to_string(), "mpeg".to_string(), "vob".to_string(), "ts".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "2M".to_string(), "4M".to_string(), 
                                    "6M".to_string(), "8M".to_string(), "10M".to_string(), 
                                    "15M".to_string(), "20M".to_string(), "30M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "6M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)),
        });

        // MPEG-1
        codecs.insert("mpeg1video".to_string(), CodecInfo {
            name: "mpeg1video".to_string(),
            display_name: "MPEG-1 Video".to_string(),
            supported_formats: vec!["mpg".to_string(), "mpeg".to_string(), "vcd".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["1M".to_string(), "1.5M".to_string(), "2M".to_string(), 
                                    "3M".to_string(), "4M".to_string(), "6M".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "1.5M".to_string(),
            default_sample_rate: 0,
            quality_range: None,
        });

        // WMV (Windows Media Video)
        codecs.insert("wmv2".to_string(), CodecInfo {
            name: "wmv2".to_string(),
            display_name: "WMV v2 (Windows Media Video)".to_string(),
            supported_formats: vec!["wmv".to_string(), "asf".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "2M".to_string(), "3M".to_string(), "4M".to_string(), 
                                    "6M".to_string(), "8M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "2M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 100)),
        });

        // H.263
        codecs.insert("h263".to_string(), CodecInfo {
            name: "h263".to_string(),
            display_name: "H.263".to_string(),
            supported_formats: vec!["3gp".to_string(), "mp4".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["64k".to_string(), "128k".to_string(), "256k".to_string(), 
                                    "384k".to_string(), "512k".to_string(), "768k".to_string(), 
                                    "1M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "384k".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)),
        });

        // Flash Video
        codecs.insert("flv".to_string(), CodecInfo {
            name: "flv".to_string(),
            display_name: "Flash Video (FLV)".to_string(),
            supported_formats: vec!["flv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["200k".to_string(), "500k".to_string(), "1M".to_string(), 
                                    "2M".to_string(), "3M".to_string(), "4M".to_string()],
            supports_vbr: true,
            supports_cbr: true,
            default_bitrate: "1M".to_string(),
            default_sample_rate: 0,
            quality_range: Some((1, 31)),
        });

        // ProRes (software)
        codecs.insert("prores".to_string(), CodecInfo {
            name: "prores".to_string(),
            display_name: "Apple ProRes (Software)".to_string(),
            supported_formats: vec!["mov".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec![], // ProRes uses quality levels
            supports_vbr: false,
            supports_cbr: false,
            default_bitrate: "auto".to_string(),
            default_sample_rate: 0,
            quality_range: Some((0, 4)), // ProRes profiles: Proxy, LT, Standard, HQ, 4444
        });

        // DNxHD/DNxHR (Avid)
        codecs.insert("dnxhd".to_string(), CodecInfo {
            name: "dnxhd".to_string(),
            display_name: "DNxHD/DNxHR (Avid)".to_string(),
            supported_formats: vec!["mov".to_string(), "mxf".to_string(), "mkv".to_string()],
            supported_sample_rates: vec![],
            supported_bit_rates: vec!["36M".to_string(), "45M".to_string(), "60M".to_string(), 
                                    "90M".to_string(), "120M".to_string(), "145M".to_string(), 
                                    "185M".to_string(), "220M".to_string(), "365M".to_string()],
            supports_vbr: false,
            supports_cbr: true,
            default_bitrate: "120M".to_string(),
            default_sample_rate: 0,
            quality_range: None,
        });

        codecs
    }

    /// Get all supported format information
    pub fn get_format_info() -> HashMap<String, FormatInfo> {
        let mut formats = HashMap::new();
        
        formats.insert("mp4".to_string(), FormatInfo {
            extension: "mp4".to_string(),
            display_name: "MP4 Container".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libaom-av1".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "subtitles".to_string()],
        });

        formats.insert("mkv".to_string(), FormatInfo {
            extension: "mkv".to_string(),
            display_name: "Matroska Video".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "libvorbis".to_string(), "libopus".to_string(), "flac".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string(), "libvpx-vp9".to_string(), "libaom-av1".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "subtitles".to_string(), "attachments".to_string()],
        });

        formats.insert("webm".to_string(), FormatInfo {
            extension: "webm".to_string(),
            display_name: "WebM".to_string(),
            preferred_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string()],
            preferred_video_codecs: vec!["libvpx-vp9".to_string(), "libvpx".to_string(), "libaom-av1".to_string()],
            container_features: vec!["metadata".to_string(), "web_optimized".to_string()],
        });

        formats.insert("avi".to_string(), FormatInfo {
            extension: "avi".to_string(),
            display_name: "AVI".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string(), "aac".to_string(), "ac3".to_string()],
            preferred_video_codecs: vec!["libx264".to_string()],
            container_features: vec!["legacy_support".to_string()],
        });

        formats.insert("mov".to_string(), FormatInfo {
            extension: "mov".to_string(),
            display_name: "QuickTime Movie".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            container_features: vec!["metadata".to_string(), "chapters".to_string(), "quicktime_features".to_string()],
        });

        // Audio-only formats
        formats.insert("mp3".to_string(), FormatInfo {
            extension: "mp3".to_string(),
            display_name: "MP3 Audio".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["id3_tags".to_string(), "widely_supported".to_string()],
        });

        formats.insert("flac".to_string(), FormatInfo {
            extension: "flac".to_string(),
            display_name: "FLAC Audio".to_string(),
            preferred_audio_codecs: vec!["flac".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["lossless".to_string(), "metadata".to_string()],
        });

        formats.insert("wav".to_string(), FormatInfo {
            extension: "wav".to_string(),
            display_name: "WAV Audio".to_string(),
            preferred_audio_codecs: vec!["pcm_s16le".to_string(), "pcm_s24le".to_string(), "pcm_f32le".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["uncompressed".to_string(), "professional".to_string()],
        });

        formats.insert("aac".to_string(), FormatInfo {
            extension: "aac".to_string(),
            display_name: "AAC Audio".to_string(),
            preferred_audio_codecs: vec!["aac".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["high_quality".to_string(), "efficient".to_string()],
        });

        formats.insert("m4a".to_string(), FormatInfo {
            extension: "m4a".to_string(),
            display_name: "M4A Audio".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "alac".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["itunes_compatible".to_string(), "metadata".to_string()],
        });

        formats.insert("ogg".to_string(), FormatInfo {
            extension: "ogg".to_string(),
            display_name: "OGG Audio".to_string(),
            preferred_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string(), "flac".to_string()],
            preferred_video_codecs: vec!["libtheora".to_string()],
            container_features: vec!["open_source".to_string(), "streaming".to_string()],
        });

        formats.insert("opus".to_string(), FormatInfo {
            extension: "opus".to_string(),
            display_name: "Opus Audio".to_string(),
            preferred_audio_codecs: vec!["libopus".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["low_latency".to_string(), "high_quality".to_string()],
        });

        formats.insert("wma".to_string(), FormatInfo {
            extension: "wma".to_string(),
            display_name: "Windows Media Audio".to_string(),
            preferred_audio_codecs: vec!["wmav2".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["windows_native".to_string(), "drm_support".to_string()],
        });

        // Additional video formats
        formats.insert("wmv".to_string(), FormatInfo {
            extension: "wmv".to_string(),
            display_name: "Windows Media Video".to_string(),
            preferred_audio_codecs: vec!["wmav2".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["wmv2".to_string(), "libx264".to_string()],
            container_features: vec!["windows_native".to_string(), "streaming".to_string()],
        });

        formats.insert("flv".to_string(), FormatInfo {
            extension: "flv".to_string(),
            display_name: "Flash Video".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["flv".to_string(), "libx264".to_string()],
            container_features: vec!["streaming".to_string(), "web_legacy".to_string()],
        });

        formats.insert("3gp".to_string(), FormatInfo {
            extension: "3gp".to_string(),
            display_name: "3GPP Mobile Video".to_string(),
            preferred_audio_codecs: vec!["libopencore_amrnb".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["h263".to_string(), "libx264".to_string()],
            container_features: vec!["mobile_optimized".to_string(), "small_size".to_string()],
        });

        formats.insert("ogv".to_string(), FormatInfo {
            extension: "ogv".to_string(),
            display_name: "OGG Video".to_string(),
            preferred_audio_codecs: vec!["libvorbis".to_string(), "libopus".to_string()],
            preferred_video_codecs: vec!["libtheora".to_string()],
            container_features: vec!["open_source".to_string(), "royalty_free".to_string()],
        });

        formats.insert("mpg".to_string(), FormatInfo {
            extension: "mpg".to_string(),
            display_name: "MPEG-1/2 Video".to_string(),
            preferred_audio_codecs: vec!["libmp3lame".to_string(), "ac3".to_string()],
            preferred_video_codecs: vec!["mpeg2video".to_string(), "mpeg1video".to_string()],
            container_features: vec!["legacy_support".to_string(), "dvd_compatible".to_string()],
        });

        formats.insert("ts".to_string(), FormatInfo {
            extension: "ts".to_string(),
            display_name: "MPEG Transport Stream".to_string(),
            preferred_audio_codecs: vec!["aac".to_string(), "ac3".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "mpeg2video".to_string()],
            container_features: vec!["streaming".to_string(), "broadcast".to_string()],
        });

        formats.insert("mts".to_string(), FormatInfo {
            extension: "mts".to_string(),
            display_name: "AVCHD Video".to_string(),
            preferred_audio_codecs: vec!["ac3".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["libx264".to_string()],
            container_features: vec!["camcorder".to_string(), "high_definition".to_string()],
        });

        formats.insert("m2ts".to_string(), FormatInfo {
            extension: "m2ts".to_string(),
            display_name: "Blu-ray Video".to_string(),
            preferred_audio_codecs: vec!["ac3".to_string(), "eac3".to_string(), "dts".to_string()],
            preferred_video_codecs: vec!["libx264".to_string(), "libx265".to_string()],
            container_features: vec!["blu_ray".to_string(), "high_definition".to_string()],
        });

        formats.insert("mxf".to_string(), FormatInfo {
            extension: "mxf".to_string(),
            display_name: "Material Exchange Format".to_string(),
            preferred_audio_codecs: vec!["pcm_s16le".to_string(), "pcm_s24le".to_string()],
            preferred_video_codecs: vec!["dnxhd".to_string(), "prores".to_string(), "mpeg2video".to_string()],
            container_features: vec!["professional".to_string(), "broadcast".to_string()],
        });

        formats.insert("dv".to_string(), FormatInfo {
            extension: "dv".to_string(),
            display_name: "Digital Video".to_string(),
            preferred_audio_codecs: vec!["pcm_s16le".to_string()],
            preferred_video_codecs: vec!["dvvideo".to_string()],
            container_features: vec!["tape_format".to_string(), "professional".to_string()],
        });

        formats.insert("asf".to_string(), FormatInfo {
            extension: "asf".to_string(),
            display_name: "Advanced Systems Format".to_string(),
            preferred_audio_codecs: vec!["wmav2".to_string(), "aac".to_string()],
            preferred_video_codecs: vec!["wmv2".to_string(), "libx264".to_string()],
            container_features: vec!["streaming".to_string(), "microsoft".to_string()],
        });

        formats.insert("vob".to_string(), FormatInfo {
            extension: "vob".to_string(),
            display_name: "DVD Video Object".to_string(),
            preferred_audio_codecs: vec!["ac3".to_string(), "libmp3lame".to_string()],
            preferred_video_codecs: vec!["mpeg2video".to_string()],
            container_features: vec!["dvd_format".to_string(), "menu_support".to_string()],
        });

        formats.insert("rm".to_string(), FormatInfo {
            extension: "rm".to_string(),
            display_name: "RealMedia".to_string(),
            preferred_audio_codecs: vec!["aac".to_string()],
            preferred_video_codecs: vec!["libx264".to_string()],
            container_features: vec!["streaming".to_string(), "legacy".to_string()],
        });

        formats.insert("gif".to_string(), FormatInfo {
            extension: "gif".to_string(),
            display_name: "Animated GIF".to_string(),
            preferred_audio_codecs: vec![], // GIF doesn't support audio
            preferred_video_codecs: vec!["gif".to_string()],
            container_features: vec!["animation".to_string(), "web_compatible".to_string()],
        });

        // Special formats
        formats.insert("ac3".to_string(), FormatInfo {
            extension: "ac3".to_string(),
            display_name: "Dolby Digital AC-3".to_string(),
            preferred_audio_codecs: vec!["ac3".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["surround_sound".to_string(), "theater_standard".to_string()],
        });

        formats.insert("eac3".to_string(), FormatInfo {
            extension: "eac3".to_string(),
            display_name: "Dolby Digital Plus E-AC-3".to_string(),
            preferred_audio_codecs: vec!["eac3".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["enhanced_surround".to_string(), "blu_ray_standard".to_string()],
        });

        formats.insert("dts".to_string(), FormatInfo {
            extension: "dts".to_string(),
            display_name: "DTS Audio".to_string(),
            preferred_audio_codecs: vec!["dts".to_string()],
            preferred_video_codecs: vec![],
            container_features: vec!["theater_quality".to_string(), "lossless_option".to_string()],
        });

        formats
    }

    /// Get best audio encoder based on output format
    pub fn get_best_audio_codec_for_format(format: &str) -> String {
        // Use comprehensive codec registry for better compatibility
        ComprehensiveCodecRegistry::get_default_audio_codec_for_format(format)
    }

    /// Get best video encoder based on output format
    pub fn get_best_video_codec_for_format(format: &str) -> String {
        let formats = Self::get_format_info();
        if let Some(format_info) = formats.get(format) {
            if let Some(codec) = format_info.preferred_video_codecs.first() {
                return codec.clone();
            }
        }
        
        // Default mapping
        match format {
            "mp4" | "mov" => "libx264".to_string(),
            "mkv" => "libx264".to_string(),
            "webm" => "libvpx-vp9".to_string(),
            "avi" => "libx264".to_string(),
            "wmv" | "asf" => "wmv2".to_string(),
            "flv" => "flv".to_string(),
            "3gp" => "h263".to_string(),
            "ogv" | "ogg" => "libtheora".to_string(),
            "mpg" | "mpeg" => "mpeg2video".to_string(),
            "ts" | "mts" | "m2ts" => "libx264".to_string(),
            "vob" => "mpeg2video".to_string(),
            "mxf" => "dnxhd".to_string(),
            "dv" => "dvvideo".to_string(),
            "rm" => "libx264".to_string(),
            "gif" => "gif".to_string(),
            _ => "libx264".to_string(), // Default to H.264
        }
    }

    /// Validate codec and format compatibility
    pub fn validate_codec_format_compatibility(codec: &str, format: &str, _is_audio: bool) -> Result<()> {
        // Special handling: copy and mkv are always compatible
        if codec == "copy" || codec == "mkv" {
            return Ok(());
        }
        
        // Use comprehensive codec registry for validation
        if ComprehensiveCodecRegistry::is_compatible(codec, format) {
            Ok(())
        } else {
            Err(anyhow!("Codec {} does not support format {}", codec, format))
        }
    }

    /// Validate sample rate compatibility
    pub fn validate_sample_rate(codec: &str, sample_rate: u32) -> Result<u32> {
        let codecs = Self::get_audio_codecs();
        
        if let Some(codec_info) = codecs.get(codec) {
            if codec_info.supported_sample_rates.contains(&sample_rate) {
                Ok(sample_rate)
            } else {
                // Find the closest supported sample rate
                let closest = codec_info.supported_sample_rates
                    .iter()
                    .min_by_key(|&&rate| (rate as i32 - sample_rate as i32).abs())
                    .copied()
                    .unwrap_or(codec_info.default_sample_rate);
                
                // Silently use compatible sample rate
                Ok(closest)
            }
        } else {
            Ok(sample_rate) // Unknown codec, keep original value
        }
    }

    /// Validate bitrate compatibility
    pub fn validate_bitrate(codec: &str, bitrate: &str) -> Result<String> {
        if bitrate == "auto" || bitrate == "lossless" {
            return Ok(bitrate.to_string());
        }

        let codecs = Self::get_audio_codecs();
        
        if let Some(codec_info) = codecs.get(codec) {
            if codec_info.supported_bit_rates.is_empty() {
                // Lossless codec
                return Ok("lossless".to_string());
            }
            
            if codec_info.supported_bit_rates.contains(&bitrate.to_string()) {
                Ok(bitrate.to_string())
            } else {
                // Silently use default bitrate
                Ok(codec_info.default_bitrate.clone())
            }
        } else {
            Ok(bitrate.to_string()) // Unknown codec, keep original value
        }
    }

    /// Detect available encoders using bundled FFmpeg
    pub fn detect_available_codecs() -> Result<Vec<String>> {
        use crate::bundled_ffmpeg::get_bundled_ffmpeg;
        
        log_info!("Detecting available FFmpeg encoders using bundled executable...");
        
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        // Run ffmpeg -encoders to get list of available encoders
        let output = bundled_ffmpeg.run_ffmpeg(&["-hide_banner", "-encoders"])?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        let mut available_codecs = Vec::new();
        
        // Parse the output - skip header lines and extract encoder names
        for line in output_str.lines() {
            // Encoder lines start with a space and have format like:
            //  V..... = Video
            //  A..... = Audio
            //  S..... = Subtitle
            if line.starts_with(' ') && line.len() > 7 {
                // Extract codec name (after the flags)
                if let Some(codec_part) = line.split_whitespace().nth(1) {
                    available_codecs.push(codec_part.to_string());
                }
            }
        }
        
        log_info!("Found {} encoders using bundled FFmpeg", available_codecs.len());
        log_debug!("First 10 encoders: {:?}", available_codecs.iter().take(10).collect::<Vec<_>>());
        
        Ok(available_codecs)
    }

    /// Smart detection of hardware encoders based on actual hardware configuration
    fn get_hardware_encoders_by_detection() -> Vec<&'static str> {
        let mut encoders = Vec::new();
        
        // Detect NVIDIA GPU
        if Self::has_nvidia_gpu() {
            encoders.extend_from_slice(&[
                "h264_nvenc", "hevc_nvenc", "av1_nvenc",  // VP9 NVENC unavailable in most FFmpeg versions
            ]);
        }
        
        // Detect AMD GPU  
        if Self::has_amd_gpu() {
            encoders.extend_from_slice(&[
                "h264_amf", "hevc_amf", "av1_amf",
            ]);
        }
        
        // Detect Intel integrated/dedicated graphics
        if Self::has_intel_gpu() {
            encoders.extend_from_slice(&[
                "h264_qsv", "hevc_qsv", "av1_qsv", "vp9_qsv",  // Intel has good VP9 support
            ]);
        }
        
        encoders
    }
    
    /// Detect NVIDIA GPU (using bundled FFmpeg)
    fn has_nvidia_gpu() -> bool {
        // Use bundled FFmpeg to check for NVENC support
        if let Ok(bundled_ffmpeg) = crate::bundled_ffmpeg::get_bundled_ffmpeg() {
            let available_encoders = bundled_ffmpeg.check_hardware_acceleration();
            available_encoders.iter().any(|e| e.contains("nvenc"))
        } else {
            false
        }
    }
    
    /// Detect AMD GPU (using bundled FFmpeg)
    fn has_amd_gpu() -> bool {
        // Use bundled FFmpeg to check for AMF support
        if let Ok(bundled_ffmpeg) = crate::bundled_ffmpeg::get_bundled_ffmpeg() {
            let available_encoders = bundled_ffmpeg.check_hardware_acceleration();
            available_encoders.iter().any(|e| e.contains("amf"))
        } else {
            false
        }
    }
    
    /// Detect Intel GPU (using bundled FFmpeg)
    fn has_intel_gpu() -> bool {
        // Use bundled FFmpeg to check for QSV support
        if let Ok(bundled_ffmpeg) = crate::bundled_ffmpeg::get_bundled_ffmpeg() {
            let available_encoders = bundled_ffmpeg.check_hardware_acceleration();
            available_encoders.iter().any(|e| e.contains("qsv"))
        } else {
            false
        }
    }

    /// Filter hardware encoders based on platform
    fn get_platform_hardware_encoders() -> Vec<&'static str> {
        #[cfg(target_os = "windows")]
        {
            // Detect actual hardware configuration and return corresponding encoders
            Self::get_hardware_encoders_by_detection()
        }
        #[cfg(target_os = "macos")]
        {
            vec![
                // macOS: VideoToolbox, possibly with Intel QuickSync
                "h264_videotoolbox", "hevc_videotoolbox", "prores_videotoolbox", // Apple VideoToolbox
                "h264_qsv", "hevc_qsv", "av1_qsv", "vp9_qsv",          // Intel QuickSync (Intel Mac)
            ]
        }
        #[cfg(target_os = "linux")]
        {
            vec![
                // Linux: VA-API, possibly with NVIDIA NVENC
                "h264_vaapi", "hevc_vaapi", "av1_vaapi", "vp8_vaapi", "vp9_vaapi", // VA-API
                "mjpeg_vaapi", "mpeg2_vaapi",                          // Additional VA-API
                "h264_nvenc", "hevc_nvenc", "av1_nvenc", "vp9_nvenc",  // NVIDIA (if proprietary drivers available)
                "h264_qsv", "hevc_qsv", "av1_qsv", "vp9_qsv",          // Intel QuickSync
            ]
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            vec![] // Other platforms do not support hardware encoding yet
        }
    }

    /// Detect hardware encoder support (using bundled FFmpeg)
    pub fn detect_hardware_encoders() -> Result<Vec<String>> {
        log_info!("Starting hardware encoder detection using bundled FFmpeg...");
        
        use crate::bundled_ffmpeg::get_bundled_ffmpeg;
        
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        let supported = bundled_ffmpeg.check_hardware_acceleration();
        
        log_info!("Final supported hardware encoders: {:?}", supported);
        Ok(supported)
    }

    /// Smart recommendation of best encoder
    pub fn get_smart_encoder_recommendation(
        output_format: &str,
        quality_preset: &str, 
        speed_priority: bool,
        available_hardware: &[String],
        is_chinese: bool
    ) -> (String, String) {
        let (video_codec, reason) = Self::recommend_video_codec(output_format, quality_preset, speed_priority, available_hardware, is_chinese);
        (video_codec, reason)
    }
    
    /// Recommend video codec
    fn recommend_video_codec(
        output_format: &str, 
        quality_preset: &str,
        speed_priority: bool,
        available_hardware: &[String],
        is_chinese: bool
    ) -> (String, String) {
        let format_lower = output_format.to_lowercase();
        
        // Helper function: return recommendation reason based on language
        let get_reason = |chinese: &str, english: &str| -> String {
            if is_chinese { chinese.to_string() } else { english.to_string() }
        };
        
        // Quality preset check functions
        let is_high_quality = quality_preset.contains("高质量") || quality_preset.contains("High Quality");
        let is_fast = quality_preset.contains("快速") || quality_preset.contains("Fast") || speed_priority;
        let _is_balanced = quality_preset.contains("平衡") || quality_preset.contains("Balanced") || (!is_high_quality && !is_fast);
        
        match format_lower.as_str() {
            "webm" => {
                // WebM format: prioritize VP9, consider hardware acceleration
                if is_fast && available_hardware.contains(&"vp9_qsv".to_string()) {
                    ("vp9_qsv".to_string(), get_reason("硬件加速VP9 (Intel QuickSync)，速度优先", "Hardware VP9 (Intel QuickSync), Speed Priority"))
                } else if is_high_quality {
                    ("libvpx-vp9".to_string(), get_reason("软件VP9，最佳质量 (适合WebM)", "Software VP9, Best Quality (Ideal for WebM)"))
                } else {
                    ("libvpx-vp9".to_string(), get_reason("VP9编码器 (WebM标准)", "VP9 Encoder (WebM Standard)"))
                }
            },
            "mp4" => {
                // MP4 format: prioritize H.264/H.265, consider hardware acceleration
                if is_fast {
                    if available_hardware.contains(&"h264_nvenc".to_string()) {
                        ("h264_nvenc".to_string(), get_reason("NVIDIA硬件加速H.264，速度最快", "NVIDIA Hardware H.264, Fastest Speed"))
                    } else if available_hardware.contains(&"h264_qsv".to_string()) {
                        ("h264_qsv".to_string(), get_reason("Intel硬件加速H.264，速度优先", "Intel Hardware H.264, Speed Priority"))
                    } else if available_hardware.contains(&"h264_amf".to_string()) {
                        ("h264_amf".to_string(), get_reason("AMD硬件加速H.264，速度优先", "AMD Hardware H.264, Speed Priority"))
                    } else {
                        ("libx264".to_string(), get_reason("软件H.264，兼容性最佳", "Software H.264, Best Compatibility"))
                    }
                } else if is_high_quality {
                    if available_hardware.contains(&"hevc_nvenc".to_string()) {
                        ("hevc_nvenc".to_string(), get_reason("NVIDIA hardware H.265, high quality and efficiency", "NVIDIA Hardware H.265, High Quality & Efficiency"))
                    } else if available_hardware.contains(&"hevc_qsv".to_string()) {
                        ("hevc_qsv".to_string(), get_reason("Intel hardware H.265, high quality", "Intel Hardware H.265, High Quality"))
                    } else {
                        ("libx265".to_string(), get_reason("Software H.265, best quality compression", "Software H.265, Best Quality Compression"))
                    }
                } else {
                    // Default balanced
                    if available_hardware.contains(&"h264_nvenc".to_string()) {
                        ("h264_nvenc".to_string(), get_reason("NVIDIA hardware H.264, balanced speed and quality", "NVIDIA Hardware H.264, Balanced Speed & Quality"))
                    } else {
                        ("libx264".to_string(), get_reason("Software H.264, wide compatibility", "Software H.264, Wide Compatibility"))
                    }
                }
            },
            "mkv" => {
                // MKV format: supports all encoders, prefer high efficiency ones
                if is_high_quality {
                    // Prefer HEVC over AV1 for better compatibility
                    if available_hardware.contains(&"hevc_nvenc".to_string()) {
                        ("hevc_nvenc".to_string(), get_reason("NVIDIA hardware H.265, high quality and compatibility", "NVIDIA Hardware H.265, High Quality & Compatibility"))
                    } else if available_hardware.contains(&"av1_nvenc".to_string()) {
                        ("hevc_nvenc".to_string(), get_reason("H.265 encoder, better compatibility", "H.265 Encoder, Better Compatibility"))
                    } else {
                        ("libx265".to_string(), get_reason("Software H.265, best quality compression", "Software H.265, Best Quality Compression"))
                    }
                } else if is_fast {
                    if available_hardware.contains(&"h264_nvenc".to_string()) {
                        ("h264_nvenc".to_string(), get_reason("NVIDIA hardware H.264, fastest speed", "NVIDIA Hardware H.264, Fastest Speed"))
                    } else {
                        ("libx264".to_string(), get_reason("Software H.264, speed optimized", "Software H.264, Speed Optimized"))
                    }
                } else {
                    if available_hardware.contains(&"hevc_nvenc".to_string()) {
                        ("hevc_nvenc".to_string(), get_reason("NVIDIA hardware H.265, balanced choice", "NVIDIA Hardware H.265, Balanced Choice"))
                    } else {
                        ("libx265".to_string(), get_reason("Software H.265, quality optimized", "Software H.265, Quality Optimized"))
                    }
                }
            },
            "avi" => {
                // AVI format: conservative choice, prioritize compatibility
                if available_hardware.contains(&"h264_nvenc".to_string()) && speed_priority {
                    ("h264_nvenc".to_string(), get_reason("NVIDIA hardware H.264, AVI compatible", "NVIDIA Hardware H.264, AVI Compatible"))
                } else {
                    ("libx264".to_string(), get_reason("Software H.264, AVI standard choice", "Software H.264, AVI Standard Choice"))
                }
            },
            _ => {
                // Other formats: default to H.264
                if available_hardware.contains(&"h264_nvenc".to_string()) {
                    ("h264_nvenc".to_string(), get_reason("NVIDIA hardware H.264, universal choice", "NVIDIA Hardware H.264, Universal Choice"))
                } else {
                    ("libx264".to_string(), get_reason("Software H.264, universal compatible", "Software H.264, Universal Compatible"))
                }
            }
        }
    }

    /// Generate recommended encoder settings
    pub fn get_recommended_settings(
        _input_format: &str, 
        output_format: &str, 
        quality_target: &str
    ) -> (VideoSettings, AudioSettings) {
        let video_codec = Self::get_best_video_codec_for_format(output_format);
        let audio_codec = Self::get_best_audio_codec_for_format(output_format);
        
        let video_settings = match quality_target {
            "high" => VideoSettings {
                codec: video_codec,
                preset: "slow".to_string(),
                profile: "high".to_string(),
                tune: "auto".to_string(),
                quality: 18, // High quality CRF
                bitrate: "auto".to_string(),
                fps: "auto".to_string(),
                resolution: (0, 0),
                use_hardware_acceleration: false,
                custom_args: String::new(),
                ..Default::default()
            },
            "medium" => VideoSettings {
                codec: video_codec,
                preset: "medium".to_string(),
                profile: "high".to_string(),
                tune: "auto".to_string(),
                quality: 23, // Default quality
                bitrate: "auto".to_string(),
                fps: "auto".to_string(),
                resolution: (0, 0),
                use_hardware_acceleration: false,
                custom_args: String::new(),
                ..Default::default()
            },
            "fast" => VideoSettings {
                codec: video_codec,
                preset: "fast".to_string(),
                profile: "main".to_string(),
                tune: "auto".to_string(),
                quality: 28, // Faster encoding
                bitrate: "auto".to_string(),
                fps: "auto".to_string(),
                resolution: (0, 0),
                use_hardware_acceleration: true,
                custom_args: String::new(),
                ..Default::default()
            },
            _ => VideoSettings::default(),
        };
        
        let audio_settings = match quality_target {
            "high" => AudioSettings {
                codec: audio_codec.clone(),
                bitrate: if audio_codec == "flac" { "lossless".to_string() } else { "320k".to_string() },
                sample_rate: "48000".to_string(),
                channels: "auto".to_string(),
                volume: 1.0,
                quality: "2".to_string(), // High quality VBR
                custom_args: String::new(),
                ..Default::default()
            },
            "medium" => AudioSettings {
                codec: audio_codec.clone(),
                bitrate: "192k".to_string(),
                sample_rate: "44100".to_string(),
                channels: "auto".to_string(),
                volume: 1.0,
                quality: "4".to_string(),
                custom_args: String::new(),
                ..Default::default()
            },
            "fast" => AudioSettings {
                codec: audio_codec,
                bitrate: "128k".to_string(),
                sample_rate: "44100".to_string(),
                channels: "auto".to_string(),
                volume: 1.0,
                quality: "auto".to_string(),
                custom_args: String::new(),
                ..Default::default()
            },
            _ => AudioSettings::default(),
        };
        
        (video_settings, audio_settings)
    }

    /// Get audio codec description for tooltip
    pub fn get_audio_codec_description(codec: &str) -> &'static str {
        match codec {
            "libmp3lame" => "Most compatible, widely supported, good compression",
            "aac" => "High quality, efficient compression, modern standard",
            "flac" => "Lossless compression, larger files, perfect audio quality",
            "pcm_s16le" => "Uncompressed 16-bit, perfect quality",
            "pcm_s24le" => "Uncompressed 24-bit, professional quality",
            "pcm_s32le" => "Uncompressed 32-bit, highest precision",
            "pcm_f32le" => "32-bit float, professional audio production",
            "pcm_alaw" => "G.711 A-law, telephone quality, 64kbps",
            "pcm_mulaw" => "G.711 μ-law, telephone quality, 64kbps",
            "libopus" => "Best quality at low bitrates, modern, efficient",
            "libvorbis" => "Open source, good quality, OGG containers",
            "ac3" => "Dolby Digital, surround sound support, theater standard",
            "eac3" => "Dolby Digital Plus, enhanced surround sound, Blu-ray standard",
            "dts" => "DTS Digital Theater Systems, high-quality surround sound",
            "alac" => "Apple Lossless, perfect quality, iTunes compatible",
            "wmav2" => "Windows Media Audio v2, Microsoft standard",
            "libopencore_amrnb" => "AMR Narrowband, speech optimized, mobile standard",
            "libspeex" => "Speex speech codec, optimized for voice",
            "g722" => "G.722 wideband audio, VoIP standard",
            "truehd" => "Dolby TrueHD, lossless surround sound",
            "auto" => "Automatically selects best codec for output format",
            _ => "Unknown or specialized codec",
        }
    }

    /// Get video codec description for tooltip
    pub fn get_video_codec_description(codec: &str) -> &'static str {
        match codec {
            "libx264" => "Most compatible H.264, excellent balance of quality and compatibility",
            "libx265" => "Better compression than H.264, smaller files, requires more CPU",
            "libvpx" => "VP8 codec, good for web, royalty-free",
            "libvpx-vp9" => "Google's VP9 codec, excellent for web, royalty-free",
            "libaom-av1" => "Next-generation codec, best compression, slower encoding",
            "h264_nvenc" => "Hardware H.264 encoding, very fast with NVIDIA GPU",
            "hevc_nvenc" => "Hardware H.265 encoding, fast encoding with NVIDIA GPU", 
            "av1_nvenc" => "Hardware AV1 encoding, fast with latest NVIDIA GPUs",
            "vp9_nvenc" => "Hardware VP9 encoding, fast with latest NVIDIA GPUs",
            "h264_qsv" => "Intel QuickSync H.264, fast with Intel integrated graphics",
            "hevc_qsv" => "Intel QuickSync H.265, fast with Intel integrated graphics",
            "av1_qsv" => "Intel QuickSync AV1, fast with latest Intel graphics",
            "vp9_qsv" => "Intel QuickSync VP9, fast with latest Intel graphics",
            "h264_vaapi" => "Hardware H.264 encoding, VA-API support (Linux)",
            "hevc_vaapi" => "Hardware H.265 encoding, VA-API support (Linux)",
            "av1_vaapi" => "Hardware AV1 encoding, VA-API support (Linux)",
            "vp8_vaapi" => "Hardware VP8 encoding, VA-API support (Linux)",
            "vp9_vaapi" => "Hardware VP9 encoding, fast with VA-API support (Linux)",
            "h264_amf" => "AMD AMF H.264 encoding, fast with AMD GPUs",
            "hevc_amf" => "AMD AMF H.265 encoding, fast with AMD GPUs",
            "av1_amf" => "AMD AMF AV1 encoding, fast with latest AMD GPUs",
            "h264_videotoolbox" => "Apple VideoToolbox H.264, fast with Apple Silicon",
            "hevc_videotoolbox" => "Apple VideoToolbox H.265, fast with Apple Silicon",
            "prores_videotoolbox" => "Apple ProRes encoding, professional quality on macOS",
            "prores" => "Apple ProRes, professional quality, large files",
            "mjpeg" => "Motion JPEG, frame-based compression, good for editing",
            "ffv1" => "FFV1 lossless codec, perfect quality, large files",
            "huffyuv" => "HuffYUV lossless codec, fast compression",
            "libtheora" => "Theora codec, open source, royalty-free",
            "dvvideo" => "Digital Video codec, tape format standard",
            "mpeg2video" => "MPEG-2, DVD and broadcast standard",
            "mpeg1video" => "MPEG-1, legacy video standard",
            "wmv2" => "Windows Media Video v2, Microsoft standard",
            "h263" => "H.263, mobile and low-bitrate video",
            "flv" => "Flash Video codec, web streaming legacy",
            "dnxhd" => "DNxHD/DNxHR, professional editing codec",
            "gif" => "Animated GIF, web animations, limited colors",
            "auto" => "Automatically selects best codec for output format",
            _ => "Hardware or specialized codec",
        }
    }

    /// Get codec quality rating (1-5 stars)
    pub fn get_codec_quality_rating(codec: &str, is_audio: bool) -> u8 {
        if is_audio {
            match codec {
                "flac" | "pcm_s16le" | "pcm_s24le" => 5, // Lossless
                "libopus" => 5, // Best lossy compression
                "aac" => 4, // High quality
                "libvorbis" => 4, // High quality
                "libmp3lame" => 3, // Standard quality
                "ac3" => 3, // Standard quality
                _ => 3,
            }
        } else {
            match codec {
                "libaom-av1" => 5, // Latest and best
                "prores_videotoolbox" => 5, // ProRes professional quality
                "av1_nvenc" | "av1_qsv" | "av1_vaapi" | "av1_amf" => 4, // Hardware AV1, high quality
                "libx265" => 4, // High quality
                "hevc_nvenc" | "hevc_qsv" | "hevc_vaapi" | "hevc_amf" | "hevc_videotoolbox" => 4, // Hardware H.265
                "libvpx-vp9" => 4, // High quality
                "vp9_nvenc" | "vp9_qsv" | "vp9_vaapi" => 4, // Hardware VP9, high quality
                "libvpx" => 3, // VP8
                "vp8_vaapi" => 3, // Hardware VP8
                "libx264" => 4, // Standard high quality
                "h264_nvenc" | "h264_qsv" | "h264_vaapi" | "h264_amf" | "h264_videotoolbox" => 3, // Hardware H.264
                _ => 3,
            }
        }
    }

    /// Get codec speed rating (1-5 stars, 5 is fastest)
    pub fn get_codec_speed_rating(codec: &str, is_audio: bool) -> u8 {
        if is_audio {
            match codec {
                "copy" => 5, // No re-encoding, fastest
                "libopus" => 4, // Fast
                "aac" => 4, // Fast
                "libmp3lame" => 3, // Medium
                "libvorbis" => 3, // Medium
                "flac" => 2, // Slower
                "pcm_s16le" | "pcm_s24le" => 1, // Large file write slow
                _ => 3,
            }
        } else {
            match codec {
                // Hardware encoders - fastest
                "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" | 
                "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" |
                "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" | 
                "h264_amf" | "hevc_amf" | "av1_amf" |
                "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" => 5, // Hardware fastest
                "libx264" => 3, // Medium
                "libvpx" => 3, // VP8 medium
                "libx265" => 2, // Slower
                "libvpx-vp9" => 2, // Slower
                "libaom-av1" => 1, // Slowest
                _ => 3,
            }
        }
    }

    /// Smart hardware encoder recommendation
    pub fn recommend_hardware_codec(software_codec: &str, output_format: &str) -> Option<String> {
        // Detect available hardware encoders
        if let Ok(available_hw) = Self::detect_hardware_encoders() {
            match software_codec {
                "libx264" => {
                    // H.264 hardware encoder priority: NVENC > QuickSync > VA-API > AMF
                    if available_hw.contains(&"h264_nvenc".to_string()) {
                        Some("h264_nvenc".to_string())
                    } else if available_hw.contains(&"h264_qsv".to_string()) {
                        Some("h264_qsv".to_string())
                    } else if available_hw.contains(&"h264_vaapi".to_string()) {
                        Some("h264_vaapi".to_string())
                    } else if available_hw.contains(&"h264_amf".to_string()) {
                        Some("h264_amf".to_string())
                    } else {
                        None
                    }
                },
                "libx265" => {
                    // H.265 hardware encoder priority
                    if available_hw.contains(&"hevc_nvenc".to_string()) {
                        Some("hevc_nvenc".to_string())
                    } else if available_hw.contains(&"hevc_qsv".to_string()) {
                        Some("hevc_qsv".to_string())
                    } else if available_hw.contains(&"hevc_vaapi".to_string()) {
                        Some("hevc_vaapi".to_string())
                    } else if available_hw.contains(&"hevc_amf".to_string()) {
                        Some("hevc_amf".to_string())
                    } else {
                        None
                    }
                },
                "libaom-av1" => {
                    // AV1 hardware encoder priority
                    if available_hw.contains(&"av1_nvenc".to_string()) {
                        Some("av1_nvenc".to_string())
                    } else if available_hw.contains(&"av1_qsv".to_string()) {
                        Some("av1_qsv".to_string())
                    } else if available_hw.contains(&"av1_vaapi".to_string()) {
                        Some("av1_vaapi".to_string())
                    } else {
                        None
                    }
                },
                "libvpx-vp9" => {
                    // VP9 hardware encoder priority: NVENC > QuickSync > VA-API
                    if output_format == "webm" || output_format == "mkv" {
                        if available_hw.contains(&"vp9_nvenc".to_string()) {
                            Some("vp9_nvenc".to_string())
                        } else if available_hw.contains(&"vp9_qsv".to_string()) {
                            Some("vp9_qsv".to_string())
                        } else if available_hw.contains(&"vp9_vaapi".to_string()) {
                            Some("vp9_vaapi".to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                "libvpx" => {
                    // VP8 hardware encoder
                    if output_format == "webm" || output_format == "mkv" {
                        if available_hw.contains(&"vp8_vaapi".to_string()) {
                            Some("vp8_vaapi".to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                _ => None
            }
        } else {
            None
        }
    }

    /// Check if encoder is hardware encoder
    pub fn is_hardware_encoder(codec: &str) -> bool {
        matches!(codec, 
            // NVIDIA NVENC
            "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" |
            // Intel QuickSync
            "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" |
            // VA-API (Linux)
            "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" |
            // AMD AMF
            "h264_amf" | "hevc_amf" | "av1_amf" |
            // Apple VideoToolbox
            "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox"
        )
    }
}