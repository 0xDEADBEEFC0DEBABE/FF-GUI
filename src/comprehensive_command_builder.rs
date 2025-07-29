use crate::comprehensive_codec_registry::*;
use crate::app_state::{VideoSettings, AudioSettings};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Comprehensive FFmpeg command builder that supports all codecs and formats
pub struct ComprehensiveCommandBuilder;

impl ComprehensiveCommandBuilder {
    /// Build FFmpeg command for video conversion using comprehensive codec registry
    pub fn build_video_conversion_command(
        input_file: &str,
        output_file: &str,
        video_settings: &VideoSettings,
        audio_settings: Option<&AudioSettings>
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();
        
        // Input file
        args.push("-i".to_string());
        args.push(input_file.to_string());
        
        // Video codec and settings
        if video_settings.copy_video {
            args.push("-c:v".to_string());
            args.push("copy".to_string());
        } else {
            let video_codecs = ComprehensiveCodecRegistry::get_video_codecs();
            
            if let Some(codec_info) = video_codecs.get(&video_settings.codec) {
                // Validate codec-format compatibility
                if !ComprehensiveCodecRegistry::is_compatible(&video_settings.codec, &video_settings.container_format) {
                    return Err(anyhow!("Codec {} is not compatible with format {}", 
                        video_settings.codec, video_settings.container_format));
                }
                
                args.push("-c:v".to_string());
                args.push(video_settings.codec.clone());
                
                // Add codec-specific parameters
                Self::add_video_codec_parameters(&mut args, codec_info, video_settings)?;
            } else if video_settings.codec != "auto" {
                return Err(anyhow!("Unknown video codec: {}", video_settings.codec));
            }
        }
        
        // Audio codec and settings
        if let Some(audio_settings) = audio_settings {
            if audio_settings.copy_audio {
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            } else {
                let audio_codecs = ComprehensiveCodecRegistry::get_audio_codecs();
                
                if let Some(codec_info) = audio_codecs.get(&audio_settings.codec) {
                    // Validate codec-format compatibility
                    if !ComprehensiveCodecRegistry::is_compatible(&audio_settings.codec, &video_settings.container_format) {
                        return Err(anyhow!("Audio codec {} is not compatible with format {}", 
                            audio_settings.codec, video_settings.container_format));
                    }
                    
                    args.push("-c:a".to_string());
                    args.push(audio_settings.codec.clone());
                    
                    // Add codec-specific parameters
                    Self::add_audio_codec_parameters(&mut args, codec_info, audio_settings)?;
                } else if audio_settings.codec != "auto" {
                    return Err(anyhow!("Unknown audio codec: {}", audio_settings.codec));
                }
            }
        }
        
        // Video filters and transformations
        let mut video_filters = Vec::new();
        
        // Handle format conversion and scaling for NVENC with hardware acceleration
        if video_settings.codec.contains("nvenc") && video_settings.use_hardware_acceleration {
            // When using CUDA hardware acceleration, use CUDA-aware scaling and format conversion
            // First, use scale_cuda for CUDA memory scaling
            if let (Some(width), Some(height)) = (video_settings.width, video_settings.height) {
                if width > 0 && height > 0 {
                    video_filters.push(format!("scale_cuda={}:{}:format=yuv420p", width, height));
                } else {
                    // Just format conversion if no scaling needed
                    video_filters.push("scale_cuda=format=yuv420p".to_string());
                }
            } else {
                // Just format conversion if no scaling needed
                video_filters.push("scale_cuda=format=yuv420p".to_string());
            }
        } else {
            // Software path - add format conversion for NVENC if needed
            if video_settings.codec.contains("nvenc") {
                video_filters.push("format=yuv420p".to_string());
            }
            
            // Resolution for software path
            if let (Some(width), Some(height)) = (video_settings.width, video_settings.height) {
                if width > 0 && height > 0 {
                    video_filters.push(format!("scale={}:{}", width, height));
                }
            }
        }
        
        // Crop (skip for CUDA hardware acceleration path - would need hwdownload/hwupload)
        if video_settings.crop_top.is_some() || video_settings.crop_bottom.is_some() || 
           video_settings.crop_left.is_some() || video_settings.crop_right.is_some() {
            if !(video_settings.codec.contains("nvenc") && video_settings.use_hardware_acceleration) {
                let top = video_settings.crop_top.unwrap_or(0);
                let bottom = video_settings.crop_bottom.unwrap_or(0);
                let left = video_settings.crop_left.unwrap_or(0);
                let right = video_settings.crop_right.unwrap_or(0);
                video_filters.push(format!("crop=iw-{}-{}:ih-{}-{}:{}:{}", left, right, top, bottom, left, top));
            }
        }
        
        // For other filters, skip CUDA hardware acceleration path to avoid compatibility issues
        if !(video_settings.codec.contains("nvenc") && video_settings.use_hardware_acceleration) {
            // Rotation
            if video_settings.use_custom_rotation && video_settings.custom_rotation_angle != 0.0 {
                let angle_rad = video_settings.custom_rotation_angle * std::f32::consts::PI / 180.0;
                video_filters.push(format!("rotate={}", angle_rad));
            } else if video_settings.rotation != 0 {
                match video_settings.rotation {
                    90 => video_filters.push("transpose=1".to_string()),
                    180 => video_filters.push("transpose=2,transpose=2".to_string()),
                    270 => video_filters.push("transpose=2".to_string()),
                    _ => {}
                }
            }
            
            // Flip
            if video_settings.flip_horizontal {
                video_filters.push("hflip".to_string());
            }
            if video_settings.flip_vertical {
                video_filters.push("vflip".to_string());
            }
            
            // Color adjustments
            if video_settings.brightness != 0.0 || video_settings.contrast != 1.0 || video_settings.saturation != 1.0 {
                video_filters.push(format!("eq=brightness={}:contrast={}:saturation={}", 
                    video_settings.brightness, video_settings.contrast, video_settings.saturation));
            }
            
            // Filters
            if video_settings.denoise {
                video_filters.push("hqdn3d".to_string());
            }
            if video_settings.deinterlace {
                video_filters.push("yadif".to_string());
            }
            if video_settings.stabilize {
                video_filters.push("deshake".to_string());
            }
        }
        
        // Apply video filters
        if !video_filters.is_empty() {
            args.push("-vf".to_string());
            args.push(video_filters.join(","));
        }
        
        // Frame rate
        if video_settings.fps != "auto" && !video_settings.fps.is_empty() {
            args.push("-r".to_string());
            args.push(video_settings.fps.clone());
        }
        
        // Output file
        args.push("-y".to_string()); // Overwrite output file
        args.push(output_file.to_string());
        
        Ok(args)
    }
    
    /// Build FFmpeg command for audio conversion
    pub fn build_audio_conversion_command(
        input_file: &str,
        output_file: &str,
        audio_settings: &AudioSettings
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();
        
        // Input file
        args.push("-i".to_string());
        args.push(input_file.to_string());
        
        // Audio codec and settings
        if audio_settings.copy_audio {
            args.push("-c:a".to_string());
            args.push("copy".to_string());
        } else {
            let audio_codecs = ComprehensiveCodecRegistry::get_audio_codecs();
            
            if let Some(codec_info) = audio_codecs.get(&audio_settings.codec) {
                args.push("-c:a".to_string());
                args.push(audio_settings.codec.clone());
                
                // Add codec-specific parameters
                Self::add_audio_codec_parameters(&mut args, codec_info, audio_settings)?;
            } else if audio_settings.codec != "auto" {
                return Err(anyhow!("Unknown audio codec: {}", audio_settings.codec));
            }
        }
        
        // Audio filters
        let mut audio_filters = Vec::new();
        
        // Volume adjustment
        if audio_settings.volume != 1.0 {
            audio_filters.push(format!("volume={}", audio_settings.volume));
        }
        
        // Fade in/out
        if audio_settings.fade_in {
            audio_filters.push("afade=t=in:d=3".to_string());
        }
        if audio_settings.fade_out {
            audio_filters.push("afade=t=out:d=3".to_string());
        }
        
        // Apply audio filters
        if !audio_filters.is_empty() {
            args.push("-af".to_string());
            args.push(audio_filters.join(","));
        }
        
        // Sample rate
        if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
            args.push("-ar".to_string());
            args.push(audio_settings.sample_rate.clone());
        }
        
        // Channels
        if audio_settings.channels != "auto" && !audio_settings.channels.is_empty() {
            args.push("-ac".to_string());
            args.push(audio_settings.channels.clone());
        }
        
        // Time range (trim)
        if !audio_settings.start_time.is_empty() && audio_settings.start_time != "00:00:00" {
            args.push("-ss".to_string());
            args.push(audio_settings.start_time.clone());
        }
        if !audio_settings.end_time.is_empty() {
            args.push("-to".to_string());
            args.push(audio_settings.end_time.clone());
        }
        
        // Output file
        args.push("-y".to_string());
        args.push(output_file.to_string());
        
        Ok(args)
    }
    
    /// Add video codec-specific parameters
    fn add_video_codec_parameters(
        args: &mut Vec<String>,
        codec_info: &CodecInfo,
        settings: &VideoSettings
    ) -> Result<()> {
        // Quality/CRF settings
        if let Some((min_crf, max_crf)) = codec_info.quality_range {
            if settings.crf >= min_crf as i32 && settings.crf <= max_crf as i32 {
                args.push("-crf".to_string());
                args.push(settings.crf.to_string());
            }
        }
        
        // Preset
        if !codec_info.preset_options.is_empty() && settings.preset != "auto" {
            if codec_info.preset_options.contains(&settings.preset) {
                args.push("-preset".to_string());
                args.push(settings.preset.clone());
            }
        }
        
        // Profile
        if !codec_info.profile_options.is_empty() && settings.profile != "auto" {
            if codec_info.profile_options.contains(&settings.profile) {
                args.push("-profile:v".to_string());
                args.push(settings.profile.clone());
            }
        }
        
        // Level
        if !codec_info.level_options.is_empty() && settings.level != "auto" {
            if codec_info.level_options.contains(&settings.level) {
                args.push("-level".to_string());
                args.push(settings.level.clone());
            }
        }
        
        // Pixel format
        if !codec_info.supported_pixel_formats.is_empty() && settings.pixel_format != "auto" {
            if codec_info.supported_pixel_formats.contains(&settings.pixel_format) {
                args.push("-pix_fmt".to_string());
                args.push(settings.pixel_format.clone());
            }
        }
        
        // Bitrate (if not using CRF)
        if settings.bitrate != "auto" && !settings.bitrate.is_empty() {
            if codec_info.supported_bit_rates.contains(&settings.bitrate) {
                args.push("-b:v".to_string());
                args.push(settings.bitrate.clone());
            }
        }
        
        // Tune parameter
        if settings.tune != "auto" && !settings.tune.is_empty() {
            args.push("-tune".to_string());
            args.push(settings.tune.clone());
        }
        
        Ok(())
    }
    
    /// Add audio codec-specific parameters
    fn add_audio_codec_parameters(
        args: &mut Vec<String>,
        codec_info: &CodecInfo,
        settings: &AudioSettings
    ) -> Result<()> {
        // Bitrate
        if settings.bitrate != "auto" && !settings.bitrate.is_empty() {
            if codec_info.supported_bit_rates.contains(&settings.bitrate) {
                args.push("-b:a".to_string());
                args.push(settings.bitrate.clone());
            }
        }
        
        // VBR quality (for codecs that support it)
        if codec_info.supports_vbr && settings.vbr_quality > 0 {
            if let Some((min_quality, max_quality)) = codec_info.quality_range {
                if settings.vbr_quality >= min_quality as i32 && settings.vbr_quality <= max_quality as i32 {
                    match codec_info.name.as_str() {
                        "libmp3lame" => {
                            args.push("-q:a".to_string());
                            args.push(settings.vbr_quality.to_string());
                        },
                        "libvorbis" | "libopus" => {
                            args.push("-q:a".to_string());
                            args.push(settings.vbr_quality.to_string());
                        },
                        "aac" | "libfdk_aac" => {
                            args.push("-vbr".to_string());
                            args.push(settings.vbr_quality.to_string());
                        },
                        _ => {}
                    }
                }
            }
        }
        
        // Compression level (for lossless codecs)
        if codec_info.name == "flac" && codec_info.quality_range.is_some() {
            args.push("-compression_level".to_string());
            args.push(settings.vbr_quality.to_string());
        }
        
        Ok(())
    }
    
    /// Get recommended codec for a format
    pub fn get_recommended_codec(format: &str, codec_type: CodecType) -> Option<String> {
        let (video_codecs, audio_codecs) = ComprehensiveCodecRegistry::get_recommended_codecs_for_format(format);
        
        match codec_type {
            CodecType::Video => video_codecs.first().cloned(),
            CodecType::Audio => audio_codecs.first().cloned(),
            CodecType::Subtitle => None,
        }
    }
    
    /// Validate codec and format combination
    pub fn validate_codec_format_combination(codec: &str, format: &str) -> Result<()> {
        if !ComprehensiveCodecRegistry::is_compatible(codec, format) {
            return Err(anyhow!("Codec '{}' is not compatible with format '{}'", codec, format));
        }
        Ok(())
    }
    
    /// Get all compatible formats for a codec
    pub fn get_compatible_formats(codec: &str) -> Vec<String> {
        let compatibility = ComprehensiveCodecRegistry::get_codec_format_compatibility();
        compatibility.get(codec).cloned().unwrap_or_default()
    }
    
    /// Get all compatible codecs for a format
    pub fn get_compatible_codecs(format: &str, codec_type: CodecType) -> Vec<String> {
        let compatibility = ComprehensiveCodecRegistry::get_codec_format_compatibility();
        let mut compatible_codecs = Vec::new();
        
        let codecs = match codec_type {
            CodecType::Video => ComprehensiveCodecRegistry::get_video_codecs(),
            CodecType::Audio => ComprehensiveCodecRegistry::get_audio_codecs(),
            CodecType::Subtitle => HashMap::new(),
        };
        
        for (codec_name, _) in codecs {
            if let Some(compatible_formats) = compatibility.get(&codec_name) {
                if compatible_formats.contains(&format.to_string()) {
                    compatible_codecs.push(codec_name);
                }
            }
        }
        
        compatible_codecs
    }
}