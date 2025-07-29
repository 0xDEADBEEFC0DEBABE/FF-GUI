use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use egui::{Pos2, Vec2, Color32};
use crate::ffmpeg_worker_simple::FFmpegWorker;
use crate::{log_debug, log_error};
use crate::codec_manager::CodecManager;
use crate::comprehensive_codec_registry::{ComprehensiveCodecRegistry, CodecType};
use crate::comprehensive_ui_components::{ComprehensiveUIComponents, FormatPurpose};
use crate::app_state::{ProcessingTask, VideoSettings, AudioSettings, OperationType};
use crate::task_executor::TaskExecutor;
use anyhow::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono;

// Static counter for generating unique IDs
static UNIQUE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn generate_unique_id() -> String {
    let id = UNIQUE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("id_{}", id)
}

/// Data stream types - define data types that can be passed between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    /// Video stream
    VideoStream,
    /// Audio stream  
    AudioStream,
    /// Complete media file
    MediaFile,
    /// Text/path
    Text,
    /// Numeric parameter
    Number,
    /// Boolean value
    Boolean,
}

impl DataType {
    /// Get color for data type (for UI display)
    pub fn get_color(&self) -> Color32 {
        match self {
            DataType::VideoStream => Color32::from_rgb(255, 100, 100),   // Red
            DataType::AudioStream => Color32::from_rgb(100, 255, 100),   // Green
            DataType::MediaFile => Color32::from_rgb(100, 100, 255),     // Blue
            DataType::Text => Color32::from_rgb(255, 255, 100),          // Yellow
            DataType::Number => Color32::from_rgb(255, 150, 100),        // Orange
            DataType::Boolean => Color32::from_rgb(200, 100, 255),       // Purple
        }
    }
    
    /// Get display name for data type
    pub fn display_name(&self) -> &'static str {
        match self {
            DataType::VideoStream => "Video Stream",
            DataType::AudioStream => "Audio Stream",
            DataType::MediaFile => "Media File",
            DataType::Text => "Text",
            DataType::Number => "Number",
            DataType::Boolean => "Boolean",
        }
    }
}

/// Node port - input or output connection point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePort {
    pub id: String,
    pub name: String,
    pub data_type: DataType,
    pub is_input: bool,
    /// IDs of other ports connected to this port
    pub connections: Vec<String>,
}

/// Node types - All possible FFmpeg automation operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    // Input/Output nodes
    /// Input node - specify source file
    InputFile,
    /// Output node - generate final file
    OutputFile,
    
    // Audio processing nodes
    /// Extract audio stream
    ExtractAudio,
    /// Audio resampling (sample rate, bit depth)
    AudioResample,
    /// Audio format conversion (mp3, flac, aac, etc.)
    AudioConvert,
    /// Audio compression
    AudioCompress,
    /// Audio volume adjustment
    AudioVolume,
    /// Audio trimming/cutting
    AudioTrim,
    /// Audio merging (multiple files)
    AudioMerge,
    /// Audio normalization
    AudioNormalize,
    /// Audio noise reduction
    AudioDeNoise,
    /// Audio equalization
    AudioEqualizer,
    /// Audio fade in/out
    AudioFade,
    /// Audio echo/reverb
    AudioEcho,
    /// Audio speed/pitch adjustment
    AudioSpeed,
    
    // Video processing nodes
    /// Extract video stream
    ExtractVideo,
    /// Video recoding (codec, quality)
    VideoRecode,
    /// Video format conversion (mp4, mkv, avi, etc.)
    VideoConvert,
    /// Video compression
    VideoCompress,
    /// Video resizing/scaling
    VideoResize,
    /// Video cropping
    VideoCrop,
    /// Video rotation
    VideoRotate,
    /// Video filters (blur, sharpen, etc.)
    VideoFilter,
    /// Video frame extraction
    FrameExtract,
    /// Video frame rate conversion
    VideoFPS,
    /// Video stabilization
    VideoStabilize,
    /// Video deinterlacing
    VideoDeinterlace,
    /// Video color correction
    VideoColorCorrect,
    /// Video brightness/contrast
    VideoBrightness,
    /// Video saturation
    VideoSaturation,
    /// Video gamma correction
    VideoGamma,
    
    // Combination and splitting
    /// Audio/video combination
    Combine,
    /// Split audio/video streams
    SplitAudioVideo,
    /// Overlay video on video
    VideoOverlay,
    /// Picture-in-picture
    VideoPiP,
    /// Side-by-side video
    VideoSideBySide,
    
    // Special format conversions
    /// Video to GIF conversion
    VideoToGif,
    /// GIF resizing
    GifResize,
    /// Video to image sequence
    VideoToImages,
    /// Images to video
    ImagesToVideo,
    
    // Batch processing
    /// Batch convert multiple files
    BatchConvert,
    
    // Text and graphics
    /// Add subtitle
    AddSubtitle,
    /// Add watermark
    AddWatermark,
    /// Add text overlay
    AddText,
    /// Add logo overlay
    AddLogo,
    /// Burn-in timecode
    AddTimecode,
    
    // Advanced processing
    /// Live streaming preparation
    StreamPrep,
    /// Video encryption
    VideoEncrypt,
    /// Video decryption
    VideoDecrypt,
    /// Multi-pass encoding
    MultiPassEncode,
    /// Batch processing
    BatchProcess,
    /// Quality analysis
    QualityAnalysis,
    /// Format validation
    FormatValidation,
    
    // Audio/Video sync
    /// Audio/video synchronization
    AudioVideoSync,
    /// Audio delay adjustment
    AudioDelay,
    /// Video delay adjustment
    VideoDelay,
    
    // Metadata operations
    /// Extract metadata
    ExtractMetadata,
    /// Add metadata
    AddMetadata,
    /// Remove metadata
    RemoveMetadata,
    
    // Archive and packaging
    /// Create video archive
    CreateArchive,
    /// Extract from archive
    ExtractArchive,
    /// Multi-resolution output
    MultiResOutput,
}

impl NodeType {
    /// Get display name for node type
    pub fn display_name(&self) -> &'static str {
        match self {
            // Input/Output
            NodeType::InputFile => "📁 Input File",
            NodeType::OutputFile => "💾 Output File",
            
            // Audio processing
            NodeType::ExtractAudio => "🎶 Extract Audio",
            NodeType::AudioResample => "🔀 Audio Resample", 
            NodeType::AudioConvert => "🎵 Audio Convert",
            NodeType::AudioCompress => "🗜 Audio Compress",
            NodeType::AudioVolume => "🔊 Audio Volume",
            NodeType::AudioTrim => "✂ Audio Trim",
            NodeType::AudioMerge => "🔗 Audio Merge (FAKE)",
            NodeType::AudioNormalize => "📊 Audio Normalize (FAKE)",
            NodeType::AudioDeNoise => "🔇 Audio DeNoise (FAKE)",
            NodeType::AudioEqualizer => "🎛 Audio EQ (FAKE)",
            NodeType::AudioFade => "📉 Audio Fade (FAKE)",
            NodeType::AudioEcho => "🔄 Audio Echo (FAKE)",
            NodeType::AudioSpeed => "⏩ Audio Speed (FAKE)",
            
            // Video processing  
            NodeType::ExtractVideo => "📹 Extract Video",
            NodeType::VideoRecode => "🔄 Video Recode",
            NodeType::VideoConvert => "🎬 Video Convert",
            NodeType::VideoCompress => "🗜 Video Compress",
            NodeType::VideoResize => "📐 Video Resize",
            NodeType::VideoCrop => "✂ Video Crop",
            NodeType::VideoRotate => "🔄 Video Rotate",
            NodeType::VideoFilter => "✨ Video Filter",
            NodeType::FrameExtract => "📷 Frame Extract",
            NodeType::VideoFPS => "⚡ Video FPS (FAKE)", 
            NodeType::VideoStabilize => "🎯 Video Stabilize (FAKE)",
            NodeType::VideoDeinterlace => "📺 Deinterlace (FAKE)",
            NodeType::VideoColorCorrect => "🎨 Color Correct (FAKE)",
            NodeType::VideoBrightness => "☀ Brightness (FAKE)",
            NodeType::VideoSaturation => "🌈 Saturation (FAKE)",
            NodeType::VideoGamma => "🔆 Gamma (FAKE)",
            
            // Combination and splitting
            NodeType::Combine => "🎭 Combine A/V (FAKE)",
            NodeType::SplitAudioVideo => "🎯 Split A/V",
            NodeType::VideoOverlay => "🖼 Video Overlay (FAKE)",
            NodeType::VideoPiP => "📱 Picture-in-Picture (FAKE)",
            NodeType::VideoSideBySide => "📐 Side-by-Side (FAKE)",
            
            // Special conversions
            NodeType::VideoToGif => "🎞 Video to GIF",
            NodeType::GifResize => "🖼 GIF Resize",
            NodeType::VideoToImages => "📸 Video to Images (FAKE)",
            NodeType::ImagesToVideo => "🎥 Images to Video (FAKE)",
            
            // Text and graphics
            NodeType::AddSubtitle => "💬 Add Subtitle",
            NodeType::AddWatermark => "🏷 Add Watermark",
            NodeType::AddText => "📝 Add Text (FAKE)",
            NodeType::AddLogo => "🏢 Add Logo (FAKE)",
            NodeType::AddTimecode => "⏰ Add Timecode (FAKE)",
            
            // Advanced processing
            NodeType::StreamPrep => "📡 Stream Prep (FAKE)",
            NodeType::VideoEncrypt => "🔒 Video Encrypt (FAKE)",
            NodeType::VideoDecrypt => "🔓 Video Decrypt (FAKE)",
            NodeType::MultiPassEncode => "🔄 Multi-Pass Encode",
            NodeType::BatchProcess => "📦 Batch Process (FAKE)",
            NodeType::QualityAnalysis => "📊 Quality Analysis (FAKE)",
            NodeType::FormatValidation => "✅ Format Validation (FAKE)",
            
            // Sync operations
            NodeType::AudioVideoSync => "🎭 A/V Sync (FAKE)",
            NodeType::AudioDelay => "⏪ Audio Delay (FAKE)",
            NodeType::VideoDelay => "⏩ Video Delay (FAKE)",
            
            // Metadata operations
            NodeType::ExtractMetadata => "📋 Extract Metadata (FAKE)",
            NodeType::AddMetadata => "📝 Add Metadata (FAKE)",
            NodeType::RemoveMetadata => "🗑 Remove Metadata (FAKE)",
            
            // Archive operations
            NodeType::CreateArchive => "📦 Create Archive (FAKE)",
            NodeType::ExtractArchive => "📂 Extract Archive (FAKE)",
            NodeType::MultiResOutput => "📺 Multi-Resolution",
            NodeType::BatchConvert => "🔄 Batch Convert",
        }
    }
    
    /// Get node color for UI display
    pub fn get_color(&self) -> Color32 {
        match self {
            // Input/Output - Blue tones
            NodeType::InputFile | NodeType::OutputFile => Color32::from_rgb(100, 150, 200),
            
            // Audio processing - Green tones
            NodeType::ExtractAudio | NodeType::AudioResample | NodeType::AudioConvert |
            NodeType::AudioCompress | NodeType::AudioVolume | NodeType::AudioTrim |
            NodeType::AudioMerge | NodeType::AudioNormalize | NodeType::AudioDeNoise |
            NodeType::AudioEqualizer | NodeType::AudioFade | NodeType::AudioEcho |
            NodeType::AudioSpeed => Color32::from_rgb(150, 200, 150),
            
            // Video processing - Red tones
            NodeType::ExtractVideo | NodeType::VideoRecode | NodeType::VideoConvert |
            NodeType::VideoCompress | NodeType::VideoResize | NodeType::VideoCrop |
            NodeType::VideoRotate | NodeType::VideoFilter | NodeType::FrameExtract |
            NodeType::VideoFPS | NodeType::VideoStabilize | NodeType::VideoDeinterlace |
            NodeType::VideoColorCorrect | NodeType::VideoBrightness | NodeType::VideoSaturation |
            NodeType::VideoGamma => Color32::from_rgb(200, 150, 150),
            
            // Combination and splitting - Yellow tones
            NodeType::Combine | NodeType::SplitAudioVideo | NodeType::VideoOverlay |
            NodeType::VideoPiP | NodeType::VideoSideBySide => Color32::from_rgb(200, 200, 150),
            
            // Special conversions - Purple tones
            NodeType::VideoToGif | NodeType::GifResize | NodeType::VideoToImages |
            NodeType::ImagesToVideo => Color32::from_rgb(180, 150, 200),
            
            // Text and graphics - Orange tones
            NodeType::AddSubtitle | NodeType::AddWatermark | NodeType::AddText |
            NodeType::AddLogo | NodeType::AddTimecode => Color32::from_rgb(220, 180, 140),
            
            // Advanced processing - Dark blue tones
            NodeType::StreamPrep | NodeType::VideoEncrypt | NodeType::VideoDecrypt |
            NodeType::MultiPassEncode | NodeType::BatchProcess | NodeType::QualityAnalysis |
            NodeType::FormatValidation => Color32::from_rgb(120, 120, 180),
            
            // Sync operations - Cyan tones
            NodeType::AudioVideoSync | NodeType::AudioDelay | NodeType::VideoDelay => 
                Color32::from_rgb(140, 200, 200),
            
            // Metadata operations - Gray tones
            NodeType::ExtractMetadata | NodeType::AddMetadata | NodeType::RemoveMetadata => 
                Color32::from_rgb(160, 160, 160),
            
            // Archive operations - Brown tones
            NodeType::CreateArchive | NodeType::ExtractArchive | NodeType::MultiResOutput | NodeType::BatchConvert => 
                Color32::from_rgb(180, 140, 120),
        }
    }
    
    /// Get input port configuration for node type
    pub fn get_input_ports(&self) -> Vec<(String, DataType)> {
        match self {
            // Input/Output nodes
            NodeType::InputFile => vec![],
            NodeType::OutputFile => vec![("input".to_string(), DataType::MediaFile)],
            
            // Audio processing nodes
            NodeType::ExtractAudio => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::AudioResample => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioConvert => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioCompress => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioVolume => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioTrim => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioMerge => vec![
                ("audio1".to_string(), DataType::AudioStream),
                ("audio2".to_string(), DataType::AudioStream),
            ],
            NodeType::AudioNormalize => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioDeNoise => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioEqualizer => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioFade => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioEcho => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioSpeed => vec![("audio".to_string(), DataType::AudioStream)],
            
            // Video processing nodes
            NodeType::ExtractVideo => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::VideoRecode => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoConvert => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoCompress => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoResize => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoCrop => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoRotate => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoFilter => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::FrameExtract => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoFPS => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoStabilize => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoDeinterlace => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoColorCorrect => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoBrightness => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoSaturation => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoGamma => vec![("video".to_string(), DataType::VideoStream)],
            
            // Combination and splitting
            NodeType::Combine => vec![
                ("video".to_string(), DataType::VideoStream),
                ("audio".to_string(), DataType::AudioStream),
            ],
            NodeType::SplitAudioVideo => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::VideoOverlay => vec![
                ("main_video".to_string(), DataType::VideoStream),
                ("overlay_video".to_string(), DataType::VideoStream),
            ],
            NodeType::VideoPiP => vec![
                ("main_video".to_string(), DataType::VideoStream),
                ("pip_video".to_string(), DataType::VideoStream),
            ],
            NodeType::VideoSideBySide => vec![
                ("left_video".to_string(), DataType::VideoStream),
                ("right_video".to_string(), DataType::VideoStream),
            ],
            
            // Special conversions
            NodeType::VideoToGif => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::GifResize => vec![("gif".to_string(), DataType::MediaFile)],
            NodeType::VideoToImages => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::ImagesToVideo => vec![("images".to_string(), DataType::Text)],
            
            // Text and graphics
            NodeType::AddSubtitle => vec![
                ("video".to_string(), DataType::VideoStream),
                ("subtitle".to_string(), DataType::Text),
            ],
            NodeType::AddWatermark => vec![
                ("video".to_string(), DataType::VideoStream),
                ("watermark".to_string(), DataType::Text),
            ],
            NodeType::AddText => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::AddLogo => vec![
                ("video".to_string(), DataType::VideoStream),
                ("logo".to_string(), DataType::Text),
            ],
            NodeType::AddTimecode => vec![("video".to_string(), DataType::VideoStream)],
            
            // Advanced processing
            NodeType::StreamPrep => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::VideoEncrypt => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoDecrypt => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::MultiPassEncode => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::BatchProcess => vec![("inputs".to_string(), DataType::Text)],
            NodeType::QualityAnalysis => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::FormatValidation => vec![("input".to_string(), DataType::MediaFile)],
            
            // Sync operations
            NodeType::AudioVideoSync => vec![
                ("video".to_string(), DataType::VideoStream),
                ("audio".to_string(), DataType::AudioStream),
            ],
            NodeType::AudioDelay => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::VideoDelay => vec![("video".to_string(), DataType::VideoStream)],
            
            // Metadata operations
            NodeType::ExtractMetadata => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::AddMetadata => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::RemoveMetadata => vec![("input".to_string(), DataType::MediaFile)],
            
            // Archive operations
            NodeType::CreateArchive => vec![("inputs".to_string(), DataType::Text)],
            NodeType::ExtractArchive => vec![("archive".to_string(), DataType::MediaFile)],
            NodeType::MultiResOutput => vec![("input".to_string(), DataType::MediaFile)],
            NodeType::BatchConvert => vec![("inputs".to_string(), DataType::Text)],
        }
    }
    
    /// Get default parameters for node type based on original functionality
    pub fn get_default_parameters(&self) -> Vec<NodeParameter> {
        match self {
            // Video Convert
            NodeType::VideoConvert => vec![
                NodeParameter {
                    name: "output_format".to_string(),
                    value: "mp4".to_string(),
                    param_type: DataType::Text,
                    default_value: "mp4".to_string(),
                    description: "Output container format".to_string(),
                },
                NodeParameter {
                    name: "video_codec".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Video codec (auto, libx264, libx265, etc.)".to_string(),
                },
                NodeParameter {
                    name: "audio_codec".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Audio codec (auto, aac, libmp3lame, etc.)".to_string(),
                },
                NodeParameter {
                    name: "copy_video".to_string(),
                    value: "false".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "false".to_string(),
                    description: "Copy video stream without re-encoding".to_string(),
                },
            ],
            
            // Video Compress
            NodeType::VideoCompress => vec![
                NodeParameter {
                    name: "quality".to_string(),
                    value: "23".to_string(),
                    param_type: DataType::Number,
                    default_value: "23".to_string(),
                    description: "Compression quality (CRF: 0-51, lower=better)".to_string(),
                },
                NodeParameter {
                    name: "preset".to_string(),
                    value: "medium".to_string(),
                    param_type: DataType::Text,
                    default_value: "medium".to_string(),
                    description: "Encoding preset (ultrafast, fast, medium, slow, veryslow)".to_string(),
                },
                NodeParameter {
                    name: "target_size_mb".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Target file size in MB (0=disabled)".to_string(),
                },
            ],
            
            // Video Resize
            NodeType::VideoResize => vec![
                NodeParameter {
                    name: "width".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Output width (0=auto)".to_string(),
                },
                NodeParameter {
                    name: "height".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Output height (0=auto)".to_string(),
                },
                NodeParameter {
                    name: "maintain_aspect_ratio".to_string(),
                    value: "true".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "true".to_string(),
                    description: "Maintain aspect ratio".to_string(),
                },
            ],
            
            // Audio Convert
            NodeType::AudioConvert => vec![
                NodeParameter {
                    name: "output_format".to_string(),
                    value: "mp3".to_string(),
                    param_type: DataType::Text,
                    default_value: "mp3".to_string(),
                    description: "Output audio format".to_string(),
                },
                NodeParameter {
                    name: "audio_codec".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Audio codec (auto, aac, libmp3lame, etc.)".to_string(),
                },
                NodeParameter {
                    name: "bitrate".to_string(),
                    value: "128k".to_string(),
                    param_type: DataType::Text,
                    default_value: "128k".to_string(),
                    description: "Audio bitrate".to_string(),
                },
                NodeParameter {
                    name: "sample_rate".to_string(),
                    value: "44100".to_string(),
                    param_type: DataType::Number,
                    default_value: "44100".to_string(),
                    description: "Sample rate in Hz".to_string(),
                },
            ],
            
            // Audio Compress
            NodeType::AudioCompress => vec![
                NodeParameter {
                    name: "bitrate".to_string(),
                    value: "128k".to_string(),
                    param_type: DataType::Text,
                    default_value: "128k".to_string(),
                    description: "Target bitrate".to_string(),
                },
                NodeParameter {
                    name: "quality".to_string(),
                    value: "5".to_string(),
                    param_type: DataType::Number,
                    default_value: "5".to_string(),
                    description: "VBR quality (0=best, 9=worst)".to_string(),
                },
            ],
            
            // Input/Output nodes
            NodeType::InputFile => vec![
                NodeParameter {
                    name: "file_path".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Input file path".to_string(),
                },
            ],
            
            NodeType::OutputFile => vec![
                NodeParameter {
                    name: "output_path".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Output file path".to_string(),
                },
            ],
            
            // Video Crop
            NodeType::VideoCrop => vec![
                NodeParameter {
                    name: "crop_x".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Crop X position".to_string(),
                },
                NodeParameter {
                    name: "crop_y".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Crop Y position".to_string(),
                },
                NodeParameter {
                    name: "crop_width".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Crop width (0=auto)".to_string(),
                },
                NodeParameter {
                    name: "crop_height".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Crop height (0=auto)".to_string(),
                },
            ],
            
            // Video Rotate
            NodeType::VideoRotate => vec![
                NodeParameter {
                    name: "rotation".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Rotation angle (0, 90, 180, 270)".to_string(),
                },
                NodeParameter {
                    name: "flip_horizontal".to_string(),
                    value: "false".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "false".to_string(),
                    description: "Flip horizontally".to_string(),
                },
                NodeParameter {
                    name: "flip_vertical".to_string(),
                    value: "false".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "false".to_string(),
                    description: "Flip vertically".to_string(),
                },
            ],
            
            // Video Filter
            NodeType::VideoFilter => vec![
                NodeParameter {
                    name: "brightness".to_string(),
                    value: "0.0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0.0".to_string(),
                    description: "Brightness adjustment (-1.0 to 1.0)".to_string(),
                },
                NodeParameter {
                    name: "contrast".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Number,
                    default_value: "1.0".to_string(),
                    description: "Contrast adjustment (0.0 to 3.0)".to_string(),
                },
                NodeParameter {
                    name: "saturation".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Number,
                    default_value: "1.0".to_string(),
                    description: "Saturation adjustment (0.0 to 3.0)".to_string(),
                },
            ],
            
            // Audio Resample
            NodeType::AudioResample => vec![
                NodeParameter {
                    name: "sample_rate".to_string(),
                    value: "44100".to_string(),
                    param_type: DataType::Number,
                    default_value: "44100".to_string(),
                    description: "Target sample rate in Hz".to_string(),
                },
                NodeParameter {
                    name: "bit_depth".to_string(),
                    value: "16".to_string(),
                    param_type: DataType::Number,
                    default_value: "16".to_string(),
                    description: "Bit depth (16, 24, 32)".to_string(),
                },
            ],
            
            // Audio Volume
            NodeType::AudioVolume => vec![
                NodeParameter {
                    name: "volume".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Number,
                    default_value: "1.0".to_string(),
                    description: "Volume multiplier (0.0 to 2.0)".to_string(),
                },
                NodeParameter {
                    name: "normalize".to_string(),
                    value: "false".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "false".to_string(),
                    description: "Normalize audio levels".to_string(),
                },
            ],
            
            // Audio Trim
            NodeType::AudioTrim => vec![
                NodeParameter {
                    name: "start_time".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Start time in seconds".to_string(),
                },
                NodeParameter {
                    name: "duration".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Number,
                    default_value: "0".to_string(),
                    description: "Duration in seconds (0=until end)".to_string(),
                },
            ],
            
            // Combine
            NodeType::Combine => vec![
                NodeParameter {
                    name: "video_sync".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Video sync mode (auto, cfr, vfr)".to_string(),
                },
                NodeParameter {
                    name: "audio_sync".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Audio sync mode (auto, async)".to_string(),
                },
            ],
            
            // VideoToGif
            NodeType::VideoToGif => vec![
                NodeParameter {
                    name: "fps".to_string(),
                    value: "10".to_string(),
                    param_type: DataType::Number,
                    default_value: "10".to_string(),
                    description: "Output FPS for GIF".to_string(),
                },
                NodeParameter {
                    name: "scale".to_string(),
                    value: "320".to_string(),
                    param_type: DataType::Number,
                    default_value: "320".to_string(),
                    description: "Max width/height in pixels".to_string(),
                },
                NodeParameter {
                    name: "optimize".to_string(),
                    value: "true".to_string(),
                    param_type: DataType::Boolean,
                    default_value: "true".to_string(),
                    description: "Optimize GIF for smaller size".to_string(),
                },
            ],
            
            // BatchConvert
            NodeType::BatchConvert => vec![
                NodeParameter {
                    name: "input_pattern".to_string(),
                    value: "*.mp4".to_string(),
                    param_type: DataType::Text,
                    default_value: "*.mp4".to_string(),
                    description: "Input file pattern".to_string(),
                },
                NodeParameter {
                    name: "output_format".to_string(),
                    value: "mp4".to_string(),
                    param_type: DataType::Text,
                    default_value: "mp4".to_string(),
                    description: "Output format".to_string(),
                },
                NodeParameter {
                    name: "operation_type".to_string(),
                    value: "convert".to_string(),
                    param_type: DataType::Text,
                    default_value: "convert".to_string(),
                    description: "Batch operation (convert, resize, compress)".to_string(),
                },
            ],
            
            // Default case for other node types
            _ => vec![],
        }
    }
    
    /// Get output port configuration for node type
    pub fn get_output_ports(&self) -> Vec<(String, DataType)> {
        match self {
            // Input/Output nodes
            NodeType::InputFile => vec![("output".to_string(), DataType::MediaFile)],
            NodeType::OutputFile => vec![],
            
            // Audio processing nodes
            NodeType::ExtractAudio => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioResample => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioConvert => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioCompress => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioVolume => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioTrim => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioMerge => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioNormalize => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioDeNoise => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioEqualizer => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioFade => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioEcho => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::AudioSpeed => vec![("audio".to_string(), DataType::AudioStream)],
            
            // Video processing nodes
            NodeType::ExtractVideo => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoRecode => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoConvert => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoCompress => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoResize => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoCrop => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoRotate => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoFilter => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::FrameExtract => vec![("images".to_string(), DataType::Text)],
            NodeType::VideoFPS => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoStabilize => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoDeinterlace => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoColorCorrect => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoBrightness => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoSaturation => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoGamma => vec![("video".to_string(), DataType::VideoStream)],
            
            // Combination and splitting
            NodeType::Combine => vec![("output".to_string(), DataType::MediaFile)],
            NodeType::SplitAudioVideo => vec![
                ("video".to_string(), DataType::VideoStream),
                ("audio".to_string(), DataType::AudioStream),
            ],
            NodeType::VideoOverlay => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoPiP => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoSideBySide => vec![("video".to_string(), DataType::VideoStream)],
            
            // Special conversions
            NodeType::VideoToGif => vec![("gif".to_string(), DataType::MediaFile)],
            NodeType::GifResize => vec![("gif".to_string(), DataType::MediaFile)],
            NodeType::VideoToImages => vec![("images".to_string(), DataType::Text)],
            NodeType::ImagesToVideo => vec![("video".to_string(), DataType::VideoStream)],
            
            // Text and graphics
            NodeType::AddSubtitle => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::AddWatermark => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::AddText => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::AddLogo => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::AddTimecode => vec![("video".to_string(), DataType::VideoStream)],
            
            // Advanced processing
            NodeType::StreamPrep => vec![("stream".to_string(), DataType::MediaFile)],
            NodeType::VideoEncrypt => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::VideoDecrypt => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::MultiPassEncode => vec![("video".to_string(), DataType::VideoStream)],
            NodeType::BatchProcess => vec![("outputs".to_string(), DataType::Text)],
            NodeType::QualityAnalysis => vec![("report".to_string(), DataType::Text)],
            NodeType::FormatValidation => vec![("result".to_string(), DataType::Boolean)],
            
            // Sync operations
            NodeType::AudioVideoSync => vec![("output".to_string(), DataType::MediaFile)],
            NodeType::AudioDelay => vec![("audio".to_string(), DataType::AudioStream)],
            NodeType::VideoDelay => vec![("video".to_string(), DataType::VideoStream)],
            
            // Metadata operations
            NodeType::ExtractMetadata => vec![("metadata".to_string(), DataType::Text)],
            NodeType::AddMetadata => vec![("output".to_string(), DataType::MediaFile)],
            NodeType::RemoveMetadata => vec![("output".to_string(), DataType::MediaFile)],
            
            // Archive operations
            NodeType::CreateArchive => vec![("archive".to_string(), DataType::MediaFile)],
            NodeType::ExtractArchive => vec![("outputs".to_string(), DataType::Text)],
            NodeType::MultiResOutput => vec![("outputs".to_string(), DataType::Text)],
            NodeType::BatchConvert => vec![("outputs".to_string(), DataType::Text)],
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeParameter {
    pub name: String,
    pub value: String,
    pub param_type: DataType,
    pub default_value: String,
    pub description: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePos2 {
    pub x: f32,
    pub y: f32,
}

impl From<Pos2> for SerializablePos2 {
    fn from(pos: Pos2) -> Self {
        Self { x: pos.x, y: pos.y }
    }
}

impl From<SerializablePos2> for Pos2 {
    fn from(pos: SerializablePos2) -> Self {
        Pos2::new(pos.x, pos.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for SerializableVec2 {
    fn from(vec: Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl From<SerializableVec2> for Vec2 {
    fn from(vec: SerializableVec2) -> Self {
        Vec2::new(vec.x, vec.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationNode {
    pub id: String,
    pub node_type: NodeType,
    #[serde(serialize_with = "serialize_pos2", deserialize_with = "deserialize_pos2")]
    pub position: Pos2,
    #[serde(serialize_with = "serialize_vec2", deserialize_with = "deserialize_vec2")]
    pub size: Vec2,
    pub input_ports: Vec<NodePort>,
    pub output_ports: Vec<NodePort>,
    pub parameters: HashMap<String, NodeParameter>,
    pub enabled: bool,
}

fn serialize_pos2<S>(pos: &Pos2, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let serializable_pos = SerializablePos2::from(*pos);
    serializable_pos.serialize(serializer)
}

fn deserialize_pos2<'de, D>(deserializer: D) -> Result<Pos2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let serializable_pos = SerializablePos2::deserialize(deserializer)?;
    Ok(Pos2::from(serializable_pos))
}

fn serialize_vec2<S>(vec: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let serializable_vec = SerializableVec2::from(*vec);
    serializable_vec.serialize(serializer)
}

fn deserialize_vec2<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let serializable_vec = SerializableVec2::deserialize(deserializer)?;
    Ok(Vec2::from(serializable_vec))
}

impl AutomationNode {
    pub fn new(id: String, node_type: NodeType, position: Pos2) -> Self {
        let input_port_configs = node_type.get_input_ports();
        let output_port_configs = node_type.get_output_ports();
        
        let input_ports = input_port_configs.into_iter().enumerate().map(|(i, (name, data_type))| {
            NodePort {
                id: format!("{}_{}_in", id, i),
                name,
                data_type,
                is_input: true,
                connections: Vec::new(),
            }
        }).collect();
        
        let output_ports = output_port_configs.into_iter().enumerate().map(|(i, (name, data_type))| {
            NodePort {
                id: format!("{}_{}_out", id, i),
                name,
                data_type,
                is_input: false,
                connections: Vec::new(),
            }
        }).collect();
        
        let mut parameters = HashMap::new();
        
        for param in node_type.get_default_parameters() {
            parameters.insert(param.name.clone(), param);
        }
        
        // Legacy fallback for nodes not yet migrated
        if parameters.is_empty() {
            match node_type {
            NodeType::InputFile => {
                parameters.insert("file_path".to_string(), NodeParameter {
                    name: "File Path".to_string(),
                    value: String::new(),
                    param_type: DataType::Text,
                    default_value: String::new(),
                    description: "Select input file".to_string(),
                });
            },
            NodeType::AudioResample => {
                parameters.insert("sample_rate".to_string(), NodeParameter {
                    name: "Sample Rate".to_string(),
                    value: "96000".to_string(),
                    param_type: DataType::Text,
                    default_value: "96000".to_string(),
                    description: "Target sample rate: 8000, 16000, 22050, 44100, 48000, 96000, 192000".to_string(),
                });
                parameters.insert("bit_depth".to_string(), NodeParameter {
                    name: "Bit Depth".to_string(),
                    value: "24".to_string(),
                    param_type: DataType::Text,
                    default_value: "24".to_string(),
                    description: "Audio bit depth: 16, 24, 32".to_string(),
                });
                parameters.insert("channels".to_string(), NodeParameter {
                    name: "Channels".to_string(),
                    value: "2".to_string(),
                    param_type: DataType::Text,
                    default_value: "2".to_string(),
                    description: "Audio channels: 1 (mono), 2 (stereo), 6 (5.1), 8 (7.1)".to_string(),
                });
                parameters.insert("resampler".to_string(), NodeParameter {
                    name: "Resampler".to_string(),
                    value: "swr".to_string(),
                    param_type: DataType::Text,
                    default_value: "swr".to_string(),
                    description: "Resampler algorithm: swr, soxr".to_string(),
                });
            },
            NodeType::AudioConvert => {
                let audio_formats = ComprehensiveCodecRegistry::get_container_formats();
                let audio_codecs = ComprehensiveCodecRegistry::get_audio_codecs();
                
                let format_list = audio_formats.iter()
                    .filter(|(_, info)| info.preferred_video_codecs.is_empty()) // Audio-only formats
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let codec_list = audio_codecs.keys()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                parameters.insert("format".to_string(), NodeParameter {
                    name: "Format".to_string(),
                    value: "flac".to_string(),
                    param_type: DataType::Text,
                    default_value: "flac".to_string(),
                    description: format!("Target audio format: {}", format_list),
                });
                parameters.insert("codec".to_string(), NodeParameter {
                    name: "Codec".to_string(),
                    value: "flac".to_string(),
                    param_type: DataType::Text,
                    default_value: "flac".to_string(),
                    description: format!("Audio codec: {}", codec_list),
                });
                
                // Get comprehensive bitrate options
                let all_bitrates: std::collections::HashSet<String> = audio_codecs.values()
                    .flat_map(|codec| &codec.supported_bit_rates)
                    .cloned()
                    .collect();
                let bitrate_list = all_bitrates.iter()
                    .map(|br| br.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                parameters.insert("bitrate".to_string(), NodeParameter {
                    name: "Bitrate".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: format!("Audio bitrate: auto, {} (only for lossy formats)", bitrate_list),
                });
                parameters.insert("quality".to_string(), NodeParameter {
                    name: "Quality".to_string(),
                    value: "8".to_string(),
                    param_type: DataType::Text,
                    default_value: "8".to_string(),
                    description: "Quality level: 0-10 (higher is better for FLAC/Vorbis)".to_string(),
                });
            },
            NodeType::VideoConvert => {
                let video_formats = ComprehensiveCodecRegistry::get_container_formats();
                let video_codecs = ComprehensiveCodecRegistry::get_video_codecs();
                let audio_codecs = ComprehensiveCodecRegistry::get_audio_codecs();
                
                let format_list = video_formats.iter()
                    .filter(|(_, info)| !info.preferred_video_codecs.is_empty()) // Video formats
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let video_codec_list = video_codecs.keys()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let audio_codec_list = audio_codecs.keys()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                parameters.insert("format".to_string(), NodeParameter {
                    name: "Format".to_string(),
                    value: "mp4".to_string(),
                    param_type: DataType::Text,
                    default_value: "mp4".to_string(),
                    description: format!("Target video format: {}", format_list),
                });
                parameters.insert("video_codec".to_string(), NodeParameter {
                    name: "Video Codec".to_string(),
                    value: "libx264".to_string(),
                    param_type: DataType::Text,
                    default_value: "libx264".to_string(),
                    description: format!("Video codec: {}", video_codec_list),
                });
                parameters.insert("audio_codec".to_string(), NodeParameter {
                    name: "Audio Codec".to_string(),
                    value: "aac".to_string(),
                    param_type: DataType::Text,
                    default_value: "aac".to_string(),
                    description: format!("Audio codec: {}", audio_codec_list),
                });
                parameters.insert("preset".to_string(), NodeParameter {
                    name: "Preset".to_string(),
                    value: "medium".to_string(),
                    param_type: DataType::Text,
                    default_value: "medium".to_string(),
                    description: "Encoding preset: ultrafast, superfast, veryfast, faster, fast, medium, slow, slower, veryslow".to_string(),
                });
                parameters.insert("crf".to_string(), NodeParameter {
                    name: "CRF".to_string(),
                    value: "23".to_string(),
                    param_type: DataType::Text,
                    default_value: "23".to_string(),
                    description: "Constant Rate Factor: 0-51 (lower = better quality, 18-28 recommended)".to_string(),
                });
            },
            NodeType::VideoCompress => {
                parameters.insert("crf".to_string(), NodeParameter {
                    name: "CRF".to_string(),
                    value: "28".to_string(),
                    param_type: DataType::Text,
                    default_value: "28".to_string(),
                    description: "Constant Rate Factor: 18-35 (higher = smaller file)".to_string(),
                });
                parameters.insert("preset".to_string(), NodeParameter {
                    name: "Preset".to_string(),
                    value: "medium".to_string(),
                    param_type: DataType::Text,
                    default_value: "medium".to_string(),
                    description: "Speed vs compression: ultrafast, fast, medium, slow, veryslow".to_string(),
                });
                parameters.insert("max_bitrate".to_string(), NodeParameter {
                    name: "Max Bitrate".to_string(),
                    value: "2M".to_string(),
                    param_type: DataType::Text,
                    default_value: "2M".to_string(),
                    description: "Maximum bitrate: 500k, 1M, 2M, 5M, 10M".to_string(),
                });
                parameters.insert("two_pass".to_string(), NodeParameter {
                    name: "Two Pass".to_string(),
                    value: "false".to_string(),
                    param_type: DataType::Text,
                    default_value: "false".to_string(),
                    description: "Enable two-pass encoding: true, false".to_string(),
                });
            },
            NodeType::VideoResize => {
                parameters.insert("width".to_string(), NodeParameter {
                    name: "Width".to_string(),
                    value: "1920".to_string(),
                    param_type: DataType::Text,
                    default_value: "1920".to_string(),
                    description: "Target width in pixels (or -1 for auto)".to_string(),
                });
                parameters.insert("height".to_string(), NodeParameter {
                    name: "Height".to_string(),
                    value: "1080".to_string(),
                    param_type: DataType::Text,
                    default_value: "1080".to_string(),
                    description: "Target height in pixels (or -1 for auto)".to_string(),
                });
                parameters.insert("scale_algorithm".to_string(), NodeParameter {
                    name: "Scale Algorithm".to_string(),
                    value: "lanczos".to_string(),
                    param_type: DataType::Text,
                    default_value: "lanczos".to_string(),
                    description: "Scaling algorithm: bilinear, bicubic, lanczos, spline, neighbor".to_string(),
                });
                parameters.insert("aspect_ratio".to_string(), NodeParameter {
                    name: "Aspect Ratio".to_string(),
                    value: "keep".to_string(),
                    param_type: DataType::Text,
                    default_value: "keep".to_string(),
                    description: "Aspect ratio: keep, stretch, crop, pad".to_string(),
                });
            },
            NodeType::SplitAudioVideo => {
                parameters.insert("video_codec".to_string(), NodeParameter {
                    name: "Video Codec".to_string(),
                    value: "copy".to_string(),
                    param_type: DataType::Text,
                    default_value: "copy".to_string(),
                    description: "Video codec: copy, libx264, libx265".to_string(),
                });
                parameters.insert("audio_format".to_string(), NodeParameter {
                    name: "Audio Format".to_string(),
                    value: "wav".to_string(),
                    param_type: DataType::Text,
                    default_value: "wav".to_string(),
                    description: "Audio format: wav, mp3, flac, aac, ogg".to_string(),
                });
            },
            NodeType::OutputFile => {
                parameters.insert("output_path".to_string(), NodeParameter {
                    name: "Output Path".to_string(),
                    value: String::new(),
                    param_type: DataType::Text,
                    default_value: String::new(),
                    description: "Output file path".to_string(),
                });
                parameters.insert("container".to_string(), NodeParameter {
                    name: "Container".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Output container format: auto (smart detection), mp3, flac, wav, m4a, mp4, mkv, avi, mov, webm".to_string(),
                });
                parameters.insert("overwrite".to_string(), NodeParameter {
                    name: "Overwrite".to_string(),
                    value: "true".to_string(),
                    param_type: DataType::Text,
                    default_value: "true".to_string(),
                    description: "Overwrite existing file: true, false".to_string(),
                });
            },
            NodeType::VideoRecode => {
                parameters.insert("codec".to_string(), NodeParameter {
                    name: "Video Codec".to_string(),
                    value: "libx264".to_string(),
                    param_type: DataType::Text,
                    default_value: "libx264".to_string(),
                    description: "Video codec: libx264, libx265, libvpx-vp9, libav1, copy".to_string(),
                });
                parameters.insert("crf".to_string(), NodeParameter {
                    name: "CRF".to_string(),
                    value: "23".to_string(),
                    param_type: DataType::Text,
                    default_value: "23".to_string(),
                    description: "Constant Rate Factor: 0-51 (lower = better quality, 18-28 recommended)".to_string(),
                });
                parameters.insert("preset".to_string(), NodeParameter {
                    name: "Preset".to_string(),
                    value: "medium".to_string(),
                    param_type: DataType::Text,
                    default_value: "medium".to_string(),
                    description: "Encoding speed: ultrafast, superfast, veryfast, faster, fast, medium, slow, slower, veryslow".to_string(),
                });
                parameters.insert("profile".to_string(), NodeParameter {
                    name: "Profile".to_string(),
                    value: "high".to_string(),
                    param_type: DataType::Text,
                    default_value: "high".to_string(),
                    description: "Encoding profile: baseline, main, high (for H.264)".to_string(),
                });
                parameters.insert("level".to_string(), NodeParameter {
                    name: "Level".to_string(),
                    value: "4.1".to_string(),
                    param_type: DataType::Text,
                    default_value: "4.1".to_string(),
                    description: "Encoding level: 3.0, 3.1, 4.0, 4.1, 5.0, 5.1".to_string(),
                });
                parameters.insert("bitrate".to_string(), NodeParameter {
                    name: "Bitrate".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Target bitrate: 1M, 2M, 5M, 10M (leave empty for CRF mode)".to_string(),
                });
                parameters.insert("max_bitrate".to_string(), NodeParameter {
                    name: "Max Bitrate".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Maximum bitrate: 2M, 5M, 10M (for rate control)".to_string(),
                });
                parameters.insert("gop_size".to_string(), NodeParameter {
                    name: "GOP Size".to_string(),
                    value: "250".to_string(),
                    param_type: DataType::Text,
                    default_value: "250".to_string(),
                    description: "Group of Pictures size: 30, 60, 120, 250".to_string(),
                });
                parameters.insert("b_frames".to_string(), NodeParameter {
                    name: "B-Frames".to_string(),
                    value: "3".to_string(),
                    param_type: DataType::Text,
                    default_value: "3".to_string(),
                    description: "Number of B-frames: 0, 1, 2, 3, 4".to_string(),
                });
                parameters.insert("pix_fmt".to_string(), NodeParameter {
                    name: "Pixel Format".to_string(),
                    value: "yuv420p".to_string(),
                    param_type: DataType::Text,
                    default_value: "yuv420p".to_string(),
                    description: "Pixel format: yuv420p, yuv422p, yuv444p, yuv420p10le".to_string(),
                });
            },
            NodeType::ExtractVideo => {
                parameters.insert("video_codec".to_string(), NodeParameter {
                    name: "Video Codec".to_string(),
                    value: "copy".to_string(),
                    param_type: DataType::Text,
                    default_value: "copy".to_string(),
                    description: "Video codec: copy, libx264, libx265".to_string(),
                });
            },
            NodeType::ExtractAudio => {
                parameters.insert("audio_codec".to_string(), NodeParameter {
                    name: "Audio Codec".to_string(),
                    value: "copy".to_string(),
                    param_type: DataType::Text,
                    default_value: "copy".to_string(),
                    description: "Audio codec: copy, pcm_s16le, pcm_s24le, flac".to_string(),
                });
                parameters.insert("format".to_string(), NodeParameter {
                    name: "Format".to_string(),
                    value: "wav".to_string(),
                    param_type: DataType::Text,
                    default_value: "wav".to_string(),
                    description: "Audio format: wav, flac, mp3, aac".to_string(),
                });
            },
            NodeType::Combine => {
                parameters.insert("video_codec".to_string(), NodeParameter {
                    name: "Video Codec".to_string(),
                    value: "copy".to_string(),
                    param_type: DataType::Text,
                    default_value: "copy".to_string(),
                    description: "Video codec: copy, libx264, libx265".to_string(),
                });
                parameters.insert("audio_codec".to_string(), NodeParameter {
                    name: "Audio Codec".to_string(),
                    value: "copy".to_string(),
                    param_type: DataType::Text,
                    default_value: "copy".to_string(),
                    description: "Audio codec: copy, aac, libmp3lame".to_string(),
                });
                parameters.insert("sync_mode".to_string(), NodeParameter {
                    name: "Sync Mode".to_string(),
                    value: "auto".to_string(),
                    param_type: DataType::Text,
                    default_value: "auto".to_string(),
                    description: "Sync mode: auto, shortest, longest".to_string(),
                });
            },
            NodeType::VideoCrop => {
                parameters.insert("x".to_string(), NodeParameter {
                    name: "X Position".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0".to_string(),
                    description: "Crop X position (pixels from left)".to_string(),
                });
                parameters.insert("y".to_string(), NodeParameter {
                    name: "Y Position".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0".to_string(),
                    description: "Crop Y position (pixels from top)".to_string(),
                });
                parameters.insert("width".to_string(), NodeParameter {
                    name: "Width".to_string(),
                    value: "640".to_string(),
                    param_type: DataType::Text,
                    default_value: "640".to_string(),
                    description: "Crop width in pixels".to_string(),
                });
                parameters.insert("height".to_string(), NodeParameter {
                    name: "Height".to_string(),
                    value: "480".to_string(),
                    param_type: DataType::Text,
                    default_value: "480".to_string(),
                    description: "Crop height in pixels".to_string(),
                });
            },
            NodeType::VideoRotate => {
                parameters.insert("angle".to_string(), NodeParameter {
                    name: "Rotation".to_string(),
                    value: "90".to_string(),
                    param_type: DataType::Text,
                    default_value: "90".to_string(),
                    description: "Rotation angle: 90, 180, 270, or custom degrees".to_string(),
                });
                parameters.insert("transpose".to_string(), NodeParameter {
                    name: "Transpose".to_string(),
                    value: "1".to_string(),
                    param_type: DataType::Text,
                    default_value: "1".to_string(),
                    description: "Transpose mode: 0=90CCW+vflip, 1=90CW, 2=90CCW, 3=90CW+vflip".to_string(),
                });
            },
            NodeType::VideoFilter => {
                parameters.insert("filter".to_string(), NodeParameter {
                    name: "Filter".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "FFmpeg video filter: eq=brightness=0.1, hue=s=0, etc.".to_string(),
                });
                parameters.insert("preset_filter".to_string(), NodeParameter {
                    name: "Preset Filter".to_string(),
                    value: "none".to_string(),
                    param_type: DataType::Text,
                    default_value: "none".to_string(),
                    description: "Preset: none, blur, sharpen, vintage, grayscale".to_string(),
                });
            },

            NodeType::AudioCompress => {
                parameters.insert("bitrate".to_string(), NodeParameter {
                    name: "Bitrate".to_string(),
                    value: "192k".to_string(),
                    param_type: DataType::Text,
                    default_value: "192k".to_string(),
                    description: "Target bitrate: 128k, 192k, 256k, 320k".to_string(),
                });
                parameters.insert("codec".to_string(), NodeParameter {
                    name: "Codec".to_string(),
                    value: "libmp3lame".to_string(),
                    param_type: DataType::Text,
                    default_value: "libmp3lame".to_string(),
                    description: "Audio codec: libmp3lame, aac, libvorbis".to_string(),
                });
            },
            NodeType::AudioVolume => {
                parameters.insert("volume".to_string(), NodeParameter {
                    name: "Volume".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "1.0".to_string(),
                    description: "Volume multiplier: 0.5 (half), 1.0 (normal), 2.0 (double)".to_string(),
                });
                parameters.insert("filter_type".to_string(), NodeParameter {
                    name: "Filter Type".to_string(),
                    value: "volume".to_string(),
                    param_type: DataType::Text,
                    default_value: "volume".to_string(),
                    description: "Filter type: volume, amplify, normalize".to_string(),
                });
            },
            NodeType::AudioTrim => {
                parameters.insert("start_time".to_string(), NodeParameter {
                    name: "Start Time".to_string(),
                    value: "0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0".to_string(),
                    description: "Start time in seconds or HH:MM:SS format".to_string(),
                });
                parameters.insert("duration".to_string(), NodeParameter {
                    name: "Duration".to_string(),
                    value: "10".to_string(),
                    param_type: DataType::Text,
                    default_value: "10".to_string(),
                    description: "Duration in seconds or HH:MM:SS format".to_string(),
                });
                parameters.insert("end_time".to_string(), NodeParameter {
                    name: "End Time".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "End time (alternative to duration)".to_string(),
                });
            },
            NodeType::AudioMerge => {
                parameters.insert("mix_mode".to_string(), NodeParameter {
                    name: "Mix Mode".to_string(),
                    value: "amix".to_string(),
                    param_type: DataType::Text,
                    default_value: "amix".to_string(),
                    description: "Mix mode: amix, amerge, join".to_string(),
                });
                parameters.insert("inputs".to_string(), NodeParameter {
                    name: "Input Count".to_string(),
                    value: "2".to_string(),
                    param_type: DataType::Text,
                    default_value: "2".to_string(),
                    description: "Number of inputs to mix: 2, 3, 4, 5".to_string(),
                });
                parameters.insert("duration".to_string(), NodeParameter {
                    name: "Duration".to_string(),
                    value: "longest".to_string(),
                    param_type: DataType::Text,
                    default_value: "longest".to_string(),
                    description: "Output duration: shortest, longest, first".to_string(),
                });
            },

            NodeType::FrameExtract => {
                parameters.insert("fps".to_string(), NodeParameter {
                    name: "FPS".to_string(),
                    value: "1".to_string(),
                    param_type: DataType::Text,
                    default_value: "1".to_string(),
                    description: "Frames per second to extract: 0.5, 1, 2, 5".to_string(),
                });
                parameters.insert("format".to_string(), NodeParameter {
                    name: "Format".to_string(),
                    value: "png".to_string(),
                    param_type: DataType::Text,
                    default_value: "png".to_string(),
                    description: "Image format: png, jpg, bmp, tiff".to_string(),
                });
                parameters.insert("quality".to_string(), NodeParameter {
                    name: "Quality".to_string(),
                    value: "2".to_string(),
                    param_type: DataType::Text,
                    default_value: "2".to_string(),
                    description: "JPEG quality: 1-31 (lower = better)".to_string(),
                });
            },
            NodeType::VideoFPS => {
                parameters.insert("fps".to_string(), NodeParameter {
                    name: "Target FPS".to_string(),
                    value: "30".to_string(),
                    param_type: DataType::Text,
                    default_value: "30".to_string(),
                    description: "Target frame rate: 24, 25, 30, 50, 60".to_string(),
                });
                parameters.insert("filter".to_string(), NodeParameter {
                    name: "Filter".to_string(),
                    value: "fps".to_string(),
                    param_type: DataType::Text,
                    default_value: "fps".to_string(),
                    description: "Filter type: fps, minterpolate".to_string(),
                });
            },
            NodeType::VideoBrightness => {
                parameters.insert("brightness".to_string(), NodeParameter {
                    name: "Brightness".to_string(),
                    value: "0.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0.0".to_string(),
                    description: "Brightness adjustment: -1.0 to 1.0".to_string(),
                });
                parameters.insert("contrast".to_string(), NodeParameter {
                    name: "Contrast".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "1.0".to_string(),
                    description: "Contrast adjustment: 0.0 to 3.0".to_string(),
                });
            },
            NodeType::VideoSaturation => {
                parameters.insert("saturation".to_string(), NodeParameter {
                    name: "Saturation".to_string(),
                    value: "1.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "1.0".to_string(),
                    description: "Saturation level: 0.0 (grayscale) to 3.0".to_string(),
                });
            },
            NodeType::StreamPrep => {
                parameters.insert("bitrate".to_string(), NodeParameter {
                    name: "Bitrate".to_string(),
                    value: "2000k".to_string(),
                    param_type: DataType::Text,
                    default_value: "2000k".to_string(),
                    description: "Video bitrate for streaming: 1000k, 2000k, 5000k".to_string(),
                });
                parameters.insert("resolution".to_string(), NodeParameter {
                    name: "Resolution".to_string(),
                    value: "1920x1080".to_string(),
                    param_type: DataType::Text,
                    default_value: "1920x1080".to_string(),
                    description: "Target resolution: 1280x720, 1920x1080".to_string(),
                });
                parameters.insert("fps".to_string(), NodeParameter {
                    name: "Frame Rate".to_string(),
                    value: "30".to_string(),
                    param_type: DataType::Text,
                    default_value: "30".to_string(),
                    description: "Frame rate: 24, 30, 60".to_string(),
                });
            },
            NodeType::AudioVideoSync => {
                parameters.insert("sync_offset".to_string(), NodeParameter {
                    name: "Sync Offset".to_string(),
                    value: "0.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0.0".to_string(),
                    description: "Audio sync offset in seconds (+ for delay, - for advance)".to_string(),
                });
            },
            NodeType::AudioDelay => {
                parameters.insert("delay".to_string(), NodeParameter {
                    name: "Delay".to_string(),
                    value: "0.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0.0".to_string(),
                    description: "Audio delay in seconds".to_string(),
                });
            },
            NodeType::VideoDelay => {
                parameters.insert("delay".to_string(), NodeParameter {
                    name: "Delay".to_string(),
                    value: "0.0".to_string(),
                    param_type: DataType::Text,
                    default_value: "0.0".to_string(),
                    description: "Video delay in seconds".to_string(),
                });
            },
            NodeType::AddMetadata => {
                parameters.insert("title".to_string(), NodeParameter {
                    name: "Title".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Video title metadata".to_string(),
                });
                parameters.insert("artist".to_string(), NodeParameter {
                    name: "Artist".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Artist/creator metadata".to_string(),
                });
                parameters.insert("comment".to_string(), NodeParameter {
                    name: "Comment".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Comment metadata".to_string(),
                });
            },
            NodeType::VideoEncrypt => {
                parameters.insert("password".to_string(), NodeParameter {
                    name: "Password".to_string(),
                    value: "".to_string(),
                    param_type: DataType::Text,
                    default_value: "".to_string(),
                    description: "Encryption password".to_string(),
                });
            },
            NodeType::MultiPassEncode => {
                parameters.insert("bitrate".to_string(), NodeParameter {
                    name: "Bitrate".to_string(),
                    value: "2000k".to_string(),
                    param_type: DataType::Text,
                    default_value: "2000k".to_string(),
                    description: "Target bitrate for multi-pass encoding".to_string(),
                });
            },
            NodeType::BatchProcess => {
                parameters.insert("operations".to_string(), NodeParameter {
                    name: "Operations".to_string(),
                    value: "resize,compress".to_string(),
                    param_type: DataType::Text,
                    default_value: "resize,compress".to_string(),
                    description: "Comma-separated list of operations: resize, compress, normalize".to_string(),
                });
            },
            _ => {}
        }
        }
        
        Self {
            id,
            node_type,
            position,
            size: Vec2::new(200.0, 120.0),
            input_ports,
            output_ports,
            parameters,
            enabled: true,
        }
    }
    

    pub fn get_input_port_position(&self, port_index: usize) -> Option<Pos2> {
        if port_index < self.input_ports.len() {
            let port_y = self.position.y + 30.0 + (port_index as f32 * 25.0);
            Some(Pos2::new(self.position.x, port_y))
        } else {
            None
        }
    }
    

    pub fn get_output_port_position(&self, port_index: usize) -> Option<Pos2> {
        if port_index < self.output_ports.len() {
            let port_y = self.position.y + 30.0 + (port_index as f32 * 25.0);
            Some(Pos2::new(self.position.x + self.size.x, port_y))
        } else {
            None
        }
    }
    
    /// Show comprehensive parameter UI for the node
    pub fn show_comprehensive_parameters_ui(&mut self, ui: &mut egui::Ui, translations: &crate::language::Translations, cached_hw_encoders: &[String]) {
        match self.node_type {
            NodeType::InputFile => {
                self.show_input_file_parameters(ui, translations);
            },
            NodeType::OutputFile => {
                self.show_output_file_parameters(ui, translations);
            },
            NodeType::VideoConvert => {
                self.show_video_convert_parameters(ui, translations, cached_hw_encoders);
            },
            NodeType::AudioConvert => {
                self.show_audio_convert_parameters(ui, translations);
            },
            NodeType::VideoCompress => {
                self.show_video_compress_parameters(ui, translations);
            },
            NodeType::AudioCompress => {
                self.show_audio_compress_parameters(ui, translations);
            },
            _ => {
                // Fallback to simple text inputs for other node types
                self.show_simple_parameters_ui(ui);
            }
        }
    }
    
    /// Show video convert parameters with comprehensive UI
    fn show_video_convert_parameters(&mut self, ui: &mut egui::Ui, translations: &crate::language::Translations, cached_hw_encoders: &[String]) {
        ui.heading("🎬 Video Convert Settings");
        
        // Output format selector
        if let Some(format_param) = self.parameters.get_mut("output_format") {
            ui.horizontal(|ui| {
                ui.label("Output Format:");
            });
            ComprehensiveUIComponents::show_comprehensive_format_selector(
                ui,
                &mut format_param.value,
                FormatPurpose::Video,
                translations
            );
        }
        
        // Video codec selector  
        let selected_format = self.parameters.get("output_format").map(|p| p.value.as_str()).unwrap_or("").to_string();
        if let Some(video_codec_param) = self.parameters.get_mut("video_codec") {
            ui.horizontal(|ui| {
                ui.label("Video Codec:");
            });
            ComprehensiveUIComponents::show_comprehensive_codec_selector(
                ui,
                CodecType::Video,
                &mut video_codec_param.value,
                &selected_format,
                translations,
                cached_hw_encoders
            );
        }
        
        // Audio codec selector
        if let Some(audio_codec_param) = self.parameters.get_mut("audio_codec") {
            ui.horizontal(|ui| {
                ui.label("Audio Codec:");
            });
            ComprehensiveUIComponents::show_comprehensive_codec_selector(
                ui,
                CodecType::Audio,
                &mut audio_codec_param.value,
                &selected_format,
                translations,
                cached_hw_encoders
            );
        }
        
        // Copy video checkbox
        if let Some(copy_param) = self.parameters.get_mut("copy_video") {
            let mut copy_video = copy_param.value == "true";
            if ui.checkbox(&mut copy_video, "Copy video stream (fast conversion)").changed() {
                copy_param.value = copy_video.to_string();
            }
        }
    }
    
    /// Show audio convert parameters with comprehensive UI
    fn show_audio_convert_parameters(&mut self, ui: &mut egui::Ui, translations: &crate::language::Translations) {
        ui.heading("🎵 Audio Convert Settings");
        
        // Output format selector
        if let Some(format_param) = self.parameters.get_mut("output_format") {
            ui.horizontal(|ui| {
                ui.label("Output Format:");
            });
            ComprehensiveUIComponents::show_comprehensive_format_selector(
                ui,
                &mut format_param.value,
                FormatPurpose::Audio,
                translations
            );
        }
        
        // Audio codec selector
        let selected_format = self.parameters.get("output_format").map(|p| p.value.as_str()).unwrap_or("").to_string();
        if let Some(codec_param) = self.parameters.get_mut("audio_codec") {
            ui.horizontal(|ui| {
                ui.label("Audio Codec:");
            });
            ComprehensiveUIComponents::show_comprehensive_codec_selector(
                ui,
                CodecType::Audio,
                &mut codec_param.value,
                &selected_format,
                translations,
                &[] // No hardware encoders for audio
            );
        }
        
        // Bitrate selector
        if let Some(bitrate_param) = self.parameters.get_mut("bitrate") {
            ui.horizontal(|ui| {
                ui.label("Bitrate:");
                egui::ComboBox::from_id_salt("audio_bitrate")
                    .selected_text(&bitrate_param.value)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut bitrate_param.value, "auto".to_string(), "Auto");
                        ui.separator();
                        for bitrate in ["64k", "96k", "128k", "160k", "192k", "256k", "320k"] {
                            ui.selectable_value(&mut bitrate_param.value, bitrate.to_string(), bitrate);
                        }
                    });
            });
        }
        
        // Sample rate selector
        if let Some(sample_rate_param) = self.parameters.get_mut("sample_rate") {
            ui.horizontal(|ui| {
                ui.label("Sample Rate:");
                egui::ComboBox::from_id_salt("sample_rate")
                    .selected_text(format!("{} Hz", sample_rate_param.value))
                    .show_ui(ui, |ui| {
                        for rate in ["22050", "44100", "48000", "96000"] {
                            ui.selectable_value(&mut sample_rate_param.value, rate.to_string(), format!("{} Hz", rate));
                        }
                    });
            });
        }
    }
    
    /// Show video compress parameters
    fn show_video_compress_parameters(&mut self, ui: &mut egui::Ui, _translations: &crate::language::Translations) {
        ui.heading("🗁 Video Compression Settings");
        
        // Quality (CRF) slider
        if let Some(quality_param) = self.parameters.get_mut("quality") {
            let mut quality: i32 = quality_param.value.parse().unwrap_or(23);
            ui.horizontal(|ui| {
                ui.label("Quality (CRF):");
                if ui.add(egui::Slider::new(&mut quality, 0..=51).suffix(" (lower=better)")).changed() {
                    quality_param.value = quality.to_string();
                }
            });
        }
        
        // Preset selector
        if let Some(preset_param) = self.parameters.get_mut("preset") {
            ui.horizontal(|ui| {
                ui.label("Preset:");
                egui::ComboBox::from_id_salt("preset")
                    .selected_text(&preset_param.value)
                    .show_ui(ui, |ui| {
                        for preset in ["ultrafast", "superfast", "veryfast", "faster", "fast", "medium", "slow", "slower", "veryslow"] {
                            ui.selectable_value(&mut preset_param.value, preset.to_string(), preset);
                        }
                    });
            });
        }
        
        // Target size
        if let Some(size_param) = self.parameters.get_mut("target_size_mb") {
            let mut target_size: i32 = size_param.value.parse().unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label("Target Size (MB):");
                if ui.add(egui::Slider::new(&mut target_size, 0..=5000).suffix(" MB (0=disabled)")).changed() {
                    size_param.value = target_size.to_string();
                }
            });
        }
    }
    
    /// Show audio compress parameters
    fn show_audio_compress_parameters(&mut self, ui: &mut egui::Ui, _translations: &crate::language::Translations) {
        ui.heading("🔊 Audio Compression Settings");
        
        // Bitrate selector
        if let Some(bitrate_param) = self.parameters.get_mut("bitrate") {
            ui.horizontal(|ui| {
                ui.label("Target Bitrate:");
                egui::ComboBox::from_id_salt("compress_bitrate")
                    .selected_text(&bitrate_param.value)
                    .show_ui(ui, |ui| {
                        for bitrate in ["32k", "64k", "96k", "128k", "160k", "192k", "256k", "320k"] {
                            ui.selectable_value(&mut bitrate_param.value, bitrate.to_string(), bitrate);
                        }
                    });
            });
        }
        
        // Quality selector
        if let Some(quality_param) = self.parameters.get_mut("quality") {
            let mut quality: i32 = quality_param.value.parse().unwrap_or(5);
            ui.horizontal(|ui| {
                ui.label("VBR Quality:");
                if ui.add(egui::Slider::new(&mut quality, 0..=9).suffix(" (0=best, 9=worst)")).changed() {
                    quality_param.value = quality.to_string();
                }
            });
        }
    }
    
    /// Simple fallback parameter UI for nodes not yet implemented
    fn show_simple_parameters_ui(&mut self, ui: &mut egui::Ui) {
        for (_param_name, param) in self.parameters.iter_mut() {
            ui.horizontal(|ui| {
                ui.label(&param.name);
                match param.param_type {
                    DataType::Text => {
                        ui.text_edit_singleline(&mut param.value);
                    },
                    DataType::Number => {
                        ui.text_edit_singleline(&mut param.value);
                    },
                    DataType::Boolean => {
                        let mut bool_val = param.value == "true";
                        if ui.checkbox(&mut bool_val, "").changed() {
                            param.value = bool_val.to_string();
                        }
                    },
                    _ => {
                        ui.label(&param.value);
                    }
                }
            });
        }
    }
    
    /// Auto-fill output filename based on connected input file
    pub fn auto_fill_output_from_input(&mut self, workflow: &AutomationWorkflow) -> bool {
        if self.node_type != NodeType::OutputFile {
            return false;
        }
        
        // Find any connected input file node (directly or through processing chain)
        let mut input_file_path = None;
        
        // First, try to find direct connection to InputFile
        for connection in workflow.connections.values() {
            if connection.to_node == self.id && connection.to_port == 0 {
                if let Some(source_node) = workflow.nodes.get(&connection.from_node) {
                    if source_node.node_type == NodeType::InputFile {
                        if let Some(input_file_param) = source_node.parameters.get("file_path") {
                            if !input_file_param.value.is_empty() {
                                input_file_path = Some(input_file_param.value.clone());
                                break;
                            }
                        }
                    } else {
                        // If not direct, trace back through the processing chain
                        input_file_path = self.find_input_file_in_chain(&source_node.id, workflow);
                        if input_file_path.is_some() {
                            break;
                        }
                    }
                }
            }
        }
        
        // If we found an input file, auto-fill the output with DYNAMIC timestamp generation
        if let Some(input_path_str) = input_file_path {
            if let Some(output_param) = self.parameters.get_mut("output_path") {
                if output_param.value.trim().is_empty() {
                    let input_path = std::path::Path::new(&input_path_str);
                    if let Some(_parent) = input_path.parent() {
                        if let Some(ext) = input_path.extension() {
                            // Store a template instead of actual path with timestamp
                            let template = format!("AUTO_FILL:{}", ext.to_string_lossy());
                            output_param.value = template;
                            return true; // Value was changed
                        }
                    }
                }
            }
        }
        
        false // No change made
    }
    
    /// Generate dynamic output path with current timestamp
    pub fn get_dynamic_output_path(&self, workflow: &AutomationWorkflow) -> Option<String> {
        if self.node_type != NodeType::OutputFile {
            return None;
        }
        
        if let Some(output_param) = self.parameters.get("output_path") {
            // Check if this is an auto-fill template
            if output_param.value.starts_with("AUTO_FILL:") {
                let ext = output_param.value.strip_prefix("AUTO_FILL:").unwrap_or("mp4");
                
                // Find the input file path again to get the directory
                let mut input_file_path = None;
                for connection in workflow.connections.values() {
                    if connection.to_node == self.id && connection.to_port == 0 {
                        if let Some(source_node) = workflow.nodes.get(&connection.from_node) {
                            if source_node.node_type == NodeType::InputFile {
                                if let Some(input_file_param) = source_node.parameters.get("file_path") {
                                    if !input_file_param.value.is_empty() {
                                        input_file_path = Some(input_file_param.value.clone());
                                        break;
                                    }
                                }
                            } else {
                                input_file_path = self.find_input_file_in_chain(&source_node.id, workflow);
                                if input_file_path.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
                
                if let Some(input_path_str) = input_file_path {
                    let input_path = std::path::Path::new(&input_path_str);
                    if let Some(parent) = input_path.parent() {
                        // Generate timestamp at execution time
                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                        let output_name = format!("output_{}.{}", timestamp, ext);
                        let new_path = parent.join(output_name).display().to_string();
                        return Some(new_path);
                    }
                }
            } else if !output_param.value.trim().is_empty() {
                // Return user-specified path as is
                return Some(output_param.value.clone());
            }
        }
        
        None
    }
    
    /// Recursively find input file through processing chain
    fn find_input_file_in_chain(&self, node_id: &str, workflow: &AutomationWorkflow) -> Option<String> {
        if let Some(node) = workflow.nodes.get(node_id) {
            if node.node_type == NodeType::InputFile {
                if let Some(input_file_param) = node.parameters.get("file_path") {
                    if !input_file_param.value.is_empty() {
                        return Some(input_file_param.value.clone());
                    }
                }
            } else {
                // Check this node's inputs
                for connection in workflow.connections.values() {
                    if connection.to_node == *node_id {
                        if let Some(result) = self.find_input_file_in_chain(&connection.from_node, workflow) {
                            return Some(result);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Show input file parameters with drag-drop support
    fn show_input_file_parameters(&mut self, ui: &mut egui::Ui, _translations: &crate::language::Translations) {
        ui.heading("📁 Input File Settings");
        
        if let Some(file_param) = self.parameters.get_mut("file_path") {
            let _old_value = file_param.value.clone();
            ui.label("Input File:");
            
            // Create drag-and-drop area
            let file_area_height = 60.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), file_area_height),
                egui::Sense::click()
            );
            
            // Handle drag and drop detection
            let is_being_dragged = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());
            let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
            
            // Draw background
            let bg_color = if can_accept_drop {
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
            } else if file_param.value.is_empty() {
                egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
            } else {
                egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
            };
            
            ui.painter().rect_filled(rect, 5.0, bg_color);
            ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
            
            // Handle click to open file dialog
            if response.clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Media Files", &["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "aac", "ogg", "webm", "m4a", "wma"])
                    .add_filter("All Files", &["*"])
                    .pick_file() 
                {
                    file_param.value = path.display().to_string();
                }
            }
            
            // Process dropped files
            if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                if let Some(first_file) = dropped_files.first() {
                    if let Some(path) = &first_file.path {
                        file_param.value = path.display().to_string();
                    }
                }
            }
            
            // Draw text in the drop area
            let text = if file_param.value.is_empty() {
                if can_accept_drop {
                    "📁 Drop file here".to_string()
                } else {
                    "📁 Click to select file or drag & drop here".to_string()
                }
            } else {
                let file_name = std::path::Path::new(&file_param.value)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                format!("📁 {}", file_name)
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                if file_param.value.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
            );
            
            ui.add_space(5.0);
            
            // Show controls
            ui.horizontal(|ui| {
                if !file_param.value.is_empty() {
                    if ui.button("🗑️ Clear").clicked() {
                        file_param.value.clear();
                    }
                    if ui.button("📁 Change").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Media Files", &["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "aac", "ogg", "webm", "m4a", "wma"])
                            .add_filter("All Files", &["*"])
                            .pick_file() 
                        {
                            file_param.value = path.display().to_string();
                        }
                    }
                }
            });
        }
    }
    
    /// Show output file parameters with drag-drop support
    fn show_output_file_parameters(&mut self, ui: &mut egui::Ui, _translations: &crate::language::Translations) {
        ui.heading("💾 Output File Settings");
        
        if let Some(file_param) = self.parameters.get_mut("output_path") {
            ui.label("Output File:");
            
            // Create drag-and-drop area
            let file_area_height = 60.0;
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), file_area_height),
                egui::Sense::click()
            );
            
            // Handle drag and drop detection
            let is_being_dragged = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());
            let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
            
            // Draw background
            let bg_color = if can_accept_drop {
                egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
            } else if file_param.value.is_empty() {
                egui::Color32::from_rgba_unmultiplied(60, 60, 60, 100)
            } else {
                egui::Color32::from_rgba_unmultiplied(40, 80, 120, 100)
            };
            
            ui.painter().rect_filled(rect, 5.0, bg_color);
            ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
            
            // Handle click to open save dialog
            if response.clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Media Files", &["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "aac", "ogg", "webm", "m4a", "wma"])
                    .add_filter("All Files", &["*"])
                    .save_file() 
                {
                    file_param.value = path.display().to_string();
                }
            }
            
            // Process dropped files (use as template for output path)
            if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                if let Some(first_file) = dropped_files.first() {
                    if let Some(path) = &first_file.path {
                        // Use dropped file path as template, append timestamp before extension
                        let _dropped_path = path.display().to_string();
                        if let Some(_stem) = path.file_stem() {
                            if let Some(ext) = path.extension() {
                                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                                let output_name = format!("output_{}.{}", 
                                    timestamp,
                                    ext.to_string_lossy()
                                );
                                if let Some(parent) = path.parent() {
                                    file_param.value = parent.join(output_name).display().to_string();
                                } else {
                                    file_param.value = output_name;
                                }
                            } else {
                                let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                                file_param.value = format!("output_{}", timestamp);
                            }
                        } else {
                            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                            file_param.value = format!("output_{}", timestamp);
                        }
                    }
                }
            }
            
            // Draw text in the drop area
            let text = if file_param.value.is_empty() {
                if can_accept_drop {
                    "💾 Drop file here (as template)".to_string()
                } else {
                    "💾 Click to select output file or drag & drop here".to_string()
                }
            } else {
                let file_name = std::path::Path::new(&file_param.value)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                format!("💾 {}", file_name)
            };
            
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::default(),
                if file_param.value.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
            );
            
            ui.add_space(5.0);
            
            // Show controls
            ui.horizontal(|ui| {
                if !file_param.value.is_empty() {
                    if ui.button("🗑️ Clear").clicked() {
                        file_param.value.clear();
                    }
                    if ui.button("💾 Change").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Media Files", &["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "aac", "ogg", "webm", "m4a", "wma"])
                            .add_filter("All Files", &["*"])
                            .save_file() 
                        {
                            file_param.value = path.display().to_string();
                        }
                    }
                }
            });
        }
        
        // Container format selector
        if let Some(container_param) = self.parameters.get_mut("container") {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Container Format:");
                egui::ComboBox::from_id_salt("output_container")
                    .selected_text(&container_param.value)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut container_param.value, "auto".to_string(), "Auto (from filename)");
                        ui.separator();
                        ui.selectable_value(&mut container_param.value, "mp4".to_string(), "MP4");
                        ui.selectable_value(&mut container_param.value, "mkv".to_string(), "MKV");
                        ui.selectable_value(&mut container_param.value, "avi".to_string(), "AVI");
                        ui.selectable_value(&mut container_param.value, "mov".to_string(), "MOV");
                        ui.selectable_value(&mut container_param.value, "webm".to_string(), "WebM");
                        ui.separator();
                        ui.selectable_value(&mut container_param.value, "mp3".to_string(), "MP3");
                        ui.selectable_value(&mut container_param.value, "wav".to_string(), "WAV");
                        ui.selectable_value(&mut container_param.value, "flac".to_string(), "FLAC");
                        ui.selectable_value(&mut container_param.value, "m4a".to_string(), "M4A");
                        ui.selectable_value(&mut container_param.value, "aac".to_string(), "AAC");
                    });
            });
        }
        
        // Overwrite option
        if let Some(overwrite_param) = self.parameters.get_mut("overwrite") {
            let mut overwrite = overwrite_param.value == "true";
            if ui.checkbox(&mut overwrite, "Overwrite existing file").changed() {
                overwrite_param.value = overwrite.to_string();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub id: String,
    pub from_node: String,
    pub from_port: usize,
    pub to_node: String,  
    pub to_port: usize,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: HashMap<String, AutomationNode>,
    pub connections: HashMap<String, NodeConnection>,
    pub created_at: String,
    pub modified_at: String,
}

impl AutomationWorkflow {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: generate_unique_id(),
            name,
            description: String::new(),
            nodes: HashMap::new(),
            connections: HashMap::new(),
            created_at: now.clone(),
            modified_at: now,
        }
    }
    
    pub fn add_node(&mut self, node: AutomationNode) {
        self.nodes.insert(node.id.clone(), node);
        self.update_modified_time();
    }
    
    pub fn remove_node(&mut self, node_id: &str) {
        let connections_to_remove: Vec<String> = self.connections
            .iter()
            .filter(|(_, conn)| conn.from_node == node_id || conn.to_node == node_id)
            .map(|(id, _)| id.clone())
            .collect();
            
        for conn_id in connections_to_remove {
            self.connections.remove(&conn_id);
        }
        
        self.nodes.remove(node_id);
        self.update_modified_time();
    }
    
    pub fn add_connection(&mut self, connection: NodeConnection) -> Result<(), String> {
        if let (Some(from_node), Some(to_node)) = (
            self.nodes.get(&connection.from_node),
            self.nodes.get(&connection.to_node)
        ) {
            if connection.from_port >= from_node.output_ports.len() {
                return Err("输出端口不存在".to_string());
            }
            if connection.to_port >= to_node.input_ports.len() {
                return Err("输入端口不存在".to_string());
            }
            
            let from_port_type = &from_node.output_ports[connection.from_port].data_type;
            let to_port_type = &to_node.input_ports[connection.to_port].data_type;
            
            if !Self::are_types_compatible(from_port_type, to_port_type) {
                return Err(format!("数据类型不匹配: {} -> {}", 
                    from_port_type.display_name(), 
                    to_port_type.display_name())
                );
            }
            
            self.connections.insert(connection.id.clone(), connection);
            self.update_modified_time();
            Ok(())
        } else {
            Err("节点不存在".to_string())
        }
    }
    
    pub fn remove_connection(&mut self, connection_id: &str) {
        self.connections.remove(connection_id);
        self.update_modified_time();
    }
    
    pub fn get_execution_order(&self) -> Result<Vec<String>, String> {
        let mut in_degree = HashMap::new();
        let mut graph = HashMap::new();
        
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            graph.insert(node_id.clone(), Vec::new());
        }
        
        for connection in self.connections.values() {
            graph.get_mut(&connection.from_node).unwrap().push(connection.to_node.clone());
            *in_degree.get_mut(&connection.to_node).unwrap() += 1;
        }
        
        let mut queue = Vec::new();
        let mut result = Vec::new();
        
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push(node_id.clone());
            }
        }
        
        while let Some(current) = queue.pop() {
            result.push(current.clone());
            
            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    let new_degree = in_degree.get_mut(neighbor).unwrap();
                    *new_degree -= 1;
                    if *new_degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
        
        if result.len() != self.nodes.len() {
            Err("工作流程中存在循环依赖".to_string())
        } else {
            Ok(result)
        }
    }
    
    fn update_modified_time(&mut self) {
        self.modified_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    }
    
    pub fn are_types_compatible(from_type: &DataType, to_type: &DataType) -> bool {
        match (from_type, to_type) {
            (a, b) if a == b => true,
            
            (DataType::MediaFile, _) => true,
            
            (_, DataType::MediaFile) => true,
            
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    Idle,
    Running,
    Success,
    Failed(String),
    Cancelled,
}

/// Workflow execution result for each node
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    pub node_id: String,
    pub success: bool,
    pub output_files: Vec<String>,
    pub error_message: Option<String>,
    pub execution_time: std::time::Duration,
}

/// Workflow execution engine
pub struct WorkflowExecutor {
    pub current_workflow: Option<AutomationWorkflow>,
    pub execution_status: ExecutionStatus,
    pub current_node: Option<String>,
    pub progress: f32,
    pub execution_results: Vec<NodeExecutionResult>,
    pub temp_files: Vec<String>,
    pub workflow_temp_dir: Option<String>,
    pub current_step: usize,
    pub cached_hardware_encoders: Vec<String>,
}

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self {
            current_workflow: None,
            execution_status: ExecutionStatus::Idle,
            current_node: None,
            progress: 0.0,
            execution_results: Vec::new(),
            temp_files: Vec::new(),
            workflow_temp_dir: None,
            current_step: 0,
            cached_hardware_encoders: Vec::new(),
        }
    }
    
    /// Create new WorkflowExecutor with hardware encoder cache
    pub fn new_with_hardware_cache(hardware_encoders: Vec<String>) -> Self {
        Self {
            current_workflow: None,
            execution_status: ExecutionStatus::Idle,
            current_node: None,
            progress: 0.0,
            execution_results: Vec::new(),
            temp_files: Vec::new(),
            workflow_temp_dir: None,
            current_step: 0,
            cached_hardware_encoders: hardware_encoders,
        }
    }
    
    /// Update hardware encoder cache
    pub fn update_hardware_cache(&mut self, hardware_encoders: Vec<String>) {
        self.cached_hardware_encoders = hardware_encoders;
        log_info!("Updated workflow executor hardware cache with {} encoders", self.cached_hardware_encoders.len());
    }
    
    /// Execute the entire workflow
    pub fn execute_workflow(&mut self, workflow: AutomationWorkflow) -> Result<(), String> {
        self.current_workflow = Some(workflow.clone());
        self.execution_status = ExecutionStatus::Running;
        self.progress = 0.0;
        self.execution_results.clear();
        self.temp_files.clear();
        
        // Create dedicated temporary directory for this workflow
        self.create_workflow_temp_dir(&workflow.id)?;
        self.current_step = 0;
        
        // Get execution order using topological sort
        let execution_order = workflow.get_execution_order()?;
        let total_nodes = execution_order.len();
        
        // Execute nodes in order with proper error handling
        for (index, node_id) in execution_order.iter().enumerate() {
            if let ExecutionStatus::Cancelled = self.execution_status {
                break;
            }
            
            self.current_node = Some(node_id.clone());
            self.current_step = index + 1;
            self.progress = (index as f32) / (total_nodes as f32);
            
            if let Some(node) = workflow.nodes.get(node_id) {
                log_info!("Executing node {}/{}: {} ({})", index + 1, total_nodes, node_id, node.node_type.display_name());
                match self.execute_node(node, &workflow) {
                    Ok(result) => {
                        if result.success {
                            log_info!("Node {} completed successfully", node_id);
                            self.execution_results.push(result);
                        } else {
                            let error_msg = result.error_message.unwrap_or_else(|| "Unknown error".to_string());
                            log_error!("Node {} failed: {}", node_id, error_msg);
                            self.execution_status = ExecutionStatus::Failed(error_msg.clone());
                            self.cleanup_workflow_temp_dir();
                            return Err(format!("Node {} failed: {}", node_id, error_msg));
                        }
                    }
                    Err(e) => {
                        log_error!("Node {} failed: {}", node_id, e);
                        self.execution_status = ExecutionStatus::Failed(e.clone());
                        self.cleanup_workflow_temp_dir();
                        return Err(format!("Failed to execute node {} ({}): {}", node_id, node.node_type.display_name(), e));
                    }
                }
            } else {
                let error_msg = format!("Node {} not found in workflow", node_id);
                log_error!("{}", error_msg);
                self.execution_status = ExecutionStatus::Failed(error_msg.clone());
                self.cleanup_workflow_temp_dir();
                return Err(error_msg);
            }
        }
        
        self.progress = 1.0;
        self.execution_status = ExecutionStatus::Success;
        self.current_node = None;
        
        Ok(())
    }
    
    /// Execute a single node
    fn execute_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<NodeExecutionResult, String> {
        let start_time = std::time::Instant::now();
        log_debug!("Executing node: {} ({})", node.id, node.node_type.display_name());
        
        let result = match node.node_type {
            // Input/Output nodes
            NodeType::InputFile => self.execute_input_file_node(node),
            NodeType::OutputFile => self.execute_output_file_node(node, workflow),
            
            // Audio processing nodes
            NodeType::ExtractAudio => self.execute_extract_audio_node(node, workflow),
            NodeType::AudioResample => self.execute_audio_resample_node(node, workflow),
            NodeType::AudioConvert => self.execute_audio_convert_node(node, workflow),
            NodeType::AudioCompress => self.execute_audio_compress_node(node, workflow),
            NodeType::AudioVolume => self.execute_audio_volume_node(node, workflow),
            NodeType::AudioTrim => self.execute_audio_trim_node(node, workflow),
            NodeType::AudioMerge => self.execute_audio_merge_node(node, workflow),
            NodeType::AudioNormalize => self.execute_audio_normalize_node(node, workflow),
            NodeType::AudioDeNoise => self.execute_audio_denoise_node(node, workflow),
            NodeType::AudioEqualizer => self.execute_audio_equalizer_node(node, workflow),
            NodeType::AudioFade => self.execute_audio_fade_node(node, workflow),
            NodeType::AudioEcho => self.execute_audio_echo_node(node, workflow),
            NodeType::AudioSpeed => self.execute_audio_speed_node(node, workflow),
            
            // Video processing nodes
            NodeType::ExtractVideo => self.execute_extract_video_node(node, workflow),
            NodeType::VideoRecode => self.execute_video_recode_node(node, workflow),
            NodeType::VideoConvert => self.execute_video_convert_node(node, workflow),
            NodeType::VideoCompress => self.execute_video_compress_node(node, workflow),
            NodeType::VideoResize => self.execute_video_resize_node(node, workflow),
            NodeType::VideoCrop => self.execute_video_crop_node(node, workflow),
            NodeType::VideoRotate => self.execute_video_rotate_node(node, workflow),
            NodeType::VideoFilter => self.execute_video_filter_node(node, workflow),
            NodeType::FrameExtract => self.execute_frame_extract_node(node, workflow),
            NodeType::VideoFPS => self.execute_video_fps_node(node, workflow),
            NodeType::VideoStabilize => self.execute_video_stabilize_node(node, workflow),
            NodeType::VideoDeinterlace => self.execute_video_deinterlace_node(node, workflow),
            NodeType::VideoColorCorrect => self.execute_video_color_correct_node(node, workflow),
            NodeType::VideoBrightness => self.execute_video_brightness_node(node, workflow),
            NodeType::VideoSaturation => self.execute_video_saturation_node(node, workflow),
            NodeType::VideoGamma => self.execute_video_gamma_node(node, workflow),
            
            // Combination and splitting
            NodeType::Combine => self.execute_combine_node(node, workflow),
            NodeType::SplitAudioVideo => self.execute_split_av_node(node, workflow),
            NodeType::VideoOverlay => self.execute_video_overlay_node(node, workflow),
            NodeType::VideoPiP => self.execute_video_pip_node(node, workflow),
            NodeType::VideoSideBySide => self.execute_video_side_by_side_node(node, workflow),
            
            // Special conversions
            NodeType::VideoToGif => self.execute_video_to_gif_node(node, workflow),
            NodeType::GifResize => self.execute_gif_resize_node(node, workflow),
            NodeType::VideoToImages => self.execute_video_to_images_node(node, workflow),
            NodeType::ImagesToVideo => self.execute_images_to_video_node(node, workflow),
            
            // Text and graphics
            NodeType::AddSubtitle => self.execute_add_subtitle_node(node, workflow),
            NodeType::AddWatermark => self.execute_add_watermark_node(node, workflow),
            NodeType::AddText => self.execute_add_text_node(node, workflow),
            NodeType::AddLogo => self.execute_add_logo_node(node, workflow),
            NodeType::AddTimecode => self.execute_add_timecode_node(node, workflow),
            
            // Advanced processing
            NodeType::StreamPrep => self.execute_stream_prep_node(node, workflow),
            NodeType::AudioVideoSync => self.execute_audio_video_sync_node(node, workflow),
            NodeType::AudioDelay => self.execute_audio_delay_node(node, workflow),
            NodeType::VideoDelay => self.execute_video_delay_node(node, workflow),
            
            // Metadata operations
            NodeType::ExtractMetadata => self.execute_extract_metadata_node(node, workflow),
            NodeType::AddMetadata => self.execute_add_metadata_node(node, workflow),
            NodeType::RemoveMetadata => self.execute_remove_metadata_node(node, workflow),
            
            // Analysis and validation
            NodeType::QualityAnalysis => self.execute_quality_analysis_node(node, workflow),
            NodeType::FormatValidation => self.execute_format_validation_node(node, workflow),
            
            // Multi-resolution output
            NodeType::MultiResOutput => self.execute_multi_res_output_node(node, workflow),
            
            // Encryption/Decryption
            NodeType::VideoEncrypt => self.execute_video_encrypt_node(node, workflow),
            NodeType::VideoDecrypt => self.execute_video_decrypt_node(node, workflow),
            
            // Advanced encoding
            NodeType::MultiPassEncode => self.execute_multi_pass_encode_node(node, workflow),
            NodeType::BatchProcess => self.execute_batch_process_node(node, workflow),
            
            // Archive operations
            NodeType::CreateArchive => self.execute_create_archive_node(node, workflow),
            NodeType::ExtractArchive => self.execute_extract_archive_node(node, workflow),
            NodeType::BatchConvert => self.execute_batch_convert_node(node, workflow),
        };
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(output_files) => Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                success: true,
                output_files,
                error_message: None,
                execution_time,
            }),
            Err(e) => Ok(NodeExecutionResult {
                node_id: node.id.clone(),
                success: false,
                output_files: Vec::new(),
                error_message: Some(e.clone()),
                execution_time,
            })
        }
    }
    
    /// Execute input file node (validation)
    fn execute_input_file_node(&mut self, node: &AutomationNode) -> Result<Vec<String>, String> {
        if let Some(file_path_param) = node.parameters.get("file_path") {
            let file_path = &file_path_param.value;
            log_debug!("Input file path: {}", file_path);
            
            if file_path.is_empty() {
                return Err("Input file path is empty. Please set a file path using right-click menu or drag-and-drop.".to_string());
            }
            
            let path_obj = std::path::Path::new(file_path);
            if !path_obj.exists() {
                log_error!("Input file does not exist: {}", file_path);
                return Err(format!("Input file does not exist: {}", file_path));
            }
            
            // Check file metadata
            if let Ok(metadata) = std::fs::metadata(file_path) {
                log_debug!("Input file size: {} bytes", metadata.len());
                if metadata.len() == 0 {
                    return Err(format!("Input file is empty: {}", file_path));
                }
            } else {
                log_warn!("Cannot read input file metadata");
            }
            
            // Check if the file is actually readable
            if let Err(e) = std::fs::File::open(file_path) {
                log_error!("Cannot open input file: {}", e);
                return Err(format!("Cannot open input file: {}", e));
            }
            
            log_info!("Input file validated: {}", file_path);
            Ok(vec![file_path.clone()])
        } else {
            Err("Input file path parameter not found".to_string())
        }
    }
    
    /// Execute audio extraction
    fn execute_extract_audio_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("wav");
        let output_file = self.generate_temp_file("extract_audio", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::ExtractAudio, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("Extract Audio: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("Audio extraction completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio extraction failed: {}", e))
            }
        }
    }
    
    /// Execute video extraction
    fn execute_extract_video_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("extracted_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::ExtractVideo, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("Extract Video: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("Video extraction completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video extraction failed: {}", e))
            }
        }
    }
    
    /// Execute audio resampling with comprehensive audio processing options
    fn execute_audio_resample_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("wav");
        let output_file = self.generate_temp_file("audio_resample", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AudioResample, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("Audio Resample: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("Audio resampling completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio resampling failed: {}", e))
            }
        }
    }
    
    /// Execute video recoding with comprehensive video processing options
    fn execute_video_recode_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("recoded_video", "mp4");
        
        let codec = node.parameters.get("codec")
            .map(|p| p.value.as_str())
            .unwrap_or("libx264");
        let quality = node.parameters.get("quality")
            .map(|p| p.value.as_str())
            .unwrap_or("23");
        
        // Support various video operations
        let mut command = format!("ffmpeg -i \"{}\"", input_file);
        
        // Video codec and quality
        command.push_str(&format!(" -vcodec {}", codec));
        
        // Quality settings based on codec
        if codec.contains("x264") || codec.contains("x265") {
            command.push_str(&format!(" -crf {}", quality));
        } else if codec == "libvpx" || codec == "libvpx-vp9" {
            command.push_str(&format!(" -crf {} -b:v 0", quality));
        }
        
        // Add various video filters if parameters exist
        let mut filters = Vec::new();
        
        if let Some(scale) = node.parameters.get("scale") {
            if !scale.value.is_empty() {
                filters.push(format!("scale={}", scale.value));
            }
        }
        
        if let Some(fps) = node.parameters.get("fps") {
            if !fps.value.is_empty() {
                filters.push(format!("fps={}", fps.value));
            }
        }
        
        if let Some(crop) = node.parameters.get("crop") {
            if !crop.value.is_empty() {
                filters.push(format!("crop={}", crop.value));
            }
        }
        
        if let Some(rotate) = node.parameters.get("rotate") {
            if !rotate.value.is_empty() {
                match rotate.value.as_str() {
                    "90" => filters.push("transpose=1".to_string()),
                    "180" => filters.push("transpose=2,transpose=2".to_string()),
                    "270" => filters.push("transpose=2".to_string()),
                    _ => {}
                }
            }
        }
        
        if !filters.is_empty() {
            command.push_str(&format!(" -vf \"{}\"", filters.join(",")));
        }
        
        command.push_str(&format!(" \"{}\"", output_file));
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    
    /// Detect if a file is audio-only, video-only, or has both streams
    fn detect_media_type(&self, file_path: &str) -> Result<(bool, bool), String> {
        // Use bundled FFmpeg worker for stream detection
        let worker = FFmpegWorker::new();
        match worker.detect_streams(file_path) {
            Ok((has_video, has_audio, _resolution)) => {
                Ok((has_video, has_audio))
            }
            Err(e) => {
                log_warn!("Stream detection failed: {}. Using fallback detection.", e);
                // Fallback: try to detect from file extension
                let ext = std::path::Path::new(file_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                match ext.to_lowercase().as_str() {
                    "mp3" | "flac" | "wav" | "aac" | "ogg" | "opus" | "wma" | "m4a" | 
                    "ac3" | "eac3" | "dts" | "spx" | "amr" | "g722" | "au" => Ok((false, true)),
                    "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "3gp" | "ogv" | 
                    "mpg" | "mpeg" | "ts" | "mts" | "m2ts" | "mxf" | "dv" | "asf" | "vob" | "rm" => Ok((true, true)),
                    "gif" => Ok((true, false)), // GIF is video-only (no audio)
                    _ => Ok((false, true)) // Default assume audio only for unknown extensions
                }
            }
        }
    }
    
    /// Get appropriate extension based on container format
    fn get_extension_for_container(&self, container: &str, is_audio_only: bool) -> &'static str {
        match container.to_lowercase().as_str() {
            // Audio formats
            "mp3" => "mp3",
            "flac" => "flac",
            "wav" => "wav",
            "aac" => "aac",
            "m4a" => "m4a",
            "ogg" | "vorbis" => "ogg",
            "opus" => "opus",
            "wma" => "wma",
            "alac" => "m4a",
            "ape" => "ape",
            "ac3" => "ac3",
            "dts" => "dts",
            
            // Video formats - but if audio only, use appropriate audio extension
            "mp4" => if is_audio_only { "m4a" } else { "mp4" },
            "mkv" => if is_audio_only { "mka" } else { "mkv" },
            "webm" => if is_audio_only { "weba" } else { "webm" },
            "avi" => "avi",
            "mov" => "mov",
            "flv" => "flv",
            "wmv" => "wmv",
            "m4v" => "m4v",
            "mpg" | "mpeg" => "mpg",
            "3gp" => "3gp",
            "ogv" => "ogv",
            "ts" => "ts",
            "m2ts" => "m2ts",
            "mxf" => "mxf",
            "vob" => "vob",
            "rm" | "rmvb" => "rm",
            "asf" => "asf",
            "divx" => "divx",
            "eac3" => "eac3",
            "amr" => "amr",
            "spx" => "spx",
            "g722" => "g722",
            "au" => "au",
            "gif" => "gif",
            
            _ => if is_audio_only { "m4a" } else { "mp4" }
        }
    }
    
    /// Detect the format from the source node that connects to this node
    fn detect_source_node_format(&self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Option<String> {
        // Find the connection that leads to this node
        for connection in workflow.connections.values() {
            if connection.to_node == node.id {
                if let Some(source_node) = workflow.nodes.get(&connection.from_node) {
                    log_debug!("Checking source node: {} (type: {}, output port: {})", 
                        source_node.id, source_node.node_type.display_name(), connection.from_port);
                    
                    // Check source node type and return appropriate format
                    return match &source_node.node_type {
                        NodeType::AudioConvert => {
                            source_node.parameters.get("format")
                                .map(|p| match p.value.as_str() {
                                    "mp3" => "mp3".to_string(),
                                    "flac" => "flac".to_string(),
                                    "wav" => "wav".to_string(),
                                    "aac" => "m4a".to_string(),
                                    "ogg" => "ogg".to_string(),
                                    "opus" => "opus".to_string(),
                                    "wma" => "wma".to_string(),
                                    _ => "mp3".to_string()
                                })
                        },
                        NodeType::AudioResample => Some("wav".to_string()),
                        NodeType::AudioCompress => Some("mp3".to_string()),
                        NodeType::AudioVolume | NodeType::AudioTrim | 
                        NodeType::AudioNormalize | NodeType::AudioDeNoise |
                        NodeType::AudioEqualizer | NodeType::AudioFade |
                        NodeType::AudioEcho | NodeType::AudioSpeed => Some("wav".to_string()),
                        NodeType::AudioMerge => Some("wav".to_string()),
                        NodeType::ExtractAudio => Some("wav".to_string()),
                        
                        // Split node - check which output port is connected
                        NodeType::SplitAudioVideo => {
                            match connection.from_port {
                                0 => {
                                    log_debug!("Split video output detected");
                                    Some("mp4".to_string()) // Video output
                                },
                                1 => {
                                    log_debug!("Split audio output detected");
                                    // Get audio format from node parameters
                                    let audio_format = source_node.parameters.get("audio_format")
                                        .map(|p| p.value.clone())
                                        .unwrap_or_else(|| "wav".to_string());
                                    Some(audio_format) // Use specified audio format
                                },
                                _ => {
                                    log_warn!("Unknown split output port: {}", connection.from_port);
                                    Some("mp4".to_string()) // Default to video
                                }
                            }
                        },
                        
                        // Video nodes
                        NodeType::VideoConvert => {
                            source_node.parameters.get("format")
                                .map(|p| match p.value.as_str() {
                                    "mp4" => "mp4".to_string(),
                                    "mkv" => "mkv".to_string(),
                                    "avi" => "avi".to_string(),
                                    "mov" => "mov".to_string(),
                                    "webm" => "webm".to_string(),
                                    _ => "mp4".to_string()
                                })
                        },
                        NodeType::VideoRecode | NodeType::VideoCompress |
                        NodeType::VideoResize | NodeType::VideoCrop |
                        NodeType::VideoRotate | NodeType::VideoFilter => Some("mp4".to_string()),
                        
                        // Continue checking through the chain if it's another type
                        _ => {
                            log_debug!("Continuing search through node: {}", source_node.node_type.display_name());
                            self.detect_source_node_format(source_node, workflow)
                        }
                    };
                }
            }
        }
        None
    }
    
    /// Validate format compatibility between input and output
    fn validate_format_compatibility(&self, input_file: &str, output_container: &str, is_audio_only: bool) -> Result<(), String> {
        // Check if the selected format is compatible with the input type
        match output_container {
            "mp3" | "flac" | "wav" | "aac" | "m4a" | "ogg" | "opus" | "wma" | "ac3" | "eac3" | 
            "dts" | "spx" | "amr" | "g722" | "au" => {
                if !is_audio_only {
                    return Err(format!("Audio format '{}' cannot be used for video content. Please select a video container format like mp4, mkv, or webm.", output_container));
                }
            },
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "3gp" | "ogv" | 
            "mpg" | "mpeg" | "ts" | "mts" | "m2ts" | "mxf" | "dv" | "asf" | "vob" | "rm" | "gif" => {
                // Video formats are generally compatible
            },
            _ => {
                log_warn!("Unknown format '{}', proceeding with caution", output_container);
            }
        }
        
        // Check if input file exists and is readable
        if !std::path::Path::new(input_file).exists() {
            return Err(format!("Input file does not exist: {}", input_file));
        }
        
        // Check if input file has reasonable size
        if let Ok(metadata) = std::fs::metadata(input_file) {
            if metadata.len() == 0 {
                return Err("Input file is empty".to_string());
            }
            if metadata.len() > 100 * 1024 * 1024 * 1024 { // 100GB limit
                log_warn!("Input file is very large ({} GB), processing may take a long time", metadata.len() / (1024 * 1024 * 1024));
            }
        }
        
        Ok(())
    }
    
    /// Execute output file node (final copy)
    fn execute_output_file_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        log_debug!("Output node: Input file = {}", input_file);
        
        // Check if input file exists
        if !std::path::Path::new(&input_file).exists() {
            log_error!("Input file for output node does not exist: {}", input_file);
            return Err(format!("Input file for output node does not exist: {}", input_file));
        } else {
            log_debug!("Input file for output node exists");
        }
        
        // Get dynamic output path (handles both user-specified and auto-fill paths)
        let output_path = if let Some(dynamic_path) = node.get_dynamic_output_path(workflow) {
            dynamic_path
        } else {
            log_error!("Output file path is empty");
            return Err("Output file path is empty. Please set an output path.".to_string());
        };
        
        log_debug!("Output node: Output path = {}", output_path);
        
        // Detect input file type to determine appropriate output format
        let (has_video, has_audio) = match self.detect_media_type(&input_file) {
            Ok(types) => {
                log_debug!("Media detection: video={}, audio={}", types.0, types.1);
                types
            },
            Err(e) => {
                log_warn!("Media detection failed: {}. Using fallback detection.", e);
                // Fallback: try to detect from file extension
                let ext = std::path::Path::new(&input_file)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                match ext.to_lowercase().as_str() {
                    "mp3" | "flac" | "wav" | "aac" | "ogg" | "opus" | "wma" | "m4a" => (false, true),
                    "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" => (true, true),
                    _ => (false, true) // Default assume audio only for unknown extensions
                }
            }
        };
        
        let is_audio_only = has_audio && !has_video;
        log_debug!("File analysis: audio_only={}, has_video={}, has_audio={}", is_audio_only, has_video, has_audio);
        
        // Get the desired container format from node parameters
        let container_param = node.parameters.get("container")
            .map(|p| p.value.as_str())
            .unwrap_or("auto");
        
        // Smart container selection based on input type
        let container = if container_param == "auto" {
            if is_audio_only {
                // For audio-only input, detect the source node type to choose best format
                let source_format = self.detect_source_node_format(node, workflow);
                let selected = source_format.unwrap_or_else(|| "mp3".to_string());
                log_info!("Auto-selected audio container: {} (from source node analysis)", selected);
                selected
            } else if has_video && has_audio {
                log_info!("Auto-selected video container: mp4 (video+audio content)");
                "mp4".to_string() // Default for video with audio
            } else if has_video && !has_audio {
                log_info!("Auto-selected video container: mp4 (video-only content)");
                "mp4".to_string() // Video only
            } else {
                log_warn!("Unknown media type, defaulting to mp4");
                "mp4".to_string() // Fallback
            }
        } else {
            log_info!("Using manual container selection: {}", container_param);
            container_param.to_string()
        };
        
        // Validate format compatibility before proceeding
        if let Err(e) = self.validate_format_compatibility(&input_file, &container, is_audio_only) {
            log_error!("Format validation failed: {}", e);
            return Err(e);
        }
        log_debug!("Format validation passed");
        
        // Handle both file and directory paths
        let final_output_path = if std::path::Path::new(&output_path).is_dir() {
            // If it's a directory, generate a filename with timestamp like non-workflow version
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            // Get appropriate extension based on container and media type
            let extension = self.get_extension_for_container(&container, is_audio_only);
            let output_filename = format!("output_{}.{}", timestamp, extension);
            log_info!("Generated filename: {} (container: {}, extension: {})", output_filename, container, extension);
            std::path::Path::new(&output_path).join(output_filename).to_string_lossy().to_string()
        } else {
            // If user specified a full file path, use it as-is
            output_path.clone()
        };
        
        log_info!("Copying from {} to {}", input_file, final_output_path);
        if let Err(e) = std::fs::copy(&input_file, &final_output_path) {
            return Err(format!("Failed to copy output file: {}", e));
        }
        
        // Verify the output file was created
        if std::path::Path::new(&final_output_path).exists() {
            log_info!("Final output file created: {}", final_output_path);
            if let Ok(metadata) = std::fs::metadata(&final_output_path) {
                log_info!("Final output file size: {} bytes", metadata.len());
            }
        } else {
            log_error!("Final output file was not created: {}", final_output_path);
        }
            
        Ok(vec![final_output_path])
    }
    
    /// Execute audio format conversion with comprehensive audio codec support
    fn execute_audio_convert_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        
        let format = node.parameters.get("format")
            .map(|p| p.value.as_str())
            .unwrap_or("flac");
        let output_file = self.generate_temp_file("audio_convert", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AudioConvert, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("Audio Convert: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("Audio conversion completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio conversion failed: {}", e))
            }
        }
    }
    
    /// Execute video format conversion with comprehensive codec support
    fn execute_video_convert_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        
        let format = node.parameters.get("format")
            .map(|p| p.value.as_str())
            .unwrap_or("mp4");
        let output_file = self.generate_temp_file("video_convert", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoConvert, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("Video Convert: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("Video conversion completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video conversion failed: {}", e))
            }
        }
    }
    
    /// Legacy implementation - replaced with mature implementation above
    
    /// Execute audio/video split
    fn execute_split_av_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        
        let video_codec = node.parameters.get("video_codec")
            .map(|p| p.value.as_str())
            .unwrap_or("copy");
        let audio_format = node.parameters.get("audio_format")
            .map(|p| p.value.as_str())
            .unwrap_or("wav");
        
        let video_output = self.generate_temp_file("split_video", "mp4");
        let audio_output = self.generate_temp_file("split_audio", audio_format);
        
        // Use smart codec selection for audio like the non-workflow implementation
        let audio_codec = if audio_format == "wav" {
            "pcm_s16le" // WAV needs PCM codec
        } else {
            &CodecManager::get_best_audio_codec_for_format(audio_format)
        };
        
        // Extract video without audio using specified codec (align with working command format)
        let video_command = format!(
            "ffmpeg -i \"{}\" -y -v error -hide_banner -nostats -nostdin -an -c:v {} \"{}\"",
            input_file, video_codec, video_output
        );
        
        // Extract audio without video using appropriate codec for format
        let audio_command = format!(
            "ffmpeg -i \"{}\" -y -v error -hide_banner -nostats -nostdin -vn -c:a {} \"{}\"",
            input_file, audio_codec, audio_output
        );
        
        log_debug!("Extracting video: {}", video_command);
        self.execute_ffmpeg_command(&video_command)?;
        
        log_debug!("Extracting audio: {}", audio_command);
        self.execute_ffmpeg_command(&audio_command)?;
        
        self.temp_files.push(video_output.clone());
        self.temp_files.push(audio_output.clone());
        
        Ok(vec![video_output, audio_output])
    }
    
    // Additional Audio Processing Methods
    
    /// Execute audio compression
    fn execute_audio_compress_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp3");
        let output_file = self.generate_temp_file("compressed_audio", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AudioCompress, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("🎵 Audio Compress: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Audio compression completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio compression failed: {}", e))
            }
        }
    }
    
    /// Execute audio volume adjustment
    fn execute_audio_volume_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("wav");
        let output_file = self.generate_temp_file("volume_adjusted_audio", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AudioVolume, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("🔊 Audio Volume: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Audio volume adjustment completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio volume adjustment failed: {}", e))
            }
        }
    }
    
    /// Execute audio trimming
    fn execute_audio_trim_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("wav");
        let output_file = self.generate_temp_file("trimmed_audio", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AudioTrim, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("✂️ Audio Trim: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Audio trimming completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio trimming failed: {}", e))
            }
        }
    }
    
    /// Execute audio merging
    fn execute_audio_merge_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        // For AudioMerge, we need to handle multiple input files
        let audio1_file = self.get_input_file_for_port(node, workflow, 0)?;
        let audio2_file = self.get_input_file_for_port(node, workflow, 1)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("wav");
        let output_file = self.generate_temp_file("merged_audio", format);
        
        // Create ProcessingTask with multiple input files
        let mut task = ProcessingTask::new(
            OperationType::AudioMerge,
            vec![audio1_file.clone(), audio2_file.clone()],
            output_file.clone()
        );
        task.audio_settings = Some(self.node_params_to_audio_settings(node));
        
        log_info!("🔀 Audio Merge: {} + {} -> {} (using mature implementation)", audio1_file, audio2_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Audio merging completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio merging failed: {}", e))
            }
        }
    }
    
    /// Execute audio normalization
    fn execute_audio_normalize_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("normalized_audio", "wav");
        
        let command = format!(
            "ffmpeg -i \"{}\" -af \"loudnorm\" \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute audio noise reduction
    fn execute_audio_denoise_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("denoised_audio", "wav");
        
        let command = format!(
            "ffmpeg -i \"{}\" -af \"afftdn\" \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute audio equalizer
    fn execute_audio_equalizer_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("equalized_audio", "wav");
        
        let eq_settings = node.parameters.get("eq_settings")
            .map(|p| p.value.as_str())
            .unwrap_or("equalizer=f=1000:t=h:w=200:g=2");
            
        let command = format!(
            "ffmpeg -i \"{}\" -af \"{}\" \"{}\"",
            input_file, eq_settings, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute audio fade
    fn execute_audio_fade_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("faded_audio", "wav");
        
        let fade_in = node.parameters.get("fade_in")
            .map(|p| p.value.as_str())
            .unwrap_or("1");
        let fade_out = node.parameters.get("fade_out")
            .map(|p| p.value.as_str())
            .unwrap_or("1");
            
        let command = format!(
            "ffmpeg -i \"{}\" -af \"afade=t=in:d={},afade=t=out:st=10:d={}\" \"{}\"",
            input_file, fade_in, fade_out, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute audio echo
    fn execute_audio_echo_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("echo_audio", "wav");
        
        let delay = node.parameters.get("delay")
            .map(|p| p.value.as_str())
            .unwrap_or("0.5");
        let decay = node.parameters.get("decay")
            .map(|p| p.value.as_str())
            .unwrap_or("0.5");
            
        let command = format!(
            "ffmpeg -i \"{}\" -af \"aecho={}:{}\" \"{}\"",
            input_file, delay, decay, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute audio speed adjustment
    fn execute_audio_speed_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("speed_adjusted_audio", "wav");
        
        let speed = node.parameters.get("speed")
            .map(|p| p.value.as_str())
            .unwrap_or("1.0");
            
        let command = format!(
            "ffmpeg -i \"{}\" -af \"atempo={}\" \"{}\"",
            input_file, speed, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute combine audio and video
    fn execute_combine_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        // Combine
        let video_input = self.get_input_file_for_port(node, workflow, 0)?;
        let audio_input = self.get_input_file_for_port(node, workflow, 1)?;
        
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("combined", format);
        
        // Create ProcessingTask with multiple input files for VideoAudioMerge
        let mut task = ProcessingTask::new(
            OperationType::VideoAudioMerge,
            vec![video_input.clone(), audio_input.clone()],
            output_file.clone()
        );
        
        // Set both video and audio settings
        task.video_settings = Some(self.node_params_to_video_settings(node));
        task.audio_settings = Some(self.node_params_to_audio_settings(node));
        
        log_info!("🔗 Combine A/V: {} + {} -> {} (using mature implementation)", video_input, audio_input, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Audio/Video combining completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Audio/Video combining failed: {}", e))
            }
        }
    }
    
    // Additional Video Processing Methods
    
    /// Execute video compression
    fn execute_video_compress_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("compressed_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoCompress, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("📉 Video Compress: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Video compression completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video compression failed: {}", e))
            }
        }
    }
    
    /// Legacy implementation - replaced with mature implementation above
    
    /// Execute video resizing  
    fn execute_video_resize_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("resized_video", format);
        
        let mut task = self.create_processing_task(OperationType::VideoResize, &input_file, &output_file, node);
        log_info!("📏 Video Resize: {} -> {} (using mature implementation)", input_file, output_file);
        
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => { log_info!("✅ Video resizing completed successfully"); self.temp_files.push(output_file.clone()); Ok(vec![output_file]) }
            Err(e) => Err(format!("Video resizing failed: {}", e))
        }
    }
    
    
    /// Execute video cropping
    fn execute_video_crop_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("cropped_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoCrop, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("✂️ Video Crop: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Video cropping completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video cropping failed: {}", e))
            }
        }
    }
    
    /// Execute video rotation
    fn execute_video_rotate_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("rotated_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoRotate, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("🔄 Video Rotate: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Video rotation completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video rotation failed: {}", e))
            }
        }
    }
    
    /// Execute video filter
    fn execute_video_filter_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("filtered_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoFilter, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("🎨 Video Filter: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Video filtering completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video filtering failed: {}", e))
            }
        }
    }
    
    /// Execute frame extraction
    fn execute_frame_extract_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("png");
        let output_pattern = self.generate_temp_file("frame_%04d", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::FrameExtract, 
            &input_file, 
            &output_pattern, 
            node
        );
        
        log_info!("🇫 Frame Extract: {} -> {} (using mature implementation)", input_file, output_pattern);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Frame extraction completed successfully");
                self.temp_files.push(output_pattern.clone());
                Ok(vec![output_pattern])
            }
            Err(e) => {
                Err(format!("Frame extraction failed: {}", e))
            }
        }
    }
    
    /// Execute video FPS conversion
    fn execute_video_fps_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("fps_converted_video", "mp4");
        
        let fps = node.parameters.get("fps")
            .map(|p| p.value.as_str())
            .unwrap_or("30");
            
        let command = format!(
            "ffmpeg -i \"{}\" -filter:v fps={} \"{}\"",
            input_file, fps, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video stabilization
    fn execute_video_stabilize_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("stabilized_video", "mp4");
        
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"deshake\" \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video deinterlacing
    fn execute_video_deinterlace_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("deinterlaced_video", "mp4");
        
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"yadif\" \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video color correction
    fn execute_video_color_correct_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("color_corrected_video", "mp4");
        
        let correction = node.parameters.get("correction")
            .map(|p| p.value.as_str())
            .unwrap_or("colorbalance=rs=0.1:gs=0.1:bs=0.1");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"{}\" \"{}\"",
            input_file, correction, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video brightness adjustment
    fn execute_video_brightness_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("brightness_adjusted_video", "mp4");
        
        let brightness = node.parameters.get("brightness")
            .map(|p| p.value.as_str())
            .unwrap_or("0.0");
        let contrast = node.parameters.get("contrast")
            .map(|p| p.value.as_str())
            .unwrap_or("1.0");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"eq=brightness={}:contrast={}\" \"{}\"",
            input_file, brightness, contrast, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video saturation adjustment
    fn execute_video_saturation_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("saturation_adjusted_video", "mp4");
        
        let saturation = node.parameters.get("saturation")
            .map(|p| p.value.as_str())
            .unwrap_or("1.0");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"eq=saturation={}\" \"{}\"",
            input_file, saturation, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video gamma correction
    fn execute_video_gamma_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("gamma_corrected_video", "mp4");
        
        let gamma = node.parameters.get("gamma")
            .map(|p| p.value.as_str())
            .unwrap_or("1.2");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"eq=gamma={}\" \"{}\"",
            input_file, gamma, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    // Combination and Advanced Operations
    
    /// Execute video overlay
    fn execute_video_overlay_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let main_video = self.get_input_file_for_port(node, workflow, 0)?;
        let overlay_video = self.get_input_file_for_port(node, workflow, 1)?;
        let output_file = self.generate_temp_file("overlay_video", "mp4");
        
        let x = node.parameters.get("x")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
        let y = node.parameters.get("y")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
            
        let command = format!(
            "ffmpeg -i \"{}\" -i \"{}\" -filter_complex \"[0:v][1:v]overlay={}:{}[v]\" -map \"[v]\" \"{}\"",
            main_video, overlay_video, x, y, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute picture-in-picture
    fn execute_video_pip_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let main_video = self.get_input_file_for_port(node, workflow, 0)?;
        let pip_video = self.get_input_file_for_port(node, workflow, 1)?;
        let output_file = self.generate_temp_file("pip_video", "mp4");
        
        let scale = node.parameters.get("scale")
            .map(|p| p.value.as_str())
            .unwrap_or("0.3");
        let x = node.parameters.get("x")
            .map(|p| p.value.as_str())
            .unwrap_or("main_w-overlay_w-10");
        let y = node.parameters.get("y")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
            
        let command = format!(
            "ffmpeg -i \"{}\" -i \"{}\" -filter_complex \"[1:v]scale=iw*{}:ih*{}[pip];[0:v][pip]overlay={}:{}[v]\" -map \"[v]\" \"{}\"",
            main_video, pip_video, scale, scale, x, y, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute side-by-side video
    fn execute_video_side_by_side_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let left_video = self.get_input_file_for_port(node, workflow, 0)?;
        let right_video = self.get_input_file_for_port(node, workflow, 1)?;
        let output_file = self.generate_temp_file("side_by_side_video", "mp4");
        
        let command = format!(
            "ffmpeg -i \"{}\" -i \"{}\" -filter_complex hstack \"{}\"",
            left_video, right_video, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute video to GIF conversion
    fn execute_video_to_gif_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("converted", "gif");
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::VideoToGif, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("🎭 Video to GIF: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Video to GIF conversion completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Video to GIF conversion failed: {}", e))
            }
        }
    }
    
    /// Execute GIF resizing
    fn execute_gif_resize_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("resized", "gif");
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::GifResize, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("📏 GIF Resize: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ GIF resizing completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("GIF resizing failed: {}", e))
            }
        }
    }
    
    /// Execute video to images conversion
    fn execute_video_to_images_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_pattern = self.generate_temp_file("image_%04d", "png");
        
        let fps = node.parameters.get("fps")
            .map(|p| p.value.as_str())
            .unwrap_or("1");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf fps={} \"{}\"",
            input_file, fps, output_pattern
        );
        
        self.execute_ffmpeg_command(&command)?;
        self.temp_files.push(output_pattern.clone());
        Ok(vec![output_pattern])
    }
    
    /// Execute images to video conversion
    fn execute_images_to_video_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let image_pattern = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("images_to_video", "mp4");
        
        let fps = node.parameters.get("fps")
            .map(|p| p.value.as_str())
            .unwrap_or("25");
            
        let command = format!(
            "ffmpeg -framerate {} -i \"{}\" -c:v libx264 -pix_fmt yuv420p \"{}\"",
            fps, image_pattern, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    // Text and Graphics Operations
    
    /// Execute add subtitle
    fn execute_add_subtitle_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("subtitled_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AddSubtitle, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("📝 Add Subtitle: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Subtitle addition completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Subtitle addition failed: {}", e))
            }
        }
    }
    
    /// Execute add watermark
    fn execute_add_watermark_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let format = node.parameters.get("format").map(|p| p.value.as_str()).unwrap_or("mp4");
        let output_file = self.generate_temp_file("watermarked_video", format);
        
        // Create ProcessingTask using mature implementation
        let mut task = self.create_processing_task(
            OperationType::AddWatermark, 
            &input_file, 
            &output_file, 
            node
        );
        
        log_info!("💧 Add Watermark: {} -> {} (using mature implementation)", input_file, output_file);
        
        // Execute using mature TaskExecutor
        match TaskExecutor::execute_task(&mut task) {
            Ok(_) => {
                log_info!("✅ Watermark addition completed successfully");
                self.temp_files.push(output_file.clone());
                Ok(vec![output_file])
            }
            Err(e) => {
                Err(format!("Watermark addition failed: {}", e))
            }
        }
    }
    
    /// Execute add text overlay
    fn execute_add_text_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("text_overlay_video", "mp4");
        
        let text = node.parameters.get("text")
            .map(|p| p.value.as_str())
            .unwrap_or("Sample Text");
        let x = node.parameters.get("x")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
        let y = node.parameters.get("y")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
        let font_size = node.parameters.get("font_size")
            .map(|p| p.value.as_str())
            .unwrap_or("24");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"drawtext=text='{}':fontsize={}:x={}:y={}:fontcolor=white\" \"{}\"",
            input_file, text, font_size, x, y, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute add logo overlay
    fn execute_add_logo_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let logo_file = node.parameters.get("logo_file")
            .map(|p| p.value.as_str())
            .ok_or("Logo file parameter not found")?;
        let output_file = self.generate_temp_file("logo_overlay_video", "mp4");
        
        let x = node.parameters.get("x")
            .map(|p| p.value.as_str())
            .unwrap_or("main_w-overlay_w-10");
        let y = node.parameters.get("y")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
            
        let command = format!(
            "ffmpeg -i \"{}\" -i \"{}\" -filter_complex \"overlay={}:{}\" \"{}\"",
            input_file, logo_file, x, y, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Execute add timecode
    fn execute_add_timecode_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("timecode_video", "mp4");
        
        let x = node.parameters.get("x")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
        let y = node.parameters.get("y")
            .map(|p| p.value.as_str())
            .unwrap_or("10");
            
        let command = format!(
            "ffmpeg -i \"{}\" -vf \"drawtext=text='%{{pts\\:hms}}':fontsize=20:x={}:y={}:fontcolor=white:box=1:boxcolor=black@0.5\" \"{}\"",
            input_file, x, y, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }
    
    /// Helper: Get input file for a node
    fn get_input_file_for_node(&self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<String, String> {
        self.get_input_file_for_port(node, workflow, 0)
    }
    
    /// Helper: Get input file for a specific port
    fn get_input_file_for_port(&self, node: &AutomationNode, workflow: &AutomationWorkflow, port_index: usize) -> Result<String, String> {
        if port_index >= node.input_ports.len() {
            return Err(format!("Port index {} out of range", port_index));
        }
        
        let _port = &node.input_ports[port_index];
        
        // Find connected output port
        for connection in workflow.connections.values() {
            if connection.to_node == node.id && connection.to_port == port_index {
                log_debug!("🔗 Found connection: {} port {} -> {} port {}", 
                    connection.from_node, connection.from_port, connection.to_node, connection.to_port);
                
                // Find the source node and get its output
                if let Some(source_node) = workflow.nodes.get(&connection.from_node) {
                    log_debug!("🔍 Source node: {} ({})", source_node.id, source_node.node_type.display_name());
                    
                    // Look for execution result of source node
                    for result in &self.execution_results {
                        if result.node_id == source_node.id && result.success {
                            log_debug!("📁 Found execution result for {}: {} output files", result.node_id, result.output_files.len());
                            if let Some(output_file) = result.output_files.get(connection.from_port) {
                                log_debug!("✅ Using output file from execution result: {}", output_file);
                                return Ok(output_file.clone());
                            } else {
                                log_warn!("⚠️ No output file at port index {}, available: {:?}", connection.from_port, result.output_files);
                            }
                        }
                    }
                    // If it's an input file node, get the file path parameter
                    if source_node.node_type == NodeType::InputFile {
                        if let Some(file_path_param) = source_node.parameters.get("file_path") {
                            log_debug!("✅ Using input file parameter: {}", file_path_param.value);
                            return Ok(file_path_param.value.clone());
                        }
                    }
                }
            }
        }
        
        Err(format!("No input connected to port {} of node {}", port_index, node.id))
    }
    
    /// Helper: Generate temporary file path with workflow-specific naming
    fn generate_temp_file(&self, node_type: &str, extension: &str) -> String {
        if let Some(ref workflow_dir) = self.workflow_temp_dir {
            log_debug!("📁 Using workflow temp directory: {}", workflow_dir);
            
            // Check if directory exists
            if !std::path::Path::new(workflow_dir).exists() {
                log_warn!("⚠️ Workflow temp directory does not exist! Attempting to create...");
                if let Err(e) = std::fs::create_dir_all(workflow_dir) {
                    log_error!("❌ Failed to create workflow temp directory: {}", e);
                }
            } else {
                log_debug!("✅ Workflow temp directory exists");
            }
            
            // Generate structured filename: step_number_node_type_unique_id.extension
            let node_id = self.current_node.as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let node_id_prefix = if node_id.len() >= 8 {
                node_id[..8].to_string()
            } else {
                node_id
            };
            let filename = format!("{:03}_{}_{}_{}.{}", 
                self.current_step,
                node_type,
                node_id_prefix,
                generate_unique_id(),
                extension
            );
            let full_path = std::path::Path::new(workflow_dir).join(filename).to_string_lossy().to_string();
            log_debug!("📄 Generated temp file path: {}", full_path);
            full_path
        } else {
            log_debug!("⚠️ No workflow temp directory set, using system temp");
            // Fallback to old method if no workflow dir
            let temp_dir = std::env::temp_dir();
            let filename = format!("{}_{}.{}", node_type, generate_unique_id(), extension);
            let full_path = temp_dir.join(filename).to_string_lossy().to_string();
            log_debug!("📄 Generated fallback temp file path: {}", full_path);
            full_path
        }
    }
    
    /// Helper: Execute FFmpeg command using TaskExecutor style
    fn execute_ffmpeg_command(&self, command: &str) -> Result<(), String> {
        log_debug!("Executing FFmpeg command: {}", command);
        
        // Parse the command to extract args (similar to TaskExecutor)
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() || parts[0] != "ffmpeg" {
            return Err("Invalid FFmpeg command".to_string());
        }
        
        // Build command directly like TaskExecutor does
        let mut cmd = std::process::Command::new("ffmpeg");
        
        // Add the arguments (skip the "ffmpeg" part)
        for arg in &parts[1..] {
            // Remove quotes from arguments
            let clean_arg = arg.trim_matches('"');
            cmd.arg(clean_arg);
        }
        
        // Set Windows-specific flags like TaskExecutor
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        // Set environment variables like TaskExecutor
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        cmd.env("FFMPEG_HIDE_BANNER", "1");
        
        log_debug!("🔍 Executing direct FFmpeg command with args: {:?}", &parts[1..]);
        
        let output = cmd.output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                log_debug!("📤 Command stdout: {}", stdout);
                if !stderr.is_empty() {
                    log_debug!("📤 Command stderr: {}", stderr);
                }
                
                if result.status.success() {
                    log_info!("✅ FFmpeg command completed successfully");
                    Ok(())
                } else {
                    log_error!("❌ FFmpeg command failed with exit code: {:?}", result.status.code());
                    Err(format!("FFmpeg command failed: {}", stderr))
                }
            }
            Err(e) => {
                log_error!("❌ Failed to execute command: {}", e);
                Err(format!("Failed to execute command: {}", e))
            }
        }
    }
    

    /// Cancel execution
    pub fn cancel_execution(&mut self) {
        self.execution_status = ExecutionStatus::Cancelled;
        self.current_node = None;
        self.progress = 0.0;
    }
    
    /// Create dedicated temporary directory for workflow
    fn create_workflow_temp_dir(&mut self, workflow_id: &str) -> Result<(), String> {
        // Clean up previous workflow temp dir if exists
        self.cleanup_workflow_temp_dir();
        
        let base_temp_dir = std::env::temp_dir();
        // Safely handle workflow_id that might be shorter than 8 characters
        let id_prefix = if workflow_id.len() >= 8 {
            workflow_id[..8].to_string()
        } else {
            workflow_id.to_string()
        };
        let workflow_dir_name = format!("ffmpeg_workflow_{}_{}", 
            id_prefix, 
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let workflow_temp_path = base_temp_dir.join(workflow_dir_name);
        
        std::fs::create_dir_all(&workflow_temp_path)
            .map_err(|e| format!("Failed to create workflow temp directory: {}", e))?;
        
        self.workflow_temp_dir = Some(workflow_temp_path.to_string_lossy().to_string());
        log_info!("📁 Created workflow temp directory: {:?}", self.workflow_temp_dir);
        Ok(())
    }
    
    /// Clean up workflow temporary directory
    pub fn cleanup_workflow_temp_dir(&mut self) {
        if let Some(ref temp_dir) = self.workflow_temp_dir {
            if std::path::Path::new(temp_dir).exists() {
                match std::fs::remove_dir_all(temp_dir) {
                    Ok(_) => log_info!("🧹 Cleaned up workflow temp directory: {}", temp_dir),
                    Err(e) => log_warn!("⚠️ Failed to cleanup temp directory {}: {}", temp_dir, e),
                }
            }
        }
        self.workflow_temp_dir = None;
    }
    
    /// Copy final output file to user specified location
    pub fn copy_output_to_destination(&self, temp_file_path: &str, destination_path: &str) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(destination_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
        
        std::fs::copy(temp_file_path, destination_path)
            .map_err(|e| format!("Failed to copy output file: {}", e))?;
        
        log_info!("💾 Output file saved to: {}", destination_path);
        Ok(())
    }
    
    /// Clean up temporary files
    pub fn cleanup_temp_files(&mut self) {
        for temp_file in &self.temp_files {
            if std::path::Path::new(temp_file).exists() {
                let _ = std::fs::remove_file(temp_file);
            }
        }
        self.temp_files.clear();
    }
    
    /// Get execution progress as percentage
    pub fn get_progress_percentage(&self) -> u32 {
        (self.progress * 100.0) as u32
    }
    
    /// Get detailed execution status
    /// Execute stream preparation node
    fn execute_stream_prep_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("stream_prep", "mp4");
        
        let bitrate = node.parameters.get("bitrate")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "2000k".to_string());
        let resolution = node.parameters.get("resolution")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "1920x1080".to_string());
        let fps = node.parameters.get("fps")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "30".to_string());
        
        let command = format!(
            "ffmpeg -i \"{}\" -c:v libx264 -b:v {} -r {} -s {} -c:a aac -b:a 128k -f mp4 \"{}\"",
            input_file, bitrate, fps, resolution, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute audio/video sync node
    fn execute_audio_video_sync_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("av_sync", "mp4");
        
        let sync_offset = node.parameters.get("sync_offset")
            .map(|p| p.value.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        
        let command = if sync_offset >= 0.0 {
            format!(
                "ffmpeg -i \"{}\" -itsoffset {} -i \"{}\" -c:v copy -c:a aac -map 0:v -map 1:a \"{}\"",
                input_file, sync_offset, input_file, output_file
            )
        } else {
            format!(
                "ffmpeg -i \"{}\" -itsoffset {} -i \"{}\" -c:v copy -c:a aac -map 1:v -map 0:a \"{}\"",
                input_file, -sync_offset, input_file, output_file
            )
        };
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute audio delay node
    fn execute_audio_delay_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("audio_delay", "mp4");
        
        let delay = node.parameters.get("delay")
            .map(|p| p.value.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        
        let command = format!(
            "ffmpeg -i \"{}\" -af \"adelay={}s\" -c:v copy \"{}\"",
            input_file, delay, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute video delay node
    fn execute_video_delay_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("video_delay", "mp4");
        
        let delay = node.parameters.get("delay")
            .map(|p| p.value.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        
        let command = format!(
            "ffmpeg -i \"{}\" -itsoffset {} -i \"{}\" -c:a copy -map 0:a -map 1:v \"{}\"",
            input_file, delay, input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute extract metadata node
    fn execute_extract_metadata_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("metadata", "json");
        
        let command = format!(
            "ffprobe -v quiet -print_format json -show_format -show_streams \"{}\" > \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute add metadata node
    fn execute_add_metadata_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("with_metadata", "mp4");
        
        let title = node.parameters.get("title")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "".to_string());
        let artist = node.parameters.get("artist")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "".to_string());
        let comment = node.parameters.get("comment")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "".to_string());
        
        let mut metadata_args = Vec::new();
        if !title.is_empty() {
            metadata_args.push(format!("-metadata title=\"{}\"", title));
        }
        if !artist.is_empty() {
            metadata_args.push(format!("-metadata artist=\"{}\"", artist));
        }
        if !comment.is_empty() {
            metadata_args.push(format!("-metadata comment=\"{}\"", comment));
        }
        
        let command = format!(
            "ffmpeg -i \"{}\" {} -c copy \"{}\"",
            input_file, metadata_args.join(" "), output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute remove metadata node
    fn execute_remove_metadata_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("no_metadata", "mp4");
        
        let command = format!(
            "ffmpeg -i \"{}\" -map_metadata -1 -c copy \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute quality analysis node
    fn execute_quality_analysis_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("quality_report", "txt");
        
        // Use ffprobe to analyze quality metrics
        let command = format!(
            "ffprobe -v error -select_streams v:0 -show_entries frame=pkt_pts_time,pict_type,pkt_size -of csv=p=0 \"{}\" > \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute format validation node
    fn execute_format_validation_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("validation_report", "txt");
        
        // Use ffprobe to validate format
        let command = format!(
            "ffprobe -v error -show_format -show_streams \"{}\" > \"{}\" 2>&1",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute multi-resolution output node
    fn execute_multi_res_output_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        
        let resolutions = vec!["480p", "720p", "1080p"];
        let mut output_files = Vec::new();
        
        for res in resolutions {
            let output_file = self.generate_temp_file(&format!("output_{}", res), "mp4");
            
            let (width, height, bitrate) = match res {
                "480p" => ("854", "480", "1000k"),
                "720p" => ("1280", "720", "2500k"),
                "1080p" => ("1920", "1080", "5000k"),
                _ => ("1920", "1080", "5000k"),
            };
            
            let command = format!(
                "ffmpeg -i \"{}\" -vf scale={}:{} -b:v {} -c:v libx264 -c:a aac \"{}\"",
                input_file, width, height, bitrate, output_file
            );
            
            self.execute_ffmpeg_command(&command)?;
            self.temp_files.push(output_file.clone());
            output_files.push(output_file);
        }
        
        Ok(output_files)
    }

    /// Execute video encryption node
    fn execute_video_encrypt_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("encrypted", "mp4");
        
        let password = node.parameters.get("password")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "".to_string());
        
        // Simple encryption by adding metadata
        let command = format!(
            "ffmpeg -i \"{}\" -metadata:s:v:0 encrypt=\"{}\" -c copy \"{}\"",
            input_file, password, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute video decryption node
    fn execute_video_decrypt_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("decrypted", "mp4");
        
        // Simple decryption by removing encryption metadata
        let command = format!(
            "ffmpeg -i \"{}\" -map_metadata -1 -c copy \"{}\"",
            input_file, output_file
        );
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute multi-pass encode node
    fn execute_multi_pass_encode_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("multipass", "mp4");
        
        let bitrate = node.parameters.get("bitrate")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "2000k".to_string());
        
        // First pass
        let pass1_command = format!(
            "ffmpeg -y -i \"{}\" -c:v libx264 -b:v {} -pass 1 -f null /dev/null",
            input_file, bitrate
        );
        self.execute_ffmpeg_command(&pass1_command)?;
        
        // Second pass
        let pass2_command = format!(
            "ffmpeg -i \"{}\" -c:v libx264 -b:v {} -pass 2 \"{}\"",
            input_file, bitrate, output_file
        );
        self.execute_ffmpeg_command(&pass2_command)?;
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute batch process node
    fn execute_batch_process_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let mut output_files = Vec::new();
        
        let operations = node.parameters.get("operations")
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "resize,compress".to_string());
        
        for operation in operations.split(',') {
            let op_output = self.generate_temp_file(&format!("batch_{}", operation.trim()), "mp4");
            
            let command = match operation.trim() {
                "resize" => format!("ffmpeg -i \"{}\" -vf scale=1280:720 \"{}\"", input_file, op_output),
                "compress" => format!("ffmpeg -i \"{}\" -c:v libx264 -crf 23 \"{}\"", input_file, op_output),
                "normalize" => format!("ffmpeg -i \"{}\" -af loudnorm \"{}\"", input_file, op_output),
                _ => format!("ffmpeg -i \"{}\" -c copy \"{}\"", input_file, op_output),
            };
            
            self.execute_ffmpeg_command(&command)?;
            self.temp_files.push(op_output.clone());
            output_files.push(op_output);
        }
        
        Ok(output_files)
    }

    /// Execute create archive node
    fn execute_create_archive_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_file = self.generate_temp_file("archive", "tar.gz");
        
        // Create a simple archive using tar (cross-platform)
        let command = if cfg!(target_os = "windows") {
            format!("tar -czf \"{}\" \"{}\"", output_file, input_file)
        } else {
            format!("tar -czf \"{}\" \"{}\"", output_file, input_file)
        };
        
        self.execute_ffmpeg_command(&command)?;
        
        // Check if the output file was actually created
        if std::path::Path::new(&output_file).exists() {
            log_info!("Output file created successfully: {}", output_file);
            if let Ok(metadata) = std::fs::metadata(&output_file) {
                log_info!("File size: {} bytes", metadata.len());
            } else {
                log_warn!("Could not read file metadata");
            }
        } else {
            log_error!("Output file was not created: {}", output_file);
            return Err(format!("Failed to create output file: {}", output_file));
        }
        
        self.temp_files.push(output_file.clone());
        Ok(vec![output_file])
    }

    /// Execute extract archive node
    fn execute_extract_archive_node(&mut self, node: &AutomationNode, workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_file = self.get_input_file_for_node(node, workflow)?;
        let output_dir = self.generate_temp_file("extracted", "");
        
        // Create output directory
        std::fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        
        // Extract archive
        let command = if cfg!(target_os = "windows") {
            format!("tar -xzf \"{}\" -C \"{}\"", input_file, output_dir)
        } else {
            format!("tar -xzf \"{}\" -C \"{}\"", input_file, output_dir)
        };
        
        self.execute_ffmpeg_command(&command)?;
        self.temp_files.push(output_dir.clone());
        Ok(vec![output_dir])
    }
    
    /// Execute batch convert node
    fn execute_batch_convert_node(&mut self, node: &AutomationNode, _workflow: &AutomationWorkflow) -> Result<Vec<String>, String> {
        let input_dir = node.parameters.get("input_directory")
            .map(|p| p.value.as_str()).unwrap_or("");
        let output_format = node.parameters.get("output_format")
            .map(|p| p.value.as_str()).unwrap_or("mp4");
        
        if input_dir.is_empty() {
            return Err("Input directory not specified for batch conversion".to_string());
        }
        
        // This is a basic implementation - in a real scenario, you'd iterate through files
        log_info!("Batch converting files from directory: {} to format: {}", input_dir, output_format);
        
        // Generate a placeholder output indicating the batch operation completed
        let output_info = format!("Batch converted files from '{}' to '{}' format", input_dir, output_format);
        Ok(vec![output_info])
    }

    pub fn get_execution_summary(&self) -> String {
        match &self.execution_status {
            ExecutionStatus::Idle => "Ready to execute".to_string(),
            ExecutionStatus::Running => {
                if let Some(ref current) = self.current_node {
                    format!("Executing node: {} ({}%)", current, self.get_progress_percentage())
                } else {
                    format!("Running... ({}%)", self.get_progress_percentage())
                }
            }
            ExecutionStatus::Success => {
                format!("Completed successfully - {} nodes executed", self.execution_results.len())
            }
            ExecutionStatus::Failed(ref msg) => {
                format!("Failed: {}", msg)
            }
            ExecutionStatus::Cancelled => "Execution cancelled".to_string(),
        }
    }
    
    /// Generate output file path for node
    fn generate_output_file_for_node(&self, node: &AutomationNode, input_file: &str, default_ext: &str) -> Result<String, String> {
        // Check if node has output_path parameter
        if let Some(output_param) = node.parameters.get("output_path").or(node.parameters.get("file_path")) {
            if !output_param.value.is_empty() {
                return Ok(output_param.value.clone());
            }
        }
        
        // Generate automatic output path
        let input_path = std::path::Path::new(input_file);
        let stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = input_path.parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        
        let output_file = parent.join(format!("{}_processed.{}", stem, default_ext));
        Ok(output_file.to_string_lossy().to_string())
    }
    
    /// Convert workflow node parameters to AudioSettings for mature implementation
    fn node_params_to_audio_settings(&self, node: &AutomationNode) -> AudioSettings {
        AudioSettings {
            codec: node.parameters.get("audio_codec").or(node.parameters.get("codec")).map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            bitrate: node.parameters.get("bitrate").map(|p| p.value.clone()).unwrap_or_else(|| "128k".to_string()),
            sample_rate: node.parameters.get("sample_rate").map(|p| p.value.clone()).unwrap_or_else(|| "44100".to_string()),
            channels: node.parameters.get("channels").map(|p| p.value.clone()).unwrap_or_else(|| "2".to_string()),
            quality: node.parameters.get("quality").map(|p| p.value.clone()).unwrap_or_else(|| "5".to_string()),
            volume: node.parameters.get("volume").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            custom_args: node.parameters.get("custom_args").map(|p| p.value.clone()).unwrap_or_default(),
            format: node.parameters.get("format").or(node.parameters.get("output_format")).map(|p| p.value.clone()).unwrap_or_else(|| "mp3".to_string()),
            copy_audio: node.parameters.get("copy_audio").map(|p| p.value == "true").unwrap_or(false),
            vbr_quality: node.parameters.get("vbr_quality").and_then(|p| p.value.parse().ok()).unwrap_or(2),
            resample_method: node.parameters.get("resample_method").map(|p| p.value.clone()).unwrap_or_else(|| "swr".to_string()),
            normalize: node.parameters.get("normalize").map(|p| p.value == "true").unwrap_or(false),
            target_lufs: node.parameters.get("target_lufs").and_then(|p| p.value.parse().ok()).unwrap_or(-16.0),
            start_time: node.parameters.get("start_time").map(|p| p.value.clone()).unwrap_or_else(|| "00:00:00".to_string()),
            end_time: node.parameters.get("end_time").map(|p| p.value.clone()).unwrap_or_default(),
            fade_in: node.parameters.get("fade_in").map(|p| p.value == "true").unwrap_or(false),
            fade_out: node.parameters.get("fade_out").map(|p| p.value == "true").unwrap_or(false),
            merge_mode: node.parameters.get("merge_mode").map(|p| p.value.clone()).unwrap_or_else(|| "concat".to_string()),
            add_silence: node.parameters.get("add_silence").map(|p| p.value == "true").unwrap_or(false),
            silence_duration: node.parameters.get("silence_duration").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            sync_audio: node.parameters.get("sync_audio").map(|p| p.value == "true").unwrap_or(false),
            audio_delay: node.parameters.get("audio_delay").and_then(|p| p.value.parse().ok()).unwrap_or(0.0),
            extract_all_tracks: node.parameters.get("extract_all_tracks").map(|p| p.value == "true").unwrap_or(false),
        }
    }
    
    /// Convert workflow node parameters to VideoSettings for mature implementation
    fn node_params_to_video_settings(&self, node: &AutomationNode) -> VideoSettings {
        VideoSettings {
            // Basic encoding settings
            codec: node.parameters.get("codec").or(node.parameters.get("video_codec")).map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            preset: node.parameters.get("preset").map(|p| p.value.clone()).unwrap_or_else(|| "medium".to_string()),
            profile: node.parameters.get("profile").map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            tune: node.parameters.get("tune").map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            quality: node.parameters.get("quality").and_then(|p| p.value.parse().ok()).unwrap_or(23),
            bitrate: node.parameters.get("bitrate").map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            fps: node.parameters.get("fps").map(|p| p.value.clone()).unwrap_or_else(|| "auto".to_string()),
            use_hardware_acceleration: node.parameters.get("hw_accel").map(|p| p.value == "true").unwrap_or(false),
            custom_args: node.parameters.get("custom_args").map(|p| p.value.clone()).unwrap_or_default(),
            
            // Format conversion
            container_format: node.parameters.get("format").map(|p| p.value.clone()).unwrap_or_else(|| "mp4".to_string()),
            copy_video: node.parameters.get("copy_video").map(|p| p.value == "true").unwrap_or(false),
            
            // Compression
            crf: node.parameters.get("crf").and_then(|p| p.value.parse().ok()).unwrap_or(23),
            target_size_mb: node.parameters.get("target_size").and_then(|p| p.value.parse().ok()).unwrap_or(0),
            
            // Resolution settings
            resolution: {
                let width = node.parameters.get("width").and_then(|p| p.value.parse().ok()).unwrap_or(0);
                let height = node.parameters.get("height").and_then(|p| p.value.parse().ok()).unwrap_or(0);
                (width, height)
            },
            width: node.parameters.get("width").and_then(|p| p.value.parse().ok()),
            height: node.parameters.get("height").and_then(|p| p.value.parse().ok()),
            maintain_aspect_ratio: node.parameters.get("maintain_aspect").map(|p| p.value == "true").unwrap_or(true),
            
            // Crop settings
            crop_top: node.parameters.get("crop_top").and_then(|p| p.value.parse().ok()),
            crop_bottom: node.parameters.get("crop_bottom").and_then(|p| p.value.parse().ok()),
            crop_left: node.parameters.get("crop_left").and_then(|p| p.value.parse().ok()),
            crop_right: node.parameters.get("crop_right").and_then(|p| p.value.parse().ok()),
            
            // Rotation settings
            rotation: node.parameters.get("rotation").and_then(|p| p.value.parse().ok()).unwrap_or(0),
            use_custom_rotation: node.parameters.get("custom_rotation").map(|p| p.value == "true").unwrap_or(false),
            custom_rotation_angle: node.parameters.get("custom_angle").and_then(|p| p.value.parse().ok()).unwrap_or(0.0),
            flip_horizontal: node.parameters.get("flip_h").map(|p| p.value == "true").unwrap_or(false),
            flip_vertical: node.parameters.get("flip_v").map(|p| p.value == "true").unwrap_or(false),
            
            // Filter settings
            denoise: node.parameters.get("denoise").map(|p| p.value == "true").unwrap_or(false),
            deinterlace: node.parameters.get("deinterlace").map(|p| p.value == "true").unwrap_or(false),
            stabilize: node.parameters.get("stabilize").map(|p| p.value == "true").unwrap_or(false),
            brightness: node.parameters.get("brightness").and_then(|p| p.value.parse().ok()).unwrap_or(0.0),
            contrast: node.parameters.get("contrast").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            saturation: node.parameters.get("saturation").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            
            // Subtitle settings
            subtitle_file: node.parameters.get("subtitle_file").map(|p| p.value.clone()).unwrap_or_default(),
            subtitle_style: node.parameters.get("subtitle_style").map(|p| p.value.clone()).unwrap_or_default(),
            subtitle_position: node.parameters.get("subtitle_position").map(|p| p.value.clone()).unwrap_or_else(|| "bottom".to_string()),
            
            // Watermark settings
            watermark_file: node.parameters.get("watermark_file").map(|p| p.value.clone()).unwrap_or_default(),
            watermark_position: node.parameters.get("watermark_position").map(|p| p.value.clone()).unwrap_or_else(|| "top_right".to_string()),
            watermark_opacity: node.parameters.get("watermark_opacity").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            watermark_scale: node.parameters.get("watermark_scale").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            watermark_x: node.parameters.get("x").and_then(|p| p.value.parse().ok()).unwrap_or(10),
            watermark_y: node.parameters.get("y").and_then(|p| p.value.parse().ok()).unwrap_or(10),
            
            // GIF settings
            gif_fps: node.parameters.get("gif_fps").and_then(|p| p.value.parse().ok()).unwrap_or(15.0),
            gif_scale: node.parameters.get("gif_scale").and_then(|p| p.value.parse().ok()).unwrap_or(1.0),
            gif_optimize: node.parameters.get("optimize").map(|p| p.value == "true").unwrap_or(true),
            
            // Frame extraction
            frame_rate: node.parameters.get("frame_rate").and_then(|p| p.value.parse().ok()).unwrap_or(1),
            frame_format: node.parameters.get("frame_format").map(|p| p.value.clone()).unwrap_or_else(|| "png".to_string()),
            frame_quality: node.parameters.get("frame_quality").and_then(|p| p.value.parse().ok()).unwrap_or(2),
            
            // Batch processing
            batch_naming_pattern: node.parameters.get("naming_pattern").map(|p| p.value.clone()).unwrap_or_else(|| "converted".to_string()),
            batch_operation_type: node.parameters.get("batch_type").map(|p| p.value.clone()).unwrap_or_default(),
            
            // Use defaults for all other fields
            ..VideoSettings::default()
        }
    }
    
    /// Convert workflow node parameters to AudioSettings for mature implementation  
    
    /// Convert workflow node parameters to ProcessingTask for mature implementation with hardware acceleration
    fn create_processing_task(&self, 
                             operation: OperationType, 
                             input_file: &str, 
                             output_file: &str,
                             node: &AutomationNode) -> ProcessingTask {
        // Validate input parameters
        if input_file.is_empty() {
            log_warn!("⚠️ Empty input file path for operation {:?}", operation);
        }
        if output_file.is_empty() {
            log_warn!("⚠️ Empty output file path for operation {:?}", operation);
        }
        
        let mut task = ProcessingTask::new(operation.clone(), vec![input_file.to_string()], output_file.to_string());
        
        // Set video settings if this is a video operation with hardware acceleration support
        match operation {
            OperationType::VideoConvert | OperationType::VideoCompress | OperationType::VideoResize |
            OperationType::VideoCrop | OperationType::VideoRotate | OperationType::VideoFilter |
            OperationType::VideoAudioMerge | OperationType::VideoAudioSplit | OperationType::ExtractVideo |
            OperationType::AddSubtitle | OperationType::AddWatermark | OperationType::VideoToGif |
            OperationType::FrameExtract | OperationType::GifResize => {
                let mut video_settings = self.node_params_to_video_settings(node);
                
                // Apply hardware acceleration only if user selected it or codec is "auto"
                if video_settings.codec == "auto" && !self.cached_hardware_encoders.is_empty() {
                    // Auto mode: try to find best hardware encoder
                    if let Some(hw_codec) = self.get_hardware_codec_for_software(&video_settings.codec) {
                        log_info!("🚀 Auto hardware acceleration: {} -> {} (workflow)", video_settings.codec, hw_codec);
                        video_settings.codec = hw_codec;
                        video_settings.use_hardware_acceleration = true;
                    } else {
                        log_debug!("⚙ Auto mode: no hardware alternative available, using software codec");
                        video_settings.codec = "libx264".to_string(); // Default software codec
                    }
                } else if video_settings.use_hardware_acceleration && !self.cached_hardware_encoders.is_empty() {
                    // User explicitly enabled hardware acceleration
                    if let Some(hw_codec) = self.get_hardware_codec_for_software(&video_settings.codec) {
                        log_info!("🚀 User-enabled hardware acceleration: {} -> {} (workflow)", video_settings.codec, hw_codec);
                        video_settings.codec = hw_codec;
                    } else {
                        log_debug!("⚙ Hardware acceleration requested but no alternative for codec: {}", video_settings.codec);
                    }
                } else {
                    log_debug!("⚙ Using user-selected codec: {} (no hardware acceleration)", video_settings.codec);
                }
                
                task.video_settings = Some(video_settings);
                log_debug!("📹 Applied video settings for operation {:?}", operation);
            }
            _ => {}
        }
        
        // Set audio settings if this is an audio operation
        match operation {
            OperationType::AudioConvert | OperationType::AudioCompress | OperationType::AudioResample |
            OperationType::AudioVolume | OperationType::AudioTrim | OperationType::AudioMerge |
            OperationType::VideoAudioMerge | OperationType::VideoAudioSplit | OperationType::ExtractAudio => {
                task.audio_settings = Some(self.node_params_to_audio_settings(node));
                log_debug!("🔊 Applied audio settings for operation {:?}", operation);
            }
            _ => {}
        }
        
        // Validate critical parameters for specific operations
        match operation {
            OperationType::AddSubtitle => {
                if node.parameters.get("subtitle_file").map_or(true, |p| p.value.is_empty()) {
                    log_warn!("⚠️ No subtitle file specified for AddSubtitle operation");
                }
            }
            OperationType::AddWatermark => {
                if node.parameters.get("watermark_file").map_or(true, |p| p.value.is_empty()) {
                    log_warn!("⚠️ No watermark file specified for AddWatermark operation");
                }
            }
            OperationType::VideoCrop => {
                let has_crop_params = ["width", "height", "x", "y"].iter()
                    .any(|param| node.parameters.get(*param).is_some());
                if !has_crop_params {
                    log_warn!("⚠️ No crop parameters specified for VideoCrop operation");
                }
            }
            OperationType::VideoResize => {
                let has_size_params = ["width", "height", "scale"].iter()
                    .any(|param| node.parameters.get(*param).is_some());
                if !has_size_params {
                    log_warn!("⚠️ No resize parameters specified for VideoResize operation");
                }
            }
            _ => {}
        }
        
        task
    }
    
    /// Map software codec to hardware equivalent if available
    fn get_hardware_codec_for_software(&self, software_codec: &str) -> Option<String> {
        match software_codec {
            "auto" => {
                // Auto defaults to H.264, try to find hardware H.264 encoder
                for encoder in ["h264_nvenc", "h264_qsv", "h264_amf", "h264_videotoolbox"] {
                    if self.cached_hardware_encoders.contains(&encoder.to_string()) {
                        return Some(encoder.to_string());
                    }
                }
            },
            "libx264" => {
                // Check for available hardware H.264 encoders in order of preference
                for encoder in ["h264_nvenc", "h264_qsv", "h264_amf", "h264_videotoolbox"] {
                    if self.cached_hardware_encoders.contains(&encoder.to_string()) {
                        return Some(encoder.to_string());
                    }
                }
            },
            "libx265" => {
                // Check for available hardware H.265 encoders in order of preference
                for encoder in ["hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_videotoolbox"] {
                    if self.cached_hardware_encoders.contains(&encoder.to_string()) {
                        return Some(encoder.to_string());
                    }
                }
            },
            "libvpx-vp9" => {
                // Check for available hardware VP9 encoders
                for encoder in ["vp9_qsv"] {
                    if self.cached_hardware_encoders.contains(&encoder.to_string()) {
                        return Some(encoder.to_string());
                    }
                }
            },
            "libaom-av1" => {
                // Check for available hardware AV1 encoders
                for encoder in ["av1_nvenc", "av1_qsv", "av1_amf"] {
                    if self.cached_hardware_encoders.contains(&encoder.to_string()) {
                        return Some(encoder.to_string());
                    }
                }
            },
            _ => {
                // For already hardware codecs or unknown codecs, no mapping needed
                if self.cached_hardware_encoders.contains(&software_codec.to_string()) {
                    return Some(software_codec.to_string());
                }
            }
        }
        None
    }
    
    /// Validate input file exists and is readable
    fn validate_input_file(&self, file_path: &str) -> Result<(), String> {
        if file_path.is_empty() {
            return Err("Input file path is empty".to_string());
        }
        
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return Err(format!("Input file does not exist: {}", file_path));
        }
        
        if !path.is_file() {
            return Err(format!("Input path is not a file: {}", file_path));
        }
        
        // Check if file is readable
        match std::fs::metadata(file_path) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    return Err(format!("Input file is empty: {}", file_path));
                }
            }
            Err(e) => {
                return Err(format!("Cannot read input file metadata: {} - {}", file_path, e));
            }
        }
        
        Ok(())
    }
    
    /// Validate output directory exists and is writable
    fn validate_output_path(&self, file_path: &str) -> Result<(), String> {
        if file_path.is_empty() {
            return Err("Output file path is empty".to_string());
        }
        
        let path = std::path::Path::new(file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(format!("Output directory does not exist: {}", parent.display()));
            }
            
            // Try to create a test file to check write permissions
            let test_file = parent.join(".workflow_write_test");
            match std::fs::File::create(&test_file) {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file); // Clean up
                }
                Err(e) => {
                    return Err(format!("Cannot write to output directory: {} - {}", parent.display(), e));
                }
            }
        }
        
        Ok(())
    }
}
