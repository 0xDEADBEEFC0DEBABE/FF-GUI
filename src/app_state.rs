#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OperationType {
    // Video processing
    VideoConvert,           // Video format conversion
    VideoCompress,          // Video compression
    VideoResize,            // Resolution adjustment
    VideoCrop,              // Video cropping
    VideoRotate,            // Video rotation
    VideoFilter,            // Video filters

    // Audio processing
    AudioConvert,           // Audio format conversion
    AudioCompress,          // Audio compression
    AudioResample,          // Audio resampling
    AudioVolume,            // Volume adjustment
    AudioTrim,              // Audio trimming
    AudioMerge,             // Audio merging

    // Video/Audio combinations
    VideoAudioMerge,        // Video+Audio synthesis
    VideoAudioSplit,        // Video/Audio separation
    ExtractAudio,           // Audio extraction
    ExtractVideo,           // Video extraction

    // Batch processing
    BatchConvert,           // Batch conversion
    
    // Advanced features
    AddSubtitle,            // Add subtitles
    AddWatermark,           // Add watermark
    FrameExtract,           // Extract frames
    VideoToGif,             // Video to GIF
    GifResize,              // GIF resize
}

impl OperationType {
    pub fn display_name(&self, translations: &crate::language::Translations) -> String {
        match self {
            OperationType::VideoConvert => format!("🎬 {}", translations.video_convert()),
            OperationType::VideoCompress => format!("🗜 {}", translations.video_compress()),
            OperationType::VideoResize => format!("📐 {}", translations.video_resize()),
            OperationType::VideoCrop => format!("✂ {}", translations.video_crop()),
            OperationType::VideoRotate => format!("🔄 {}", translations.video_rotate()),
            OperationType::VideoFilter => format!("✨ {}", translations.video_filter()),
            
            OperationType::AudioConvert => format!("🎵 {}", translations.audio_convert()),
            OperationType::AudioCompress => format!("🗜 {}", translations.audio_compress()),
            OperationType::AudioResample => format!("⚡ {}", translations.audio_resample()),
            OperationType::AudioVolume => format!("🔊 {}", translations.audio_volume()),
            OperationType::AudioTrim => format!("✂ {}", translations.audio_trim()),
            OperationType::AudioMerge => format!("🔗 {}", translations.audio_merge()),
            
            OperationType::VideoAudioMerge => format!("🎭 {}", translations.video_audio_merge()),
            OperationType::VideoAudioSplit => format!("🎯 {}", translations.video_audio_split()),
            OperationType::ExtractAudio => format!("🎶 {}", translations.extract_audio()),
            OperationType::ExtractVideo => format!("📹 {}", translations.extract_video()),
            
            OperationType::BatchConvert => format!("📦 {}", translations.batch_convert()),
            
            OperationType::AddSubtitle => format!("💬 {}", translations.add_subtitle()),
            OperationType::AddWatermark => format!("🏷 {}", translations.add_watermark()),
            OperationType::FrameExtract => format!("📷 {}", translations.frame_extract()),
            OperationType::VideoToGif => format!("🎞 {}", translations.video_to_gif()),
            OperationType::GifResize => format!("🖼 {}", translations.gif_resize()),
        }
    }

    pub fn category(&self, translations: &crate::language::Translations) -> String {
        match self {
            OperationType::VideoConvert | OperationType::VideoCompress | 
            OperationType::VideoResize | OperationType::VideoCrop | 
            OperationType::VideoRotate | OperationType::VideoFilter => translations.video_processing().to_string(),
            
            OperationType::AudioConvert | OperationType::AudioCompress | 
            OperationType::AudioResample | OperationType::AudioVolume | 
            OperationType::AudioTrim | OperationType::AudioMerge => translations.audio_processing().to_string(),
            
            OperationType::VideoAudioMerge | OperationType::VideoAudioSplit | 
            OperationType::ExtractAudio | OperationType::ExtractVideo => translations.video_audio_operations().to_string(),
            
            OperationType::BatchConvert => translations.batch_processing().to_string(),
            
            OperationType::AddSubtitle | OperationType::AddWatermark | 
            OperationType::FrameExtract | OperationType::VideoToGif | 
            OperationType::GifResize => translations.advanced_features().to_string(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoSettings {
    pub codec: String,
    pub bitrate: String,
    pub quality: i32,
    pub fps: String,
    pub resolution: (u32, u32),
    pub use_hardware_acceleration: bool,
    pub preset: String,            // Encoding preset (ultrafast, fast, medium, slow)
    pub profile: String,           // Encoding profile (baseline, main, high)
    pub level: String,             // Encoding level (3.0, 3.1, 4.0, etc.)
    pub pixel_format: String,      // Pixel format (yuv420p, yuv444p, etc.)
    pub tune: String,              // Tuning setting (film, animation, grain)
    pub custom_args: String,       // User custom parameters
    
    // Format conversion
    pub container_format: String,
    pub copy_video: bool,
    
    // Compression
    pub crf: i32,
    pub target_size_mb: i32,
    
    // Resolution
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maintain_aspect_ratio: bool,
    
    // Crop
    pub crop_top: Option<u32>,
    pub crop_bottom: Option<u32>,
    pub crop_left: Option<u32>,
    pub crop_right: Option<u32>,
    
    // Rotation
    pub rotation: i32,
    pub use_custom_rotation: bool,
    pub custom_rotation_angle: f32,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    
    // Filters
    pub denoise: bool,
    pub deinterlace: bool,
    pub stabilize: bool,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    
    // Batch processing
    pub batch_naming_pattern: String,
    pub batch_operation_type: String,
    
    // Subtitle settings
    pub subtitle_file: String,
    pub subtitle_mode: String,  // "hard" or "soft"
    pub subtitle_style: String, // For styling options
    pub subtitle_position: String, // Position of subtitles
    
    // Watermark settings  
    pub watermark_file: String,
    pub watermark_position: String, // "top-left", "top-right", "bottom-left", "bottom-right", "center"
    pub watermark_opacity: f32,
    pub watermark_scale: f32,
    pub watermark_x: i32,       // X coordinate for watermark
    pub watermark_y: i32,       // Y coordinate for watermark
    
    // Frame extraction settings
    pub frame_extract_mode: String, // "all", "interval", "time"
    pub frame_interval: i32,        // Extract every N frames
    pub frame_start_time: String,   // Start time for extraction
    pub frame_end_time: String,     // End time for extraction  
    pub frame_format: String,       // "png", "jpg", "bmp"
    pub frame_quality: i32,         // JPEG quality (1-31, lower is better)
    pub frame_rate: i32,            // Frame rate for extraction
    
    // Subtitle style settings
    pub subtitle_font_family: String,
    pub subtitle_font_size: i32,
    pub subtitle_font_color: String,
    pub subtitle_outline_color: String,
    pub subtitle_background_color: String,
    pub subtitle_alignment: String,
    
    // GIF conversion settings
    pub gif_fps: f32,
    pub gif_scale: f32,
    pub gif_loop: bool,
    pub gif_optimize: bool,
    
    // Smart encoder preferences
    pub quality_preset: String,    // "High Quality", "Balanced", "Fast"  
    pub speed_priority: bool,      // Prioritize speed over quality
    pub gif_dither: String,  // "none", "bayer", "floyd_steinberg"
    pub gif_colors: i32,     // Number of colors (2-256)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AudioSettings {
    pub codec: String,
    pub bitrate: String,
    pub sample_rate: String,
    pub channels: String,
    pub volume: f32,
    pub quality: String,           // Quality setting (e.g. VBR quality)
    pub custom_args: String,       // User custom parameters
    
    // Format conversion
    pub format: String,
    pub copy_audio: bool,
    
    // Compression
    pub vbr_quality: i32,
    
    // Resampling
    pub resample_method: String,
    
    // Volume
    pub normalize: bool,
    pub target_lufs: f32,
    
    // Trim
    pub start_time: String,
    pub end_time: String,
    pub fade_in: bool,
    pub fade_out: bool,
    
    // Merge
    pub merge_mode: String,
    pub add_silence: bool,
    pub silence_duration: f32,
    
    // Video/Audio merge
    pub sync_audio: bool,
    pub audio_delay: f32,
    
    // Extract
    pub extract_all_tracks: bool,
}

#[derive(Clone, Debug)]
pub struct ProcessingTask {
    pub id: usize,
    pub operation: OperationType,
    pub input_files: Vec<String>,
    pub output_file: String,
    pub video_settings: Option<VideoSettings>,
    pub audio_settings: Option<AudioSettings>,
    pub progress: f32,
    pub status: TaskStatus,
    pub error_message: Option<String>,
    pub start_time: Option<std::time::Instant>,
    pub estimated_total_time: Option<std::time::Duration>,
    pub completion_time: Option<std::time::Duration>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ProcessingTask {
    pub fn new(operation: OperationType, input_files: Vec<String>, output_file: String) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static TASK_COUNTER: AtomicUsize = AtomicUsize::new(1);
        
        Self {
            id: TASK_COUNTER.fetch_add(1, Ordering::Relaxed),
            operation,
            input_files,
            output_file,
            video_settings: None,
            audio_settings: None,
            progress: 0.0,
            status: TaskStatus::Pending,
            error_message: None,
            start_time: None,
            estimated_total_time: None,
            completion_time: None,
        }
    }
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            codec: "auto".to_string(),
            bitrate: "auto".to_string(),
            quality: 23,
            fps: "auto".to_string(),
            resolution: (0, 0), // 0,0 means keep original
            use_hardware_acceleration: false,
            preset: "auto".to_string(),
            profile: "auto".to_string(),
            level: "auto".to_string(),
            pixel_format: "auto".to_string(),
            tune: "auto".to_string(),
            custom_args: String::new(),
            
            // Format conversion
            container_format: "mp4".to_string(),
            copy_video: false,
            
            // Compression
            crf: 23,
            target_size_mb: 0,
            
            // Resolution
            width: None,
            height: None,
            maintain_aspect_ratio: true,
            
            // Crop
            crop_top: None,
            crop_bottom: None,
            crop_left: None,
            crop_right: None,
            
            // Rotation
            rotation: 0,
            use_custom_rotation: false,
            custom_rotation_angle: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            
            // Filters
            denoise: false,
            deinterlace: false,
            stabilize: false,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            
            // Batch processing
            batch_naming_pattern: "{name}_converted".to_string(),
            batch_operation_type: "convert".to_string(),
            
            // Subtitle settings
            subtitle_file: String::new(),
            subtitle_mode: "soft".to_string(),
            subtitle_style: String::new(),
            subtitle_position: "bottom-center".to_string(),
            
            // Watermark settings
            watermark_file: String::new(),
            watermark_position: "top-right".to_string(),
            watermark_opacity: 0.7,
            watermark_scale: 1.0,
            watermark_x: 10,
            watermark_y: 10,
            
            // Frame extraction settings
            frame_extract_mode: "interval".to_string(),
            frame_interval: 30,
            frame_start_time: "00:00:00".to_string(),
            frame_end_time: String::new(),
            frame_format: "png".to_string(),
            frame_quality: 2,
            frame_rate: 1,
            
            // Subtitle style settings
            subtitle_font_family: "Arial".to_string(),
            subtitle_font_size: 16,
            subtitle_font_color: "white".to_string(),
            subtitle_outline_color: "black".to_string(),
            subtitle_background_color: "transparent".to_string(),
            subtitle_alignment: "center".to_string(),
            
            // GIF conversion settings
            gif_fps: 10.0,
            gif_scale: 1.0,
            gif_loop: true,
            gif_optimize: true,
            gif_dither: "floyd_steinberg".to_string(),
            gif_colors: 256,
            
            // Smart encoder preferences
            quality_preset: "Balanced".to_string(),
            speed_priority: false,
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            codec: "auto".to_string(),
            bitrate: "auto".to_string(),
            sample_rate: "44100".to_string(),
            channels: "auto".to_string(),
            volume: 1.0,
            quality: "auto".to_string(),
            custom_args: String::new(),
            
            // Format conversion
            format: "mp3".to_string(),
            copy_audio: false,
            
            // Compression
            vbr_quality: 2,
            
            // Resampling
            resample_method: "swr".to_string(),
            
            // Volume
            normalize: false,
            target_lufs: -16.0,
            
            // Trim
            start_time: "00:00:00".to_string(),
            end_time: String::new(),
            fade_in: false,
            fade_out: false,
            
            // Merge
            merge_mode: "concat".to_string(),
            add_silence: false,
            silence_duration: 1.0,
            
            // Video/Audio merge
            sync_audio: false,
            audio_delay: 0.0,
            
            // Extract
            extract_all_tracks: false,
        }
    }
}

// Project configuration structure for .ffcfg files
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectConfig {
    pub version: String,
    pub project_name: String,
    pub current_operation: Option<OperationType>,
    pub input_files: Vec<String>,
    pub output_file: String,
    pub video_settings: VideoSettings,
    pub audio_settings: AudioSettings,
    pub created_at: String,
    pub modified_at: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        Self {
            version: "1.0.0".to_string(),
            project_name: "Untitled Project".to_string(),
            current_operation: None,
            input_files: Vec::new(),
            output_file: String::new(),
            video_settings: VideoSettings::default(),
            audio_settings: AudioSettings::default(),
            created_at: now.clone(),
            modified_at: now,
        }
    }
}

impl ProjectConfig {
    pub fn from_app_state(
        current_operation: Option<OperationType>,
        input_files: Vec<String>,
        output_file: String,
        video_settings: VideoSettings,
        audio_settings: AudioSettings,
        project_name: Option<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        Self {
            version: "1.0.0".to_string(),
            project_name: project_name.unwrap_or_else(|| "Untitled Project".to_string()),
            current_operation,
            input_files,
            output_file,
            video_settings,
            audio_settings,
            created_at: now.clone(),
            modified_at: now,
        }
    }
    
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(file_path, json)?;
        Ok(())
    }
    
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(file_path)?;
        
        // Try to load with full compatibility first
        match serde_json::from_str::<ProjectConfig>(&json) {
            Ok(config) => Ok(config),
            Err(_) => {
                // If direct loading fails, try loading with partial data and fill defaults
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(mut value) => {
                        // Create a default config and merge existing data
                        let mut default_config = ProjectConfig::default();
                        
                        // Extract basic fields if they exist
                        if let Some(version) = value.get("version").and_then(|v| v.as_str()) {
                            default_config.version = version.to_string();
                        }
                        if let Some(project_name) = value.get("project_name").and_then(|v| v.as_str()) {
                            default_config.project_name = project_name.to_string();
                        }
                        if let Some(output_file) = value.get("output_file").and_then(|v| v.as_str()) {
                            default_config.output_file = output_file.to_string();
                        }
                        if let Some(input_files) = value.get("input_files").and_then(|v| v.as_array()) {
                            default_config.input_files = input_files.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                        }
                        
                        // Try to merge video_settings
                        if let Some(video_settings) = value.get_mut("video_settings") {
                            if let Ok(settings) = serde_json::from_value::<VideoSettings>(video_settings.clone()) {
                                default_config.video_settings = settings;
                            } else {
                                // Merge what we can from video settings
                                Self::merge_video_settings(&mut default_config.video_settings, video_settings);
                            }
                        }
                        
                        // Try to merge audio_settings
                        if let Some(audio_settings) = value.get_mut("audio_settings") {
                            if let Ok(settings) = serde_json::from_value::<AudioSettings>(audio_settings.clone()) {
                                default_config.audio_settings = settings;
                            } else {
                                // Merge what we can from audio settings
                                Self::merge_audio_settings(&mut default_config.audio_settings, audio_settings);
                            }
                        }
                        
                        Ok(default_config)
                    }
                    Err(e) => Err(Box::new(e))
                }
            }
        }
    }
    
    fn merge_video_settings(default: &mut VideoSettings, value: &serde_json::Value) {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                match key.as_str() {
                    "codec" => if let Some(s) = val.as_str() { default.codec = s.to_string(); },
                    "bitrate" => if let Some(s) = val.as_str() { default.bitrate = s.to_string(); },
                    "quality" => if let Some(i) = val.as_i64() { default.quality = i as i32; },
                    "fps" => if let Some(s) = val.as_str() { default.fps = s.to_string(); },
                    "use_hardware_acceleration" => if let Some(b) = val.as_bool() { default.use_hardware_acceleration = b; },
                    "preset" => if let Some(s) = val.as_str() { default.preset = s.to_string(); },
                    "profile" => if let Some(s) = val.as_str() { default.profile = s.to_string(); },
                    "tune" => if let Some(s) = val.as_str() { default.tune = s.to_string(); },
                    "container_format" => if let Some(s) = val.as_str() { default.container_format = s.to_string(); },
                    "crf" => if let Some(i) = val.as_i64() { default.crf = i as i32; },
                    "target_size_mb" => if let Some(i) = val.as_i64() { default.target_size_mb = i as i32; },
                    "rotation" => if let Some(i) = val.as_i64() { default.rotation = i as i32; },
                    "use_custom_rotation" => if let Some(b) = val.as_bool() { default.use_custom_rotation = b; },
                    "custom_rotation_angle" => if let Some(f) = val.as_f64() { default.custom_rotation_angle = f as f32; },
                    "batch_operation_type" => if let Some(s) = val.as_str() { default.batch_operation_type = s.to_string(); },
                    _ => {} // Ignore unknown fields for forward compatibility
                }
            }
        }
    }
    
    fn merge_audio_settings(default: &mut AudioSettings, value: &serde_json::Value) {
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                match key.as_str() {
                    "codec" => if let Some(s) = val.as_str() { default.codec = s.to_string(); },
                    "bitrate" => if let Some(s) = val.as_str() { default.bitrate = s.to_string(); },
                    "sample_rate" => if let Some(s) = val.as_str() { default.sample_rate = s.to_string(); },
                    "channels" => if let Some(s) = val.as_str() { default.channels = s.to_string(); },
                    "volume" => if let Some(f) = val.as_f64() { default.volume = f as f32; },
                    "quality" => if let Some(s) = val.as_str() { default.quality = s.to_string(); },
                    "format" => if let Some(s) = val.as_str() { default.format = s.to_string(); },
                    "vbr_quality" => if let Some(i) = val.as_i64() { default.vbr_quality = i as i32; },
                    _ => {} // Ignore unknown fields for forward compatibility
                }
            }
        }
    }
}