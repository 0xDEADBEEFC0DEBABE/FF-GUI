#[derive(Clone, Debug, PartialEq)]
pub enum Language {
    Chinese,
    English,
}

impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "中文",
            Language::English => "English",
        }
    }

    pub fn all_languages() -> Vec<Language> {
        vec![Language::Chinese, Language::English]
    }
}

#[derive(Clone)]
pub struct Translations {
    pub language: Language,
}

impl Translations {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    // Menu bar
    pub fn menu_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "文件",
            Language::English => "File",
        }
    }

    pub fn menu_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "设置",
            Language::English => "Settings",
        }
    }

    pub fn menu_language(&self) -> &'static str {
        match self.language {
            Language::Chinese => "语言",
            Language::English => "Language",
        }
    }

    pub fn menu_help(&self) -> &'static str {
        match self.language {
            Language::Chinese => "帮助",
            Language::English => "Help",
        }
    }

    pub fn new_project(&self) -> &'static str {
        match self.language {
            Language::Chinese => "新建项目",
            Language::English => "New Project",
        }
    }

    pub fn save_project(&self) -> &'static str {
        match self.language {
            Language::Chinese => "保存项目",
            Language::English => "Save Project",
        }
    }

    pub fn load_project(&self) -> &'static str {
        match self.language {
            Language::Chinese => "加载项目",
            Language::English => "Load Project",
        }
    }

    pub fn exit(&self) -> &'static str {
        match self.language {
            Language::Chinese => "退出",
            Language::English => "Exit",
        }
    }

    pub fn show_advanced_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "显示高级设置",
            Language::English => "Show Advanced Settings",
        }
    }

    pub fn reset_all_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "重置所有设置",
            Language::English => "Reset All Settings",
        }
    }

    pub fn toggle_dark_mode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "切换深色模式",
            Language::English => "Toggle Dark Mode",
        }
    }

    pub fn dark_mode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "深色模式",
            Language::English => "Dark Mode",
        }
    }

    pub fn light_mode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "浅色模式",
            Language::English => "Light Mode",
        }
    }

    pub fn about(&self) -> &'static str {
        match self.language {
            Language::Chinese => "关于",
            Language::English => "About",
        }
    }

    // Status bar
    pub fn active_tasks(&self) -> &'static str {
        match self.language {
            Language::Chinese => "活跃任务",
            Language::English => "Active Tasks",
        }
    }

    pub fn total_tasks(&self) -> &'static str {
        match self.language {
            Language::Chinese => "总任务",
            Language::English => "Total Tasks",
        }
    }

    pub fn clear_completed_tasks(&self) -> &'static str {
        match self.language {
            Language::Chinese => "清空已完成任务",
            Language::English => "Clear Completed Tasks",
        }
    }

    // Operation selection
    pub fn select_operation_type(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择操作类型",
            Language::English => "Select Operation Type",
        }
    }

    pub fn video_processing(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频处理",
            Language::English => "Video Processing",
        }
    }

    pub fn audio_processing(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频处理",
            Language::English => "Audio Processing",
        }
    }

    pub fn video_audio_operations(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音视频操作",
            Language::English => "Video/Audio Operations",
        }
    }

    pub fn batch_processing(&self) -> &'static str {
        match self.language {
            Language::Chinese => "批量处理",
            Language::English => "Batch Processing",
        }
    }

    pub fn advanced_features(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高级功能",
            Language::English => "Advanced Features",
        }
    }

    // Operation types
    pub fn video_convert(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频格式转换",
            Language::English => "Video Format Conversion",
        }
    }

    pub fn video_compress(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频压缩",
            Language::English => "Video Compression",
        }
    }

    pub fn video_resize(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频缩放",
            Language::English => "Video Resize",
        }
    }

    pub fn video_crop(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频裁剪",
            Language::English => "Video Crop",
        }
    }

    pub fn video_rotate(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频旋转",
            Language::English => "Video Rotate",
        }
    }

    pub fn video_filter(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频滤镜",
            Language::English => "Video Filter",
        }
    }

    pub fn audio_convert(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频格式转换",
            Language::English => "Audio Format Conversion",
        }
    }

    pub fn audio_compress(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频压缩",
            Language::English => "Audio Compression",
        }
    }

    pub fn audio_resample(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频重采样",
            Language::English => "Audio Resample",
        }
    }

    pub fn audio_volume(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音量调整",
            Language::English => "Volume Adjustment",
        }
    }

    pub fn audio_trim(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频裁剪",
            Language::English => "Audio Trim",
        }
    }

    pub fn audio_merge(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频合并",
            Language::English => "Audio Merge",
        }
    }

    pub fn video_audio_merge(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音视频合成",
            Language::English => "Video/Audio Merge",
        }
    }

    pub fn video_audio_split(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音视频分离",
            Language::English => "Video/Audio Split",
        }
    }

    pub fn extract_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "提取音频",
            Language::English => "Extract Audio",
        }
    }

    pub fn extract_video(&self) -> &'static str {
        match self.language {
            Language::Chinese => "提取视频",
            Language::English => "Extract Video",
        }
    }

    pub fn batch_convert(&self) -> &'static str {
        match self.language {
            Language::Chinese => "批量转换",
            Language::English => "Batch Convert",
        }
    }

    pub fn add_subtitle(&self) -> &'static str {
        match self.language {
            Language::Chinese => "添加字幕",
            Language::English => "Add Subtitle",
        }
    }

    pub fn add_watermark(&self) -> &'static str {
        match self.language {
            Language::Chinese => "添加水印",
            Language::English => "Add Watermark",
        }
    }

    pub fn frame_extract(&self) -> &'static str {
        match self.language {
            Language::Chinese => "提取帧",
            Language::English => "Extract Frames",
        }
    }

    // File operations
    pub fn input_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输入文件",
            Language::English => "Input File",
        }
    }

    pub fn output_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输出文件:",
            Language::English => "Output File:",
        }
    }

    pub fn select_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择文件",
            Language::English => "Select File",
        }
    }

    pub fn select_location(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择位置",
            Language::English => "Select Location",
        }
    }

    pub fn clear(&self) -> &'static str {
        match self.language {
            Language::Chinese => "清空",
            Language::English => "Clear",
        }
    }

    pub fn drop_file_here(&self) -> &'static str {
        match self.language {
            Language::Chinese => "拖放文件到这里",
            Language::English => "Drop file here",
        }
    }

    // Settings
    pub fn video_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频设置",
            Language::English => "Video Settings",
        }
    }

    pub fn audio_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频设置",
            Language::English => "Audio Settings",
        }
    }

    pub fn codec(&self) -> &'static str {
        match self.language {
            Language::Chinese => "编码器",
            Language::English => "Codec",
        }
    }

    pub fn encoder(&self) -> &'static str {
        match self.language {
            Language::Chinese => "编码器",
            Language::English => "Encoder",
        }
    }

    pub fn preset(&self) -> &'static str {
        match self.language {
            Language::Chinese => "预设",
            Language::English => "Preset",
        }
    }

    pub fn profile(&self) -> &'static str {
        match self.language {
            Language::Chinese => "配置文件",
            Language::English => "Profile",
        }
    }

    pub fn tune(&self) -> &'static str {
        match self.language {
            Language::Chinese => "调优",
            Language::English => "Tune",
        }
    }

    pub fn quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "质量 (CRF)",
            Language::English => "Quality (CRF)",
        }
    }

    pub fn bitrate(&self) -> &'static str {
        match self.language {
            Language::Chinese => "码率",
            Language::English => "Bitrate",
        }
    }

    pub fn framerate(&self) -> &'static str {
        match self.language {
            Language::Chinese => "帧率",
            Language::English => "Frame Rate",
        }
    }

    pub fn resolution(&self) -> &'static str {
        match self.language {
            Language::Chinese => "分辨率",
            Language::English => "Resolution",
        }
    }

    pub fn use_hardware_acceleration(&self) -> &'static str {
        match self.language {
            Language::Chinese => "使用硬件加速",
            Language::English => "Use Hardware Acceleration",
        }
    }

    pub fn sample_rate(&self) -> &'static str {
        match self.language {
            Language::Chinese => "采样率",
            Language::English => "Sample Rate",
        }
    }

    pub fn channels(&self) -> &'static str {
        match self.language {
            Language::Chinese => "声道",
            Language::English => "Channels",
        }
    }

    pub fn volume(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音量",
            Language::English => "Volume",
        }
    }

    // Buttons
    pub fn start_processing(&self) -> &'static str {
        match self.language {
            Language::Chinese => "开始处理",
            Language::English => "Start Processing",
        }
    }

    pub fn preview_command(&self) -> &'static str {
        match self.language {
            Language::Chinese => "预览命令",
            Language::English => "Preview Command",
        }
    }

    pub fn reset_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "重置设置",
            Language::English => "Reset Settings",
        }
    }

    // Status
    pub fn ready(&self) -> &'static str {
        match self.language {
            Language::Chinese => "就绪",
            Language::English => "Ready",
        }
    }

    pub fn project_reset(&self) -> &'static str {
        match self.language {
            Language::Chinese => "项目已重置",
            Language::English => "Project Reset",
        }
    }

    // Welcome interface
    pub fn welcome_title(&self) -> &'static str {
        match self.language {
            Language::Chinese => "欢迎使用 FF GUI",
            Language::English => "Welcome to FF GUI",
        }
    }

    pub fn welcome_instruction(&self) -> &'static str {
        match self.language {
            Language::Chinese => "请从左侧选择要执行的操作类型",
            Language::English => "Please select an operation type from the left panel",
        }
    }

    pub fn quick_start(&self) -> &'static str {
        match self.language {
            Language::Chinese => "快速开始:",
            Language::English => "Quick Start:",
        }
    }

    pub fn step1(&self) -> &'static str {
        match self.language {
            Language::Chinese => "• 选择操作类型",
            Language::English => "• Select operation type",
        }
    }

    pub fn step2(&self) -> &'static str {
        match self.language {
            Language::Chinese => "• 添加输入文件",
            Language::English => "• Add input files",
        }
    }

    pub fn step3(&self) -> &'static str {
        match self.language {
            Language::Chinese => "• 配置输出设置",
            Language::English => "• Configure output settings",
        }
    }

    pub fn step4(&self) -> &'static str {
        match self.language {
            Language::Chinese => "• 开始处理",
            Language::English => "• Start processing",
        }
    }

    // Task related
    pub fn task_list(&self) -> &'static str {
        match self.language {
            Language::Chinese => "任务列表",
            Language::English => "Task List",
        }
    }

    pub fn no_tasks(&self) -> &'static str {
        match self.language {
            Language::Chinese => "暂无任务",
            Language::English => "No tasks",
        }
    }

    pub fn task(&self) -> &'static str {
        match self.language {
            Language::Chinese => "任务",
            Language::English => "Task",
        }
    }

    pub fn status(&self) -> &'static str {
        match self.language {
            Language::Chinese => "状态",
            Language::English => "Status",
        }
    }

    pub fn error(&self) -> &'static str {
        match self.language {
            Language::Chinese => "错误",
            Language::English => "Error",
        }
    }

    pub fn cancel(&self) -> &'static str {
        match self.language {
            Language::Chinese => "取消",
            Language::English => "Cancel",
        }
    }

    pub fn delete(&self) -> &'static str {
        match self.language {
            Language::Chinese => "删除",
            Language::English => "Delete",
        }
    }

    pub fn file_info(&self) -> &'static str {
        match self.language {
            Language::Chinese => "文件信息",
            Language::English => "File Information",
        }
    }

    pub fn current_operation(&self, operation_name: &str) -> String {
        match self.language {
            Language::Chinese => format!("当前操作: {}", operation_name),
            Language::English => format!("Current Operation: {}", operation_name),
        }
    }

    pub fn task_added(&self, operation_name: &str) -> String {
        match self.language {
            Language::Chinese => format!("已添加任务: {}", operation_name),
            Language::English => format!("Task added: {}", operation_name),
        }
    }

    // New translation items
    pub fn smart_recommendations(&self) -> &'static str {
        match self.language {
            Language::Chinese => "智能推荐",
            Language::English => "Smart Recommendations",
        }
    }

    pub fn encoding_presets(&self) -> &'static str {
        match self.language {
            Language::Chinese => "编码预设",
            Language::English => "Encoding Presets",
        }
    }

    pub fn ffmpeg_command_preview(&self) -> &'static str {
        match self.language {
            Language::Chinese => "FFmpeg命令预览",
            Language::English => "FFmpeg Command Preview",
        }
    }

    pub fn copy_command(&self) -> &'static str {
        match self.language {
            Language::Chinese => "复制命令",
            Language::English => "Copy Command",
        }
    }

    pub fn command_generated(&self) -> String {
        match self.language {
            Language::Chinese => "FFmpeg命令已生成，请查看预览区域".to_string(),
            Language::English => "FFmpeg command generated, please check preview area".to_string(),
        }
    }

    pub fn video_file_required(&self) -> &'static str {
        match self.language {
            Language::Chinese => "请选择视频文件",
            Language::English => "Please select a video file",
        }
    }

    pub fn subtitle_file_required(&self) -> &'static str {
        match self.language {
            Language::Chinese => "请选择字幕文件",
            Language::English => "Please select a subtitle file",
        }
    }

    pub fn missing_files_for_subtitle(&self) -> &'static str {
        match self.language {
            Language::Chinese => "添加字幕需要视频文件和字幕文件",
            Language::English => "Adding subtitles requires both video and subtitle files",
        }
    }

    pub fn video_selected_subtitle_reminder(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频文件已选择。别忘了在下面选择字幕文件。",
            Language::English => "Video file selected. Don't forget to select a subtitle file below.",
        }
    }

    pub fn subtitle_file_ready(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字幕文件准备就绪",
            Language::English => "Subtitle file ready",
        }
    }

    pub fn watermark_file_required(&self) -> &'static str {
        match self.language {
            Language::Chinese => "请选择水印图片文件",
            Language::English => "Please select a watermark image file",
        }
    }

    pub fn watermark_image_ready(&self) -> &'static str {
        match self.language {
            Language::Chinese => "水印图片准备就绪",
            Language::English => "Watermark image ready",
        }
    }

    pub fn command_generation_error(&self) -> &'static str {
        match self.language {
            Language::Chinese => "生成命令时出错",
            Language::English => "Error generating command",
        }
    }

    pub fn command_generation_failed(&self) -> String {
        match self.language {
            Language::Chinese => "命令生成失败".to_string(),
            Language::English => "Command generation failed".to_string(),
        }
    }

    // About dialog translations
    pub fn author(&self) -> &'static str {
        match self.language {
            Language::Chinese => "作者",
            Language::English => "Author",
        }
    }

    pub fn license(&self) -> &'static str {
        match self.language {
            Language::Chinese => "许可证",
            Language::English => "License",
        }
    }

    pub fn built_with_ffmpeg(&self) -> &'static str {
        match self.language {
            Language::Chinese => "基于 FFmpeg 构建",
            Language::English => "Built with FFmpeg",
        }
    }

    pub fn developed_with_rust(&self) -> &'static str {
        match self.language {
            Language::Chinese => "使用 Rust + egui 开发",
            Language::English => "Developed with Rust + egui",
        }
    }

    pub fn close(&self) -> &'static str {
        match self.language {
            Language::Chinese => "关闭",
            Language::English => "Close",
        }
    }

    // File selection labels
    pub fn input_files_video_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输入文件 (视频和音频)",
            Language::English => "Input Files (Video and Audio)",
        }
    }

    pub fn input_audio_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输入音频文件",
            Language::English => "Input Audio Files",
        }
    }

    pub fn batch_input_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "批量输入文件",
            Language::English => "Batch Input Files",
        }
    }

    pub fn input_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输入文件",
            Language::English => "Input Files",
        }
    }

    // Additional UI translations for ui_components.rs
    pub fn select_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择文件",
            Language::English => "Select Files",
        }
    }

    pub fn drag_drop_hint(&self) -> &'static str {
        match self.language {
            Language::Chinese => "拖放文件到此处或点击选择文件",
            Language::English => "Drag files here or click to select",
        }
    }

    pub fn selected_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "已选择",
            Language::English => "Selected",
        }
    }

    pub fn files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "个文件",
            Language::English => "files",
        }
    }

    pub fn all_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "所有文件",
            Language::English => "All Files",
        }
    }

    pub fn auto_select(&self) -> &'static str {
        match self.language {
            Language::Chinese => "自动选择",
            Language::English => "Auto Select",
        }
    }

    pub fn lossless_encoding(&self) -> &'static str {
        match self.language {
            Language::Chinese => "无损编码 (不支持码率设置)",
            Language::English => "Lossless encoding (bitrate not applicable)",
        }
    }

    pub fn vbr_not_supported(&self) -> &'static str {
        match self.language {
            Language::Chinese => "当前编码器不支持VBR",
            Language::English => "Current codec doesn't support VBR",
        }
    }

    pub fn custom_parameters(&self) -> &'static str {
        match self.language {
            Language::Chinese => "自定义参数",
            Language::English => "Custom Parameters",
        }
    }

    pub fn custom(&self) -> &'static str {
        match self.language {
            Language::Chinese => "自定义:",
            Language::English => "Custom:",
        }
    }

    pub fn vbr_quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "VBR质量",
            Language::English => "VBR Quality",
        }
    }

    pub fn width_height(&self) -> &'static str {
        match self.language {
            Language::Chinese => "宽高",
            Language::English => "Width/Height",
        }
    }

    pub fn maintain_aspect_ratio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "保持宽高比",
            Language::English => "Maintain Aspect Ratio",
        }
    }

    pub fn keep_original_quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "保持原始质量",
            Language::English => "Keep Original Quality",
        }
    }

    pub fn compression_quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "压缩质量",
            Language::English => "Compression Quality",
        }
    }

    pub fn high_quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高质量",
            Language::English => "High Quality",
        }
    }

    pub fn large_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "大文件",
            Language::English => "Large File",
        }
    }

    pub fn balanced(&self) -> &'static str {
        match self.language {
            Language::Chinese => "平衡",
            Language::English => "Balanced",
        }
    }

    pub fn high_compression(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高压缩",
            Language::English => "High Compression",
        }
    }

    pub fn small_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "小文件",
            Language::English => "Small File",
        }
    }

    pub fn width(&self) -> &'static str {
        match self.language {
            Language::Chinese => "宽:",
            Language::English => "Width:",
        }
    }

    pub fn custom_width(&self) -> &'static str {
        match self.language {
            Language::Chinese => "自定义宽度:",
            Language::English => "Custom Width:",
        }
    }

    pub fn height(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高:",
            Language::English => "Height:",
        }
    }

    pub fn keep_original_hint(&self) -> &'static str {
        match self.language {
            Language::Chinese => "(0表示保持原始)",
            Language::English => "(0 means keep original)",
        }
    }

    pub fn window_title(&self) -> &'static str {
        match self.language {
            Language::Chinese => "FF GUI - 全功能音视频处理工具",
            Language::English => "FF GUI - Full-Featured Audio/Video Processing Tool",
        }
    }

    // Subtitle related translations
    pub fn subtitle_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字幕文件:",
            Language::English => "Subtitle File:",
        }
    }

    pub fn subtitle_mode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字幕模式:",
            Language::English => "Subtitle Mode:",
        }
    }

    pub fn hard_subtitle(&self) -> &'static str {
        match self.language {
            Language::Chinese => "硬字幕 (嵌入视频)",
            Language::English => "Hard Subtitle (Embedded)",
        }
    }

    pub fn soft_subtitle(&self) -> &'static str {
        match self.language {
            Language::Chinese => "软字幕 (外挂)",
            Language::English => "Soft Subtitle (External)",
        }
    }

    // Watermark related translations
    pub fn watermark_file(&self) -> &'static str {
        match self.language {
            Language::Chinese => "水印图片:",
            Language::English => "Watermark Image:",
        }
    }

    pub fn position(&self) -> &'static str {
        match self.language {
            Language::Chinese => "位置:",
            Language::English => "Position:",
        }
    }

    pub fn opacity(&self) -> &'static str {
        match self.language {
            Language::Chinese => "透明度:",
            Language::English => "Opacity:",
        }
    }

    pub fn scale(&self) -> &'static str {
        match self.language {
            Language::Chinese => "缩放:",
            Language::English => "Scale:",
        }
    }

    // Frame extraction translations
    pub fn extract_mode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "提取模式:",
            Language::English => "Extract Mode:",
        }
    }

    pub fn all_frames(&self) -> &'static str {
        match self.language {
            Language::Chinese => "所有帧",
            Language::English => "All Frames",
        }
    }

    pub fn every_n_frames(&self) -> &'static str {
        match self.language {
            Language::Chinese => "每N帧",
            Language::English => "Every N Frames",
        }
    }

    pub fn time_range(&self) -> &'static str {
        match self.language {
            Language::Chinese => "时间范围",
            Language::English => "Time Range",
        }
    }

    pub fn frame_interval(&self) -> &'static str {
        match self.language {
            Language::Chinese => "帧间隔:",
            Language::English => "Frame Interval:",
        }
    }

    pub fn start_time(&self) -> &'static str {
        match self.language {
            Language::Chinese => "开始时间:",
            Language::English => "Start Time:",
        }
    }

    pub fn end_time(&self) -> &'static str {
        match self.language {
            Language::Chinese => "结束时间:",
            Language::English => "End Time:",
        }
    }

    pub fn output_format(&self) -> &'static str {
        match self.language {
            Language::Chinese => "输出格式:",
            Language::English => "Output Format:",
        }
    }

    pub fn image_quality(&self) -> &'static str {
        match self.language {
            Language::Chinese => "图片质量:",
            Language::English => "Image Quality:",
        }
    }

    // Position options
    pub fn top_left(&self) -> &'static str {
        match self.language {
            Language::Chinese => "左上角",
            Language::English => "Top Left",
        }
    }

    pub fn top_right(&self) -> &'static str {
        match self.language {
            Language::Chinese => "右上角",
            Language::English => "Top Right",
        }
    }

    pub fn bottom_left(&self) -> &'static str {
        match self.language {
            Language::Chinese => "左下角",
            Language::English => "Bottom Left",
        }
    }

    pub fn bottom_right(&self) -> &'static str {
        match self.language {
            Language::Chinese => "右下角",
            Language::English => "Bottom Right",
        }
    }

    pub fn center(&self) -> &'static str {
        match self.language {
            Language::Chinese => "居中",
            Language::English => "Center",
        }
    }

    pub fn select_file_button(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择文件",
            Language::English => "Select File",
        }
    }

    pub fn advanced_users_hint(&self) -> &'static str {
        match self.language {
            Language::Chinese => "(高级用户可添加额外的FFmpeg参数)",
            Language::English => "(Advanced users can add extra FFmpeg parameters)",
        }
    }
    
    pub fn select_preset_hint(&self) -> &'static str {
        match self.language {
            Language::Chinese => "选择一个预设快速配置编码参数",
            Language::English => "Select a preset to quickly configure encoding parameters",
        }
    }
    
    pub fn save_current_as_preset(&self) -> &'static str {
        match self.language {
            Language::Chinese => "保存当前设置为预设",
            Language::English => "Save Current Settings as Preset",
        }
    }
    
    pub fn load_custom_preset(&self) -> &'static str {
        match self.language {
            Language::Chinese => "加载自定义预设",
            Language::English => "Load Custom Preset",
        }
    }
    
    pub fn format_specific_presets(&self) -> &'static str {
        match self.language {
            Language::Chinese => "格式特定预设:",
            Language::English => "Format-specific Presets:",
        }
    }
    
    
    // Preset names
    pub fn preset_auto(&self) -> &'static str {
        match self.language {
            Language::Chinese => "自动",
            Language::English => "Auto",
        }
    }
    
    pub fn preset_ultrafast(&self) -> &'static str {
        match self.language {
            Language::Chinese => "极快 (较低质量)",
            Language::English => "Ultrafast (Lower Quality)",
        }
    }
    
    pub fn preset_superfast(&self) -> &'static str {
        match self.language {
            Language::Chinese => "超快",
            Language::English => "Superfast",
        }
    }
    
    pub fn preset_veryfast(&self) -> &'static str {
        match self.language {
            Language::Chinese => "很快",
            Language::English => "Very Fast",
        }
    }
    
    pub fn preset_faster(&self) -> &'static str {
        match self.language {
            Language::Chinese => "更快",
            Language::English => "Faster",
        }
    }
    
    pub fn preset_fast(&self) -> &'static str {
        match self.language {
            Language::Chinese => "快",
            Language::English => "Fast",
        }
    }
    
    pub fn preset_medium(&self) -> &'static str {
        match self.language {
            Language::Chinese => "中等 (推荐)",
            Language::English => "Medium (Recommended)",
        }
    }
    
    pub fn preset_slow(&self) -> &'static str {
        match self.language {
            Language::Chinese => "慢 (更好质量)",
            Language::English => "Slow (Better Quality)",
        }
    }
    
    pub fn preset_slower(&self) -> &'static str {
        match self.language {
            Language::Chinese => "更慢",
            Language::English => "Slower",
        }
    }
    
    pub fn preset_veryslow(&self) -> &'static str {
        match self.language {
            Language::Chinese => "很慢 (最佳质量)",
            Language::English => "Very Slow (Best Quality)",
        }
    }
    
    // Profile names
    pub fn profile_baseline(&self) -> &'static str {
        match self.language {
            Language::Chinese => "Baseline (兼容性最佳)",
            Language::English => "Baseline (Best Compatibility)",
        }
    }
    
    pub fn profile_main(&self) -> &'static str {
        match self.language {
            Language::Chinese => "Main (平衡)",
            Language::English => "Main (Balanced)",
        }
    }
    
    pub fn profile_high(&self) -> &'static str {
        match self.language {
            Language::Chinese => "High (质量最佳)",
            Language::English => "High (Best Quality)",
        }
    }
    
    // Tune options
    pub fn tune_film(&self) -> &'static str {
        match self.language {
            Language::Chinese => "电影",
            Language::English => "Film",
        }
    }
    
    pub fn tune_animation(&self) -> &'static str {
        match self.language {
            Language::Chinese => "动画",
            Language::English => "Animation",
        }
    }
    
    pub fn tune_grain(&self) -> &'static str {
        match self.language {
            Language::Chinese => "颗粒感",
            Language::English => "Grain",
        }
    }
    
    pub fn tune_stillimage(&self) -> &'static str {
        match self.language {
            Language::Chinese => "静态图像",
            Language::English => "Still Image",
        }
    }
    
    pub fn tune_psnr(&self) -> &'static str {
        match self.language {
            Language::Chinese => "PSNR优化",
            Language::English => "PSNR Optimization",
        }
    }
    
    pub fn tune_ssim(&self) -> &'static str {
        match self.language {
            Language::Chinese => "SSIM优化",
            Language::English => "SSIM Optimization",
        }
    }
    
    pub fn tune_fastdecode(&self) -> &'static str {
        match self.language {
            Language::Chinese => "快速解码",
            Language::English => "Fast Decode",
        }
    }
    
    pub fn tune_zerolatency(&self) -> &'static str {
        match self.language {
            Language::Chinese => "零延迟",
            Language::English => "Zero Latency",
        }
    }
    
    // Codec names
    pub fn codec_auto(&self) -> &'static str {
        match self.language {
            Language::Chinese => "🎯 自动选择",
            Language::English => "🎯 Auto Select",
        }
    }
    
    pub fn codec_hardware(&self) -> &'static str {
        match self.language {
            Language::Chinese => "(硬件)",
            Language::English => "(Hardware)",
        }
    }
    
    pub fn keep_original(&self) -> &'static str {
        match self.language {
            Language::Chinese => "(0表示保持原始)",
            Language::English => "(0 means keep original)",
        }
    }
    
    pub fn sample_rate_44_1(&self) -> &'static str {
        match self.language {
            Language::Chinese => "44.1 kHz (标准)",
            Language::English => "44.1 kHz (Standard)",
        }
    }
    
    pub fn sample_rate_48(&self) -> &'static str {
        match self.language {
            Language::Chinese => "48 kHz (推荐)",
            Language::English => "48 kHz (Recommended)",
        }
    }
    
    pub fn sample_rate_96(&self) -> &'static str {
        match self.language {
            Language::Chinese => "💎 96 kHz (专业级)",
            Language::English => "💎 96 kHz (Professional)",
        }
    }
    
    pub fn high_quality_hint(&self) -> &'static str {
        match self.language {
            Language::Chinese => "(高质量)",
            Language::English => "(High Quality)",
        }
    }
    
    // Additional file type translations
    pub fn audio_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "音频文件",
            Language::English => "Audio Files",
        }
    }
    
    pub fn video_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频文件",
            Language::English => "Video Files",
        }
    }
    
    pub fn image_files(&self) -> &'static str {
        match self.language {
            Language::Chinese => "图片文件",
            Language::English => "Image Files",
        }
    }
    
    // Quality translations
    pub fn lossless(&self) -> &'static str {
        match self.language {
            Language::Chinese => "无损",
            Language::English => "Lossless",
        }
    }
    
    pub fn copy_stream(&self) -> &'static str {
        match self.language {
            Language::Chinese => "复制流",
            Language::English => "Copy Stream",
        }
    }
    
    pub fn no_reencoding(&self) -> &'static str {
        match self.language {
            Language::Chinese => "不重编码",
            Language::English => "No Re-encoding",
        }
    }
    
    pub fn recommended_settings_for(&self) -> &'static str {
        match self.language {
            Language::Chinese => "基于输出格式",
            Language::English => "Recommended settings for output format",
        }
    }
    
    pub fn recommended_settings_suffix(&self) -> &'static str {
        match self.language {
            Language::Chinese => "的推荐设置",
            Language::English => "",
        }
    }
    
    pub fn fast(&self) -> &'static str {
        match self.language {
            Language::Chinese => "快速",
            Language::English => "Fast",
        }
    }
    
    // Open source disclaimer
    pub fn open_source_disclaimer(&self) -> &'static str {
        match self.language {
            Language::Chinese => "免责声明",
            Language::English => "Disclaimer",
        }
    }
    
    pub fn disclaimer_text(&self) -> &'static str {
        match self.language {
            Language::Chinese => "本软件按\"原样\"提供，不提供任何明示或暗示的保证。作者不对使用此软件产生的任何损害负责。",
            Language::English => "This software is provided \"AS IS\" without warranty of any kind. The author is not responsible for any damages resulting from the use of this software.",
        }
    }
    
    pub fn license_notice(&self) -> &'static str {
        match self.language {
            Language::Chinese => "本软件基于GPL v3.0许可证发布。您可以自由地重新分发和修改它。",
            Language::English => "This software is released under the GPL v3.0 license. You are free to redistribute and modify it.",
        }
    }
    
    pub fn third_party_notice(&self) -> &'static str {
        match self.language {
            Language::Chinese => "本软件使用FFmpeg库，FFmpeg是其各自所有者的商标。",
            Language::English => "This software uses the FFmpeg library. FFmpeg is a trademark of its respective owners.",
        }
    }
    
    // Project management translations
    pub fn save_project_as(&self) -> &'static str {
        match self.language {
            Language::Chinese => "另存为...",
            Language::English => "Save As...",
        }
    }
    
    pub fn load_project_from(&self) -> &'static str {
        match self.language {
            Language::Chinese => "打开项目...",
            Language::English => "Open Project...",
        }
    }
    
    pub fn project_saved(&self) -> &'static str {
        match self.language {
            Language::Chinese => "项目已保存",
            Language::English => "Project saved",
        }
    }
    
    pub fn project_loaded(&self) -> &'static str {
        match self.language {
            Language::Chinese => "项目已加载",
            Language::English => "Project loaded",
        }
    }
    
    pub fn save_error(&self) -> &'static str {
        match self.language {
            Language::Chinese => "保存失败",
            Language::English => "Save failed",
        }
    }
    
    pub fn load_error(&self) -> &'static str {
        match self.language {
            Language::Chinese => "加载失败",
            Language::English => "Load failed",
        }
    }
    
    // New operation types
    pub fn video_to_gif(&self) -> &'static str {
        match self.language {
            Language::Chinese => "视频转GIF",
            Language::English => "Video to GIF",
        }
    }
    
    pub fn gif_resize(&self) -> &'static str {
        match self.language {
            Language::Chinese => "GIF尺寸调整",
            Language::English => "GIF Resize",
        }
    }
    
    // Subtitle style translations
    pub fn font_family(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字体:",
            Language::English => "Font:",
        }
    }
    
    pub fn font_size(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字体大小:",
            Language::English => "Font Size:",
        }
    }
    
    pub fn font_color(&self) -> &'static str {
        match self.language {
            Language::Chinese => "字体颜色:",
            Language::English => "Font Color:",
        }
    }
    
    pub fn outline_color(&self) -> &'static str {
        match self.language {
            Language::Chinese => "边框颜色:",
            Language::English => "Outline Color:",
        }
    }
    
    pub fn background_color(&self) -> &'static str {
        match self.language {
            Language::Chinese => "背景颜色:",
            Language::English => "Background Color:",
        }
    }
    
    pub fn alignment(&self) -> &'static str {
        match self.language {
            Language::Chinese => "对齐方式:",
            Language::English => "Alignment:",
        }
    }
    
    // GIF settings translations
    pub fn gif_settings(&self) -> &'static str {
        match self.language {
            Language::Chinese => "GIF设置",
            Language::English => "GIF Settings",
        }
    }
    
    pub fn gif_fps(&self) -> &'static str {
        match self.language {
            Language::Chinese => "帧率:",
            Language::English => "Frame Rate:",
        }
    }
    
    pub fn gif_scale(&self) -> &'static str {
        match self.language {
            Language::Chinese => "缩放比例:",
            Language::English => "Scale:",
        }
    }
    
    pub fn gif_loop(&self) -> &'static str {
        match self.language {
            Language::Chinese => "循环播放",
            Language::English => "Loop",
        }
    }
    
    pub fn gif_optimize(&self) -> &'static str {
        match self.language {
            Language::Chinese => "优化文件大小",
            Language::English => "Optimize Size",
        }
    }
    
    pub fn gif_dither(&self) -> &'static str {
        match self.language {
            Language::Chinese => "抖动算法:",
            Language::English => "Dithering:",
        }
    }
    
    pub fn gif_colors(&self) -> &'static str {
        match self.language {
            Language::Chinese => "颜色数量:",
            Language::English => "Color Count:",
        }
    }
    
    // Comprehensive codec and format translations
    pub fn codec_category_h264_family(&self) -> &'static str {
        match self.language {
            Language::Chinese => "H.264系列",
            Language::English => "H.264 Family",
        }
    }
    
    pub fn codec_category_h265_family(&self) -> &'static str {
        match self.language {
            Language::Chinese => "H.265系列",
            Language::English => "H.265 Family",
        }
    }
    
    pub fn codec_category_vp8vp9_family(&self) -> &'static str {
        match self.language {
            Language::Chinese => "VP8/VP9系列",
            Language::English => "VP8/VP9 Family",
        }
    }
    
    pub fn codec_category_av1_family(&self) -> &'static str {
        match self.language {
            Language::Chinese => "AV1系列",
            Language::English => "AV1 Family",
        }
    }
    
    pub fn codec_category_hardware_encoders(&self) -> &'static str {
        match self.language {
            Language::Chinese => "硬件编码器",
            Language::English => "Hardware Encoders",
        }
    }
    
    pub fn codec_category_legacy_video(&self) -> &'static str {
        match self.language {
            Language::Chinese => "传统视频编码器",
            Language::English => "Legacy Video Codecs",
        }
    }
    
    pub fn codec_category_high_quality_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高质量音频",
            Language::English => "High Quality Audio",
        }
    }
    
    pub fn codec_category_compressed_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "压缩音频",
            Language::English => "Compressed Audio",
        }
    }
    
    pub fn codec_category_lossless_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "无损音频",
            Language::English => "Lossless Audio",
        }
    }
    
    pub fn codec_category_speech_codecs(&self) -> &'static str {
        match self.language {
            Language::Chinese => "语音编码器",
            Language::English => "Speech Codecs",
        }
    }
    
    pub fn codec_category_legacy_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "传统音频编码器",
            Language::English => "Legacy Audio Codecs",
        }
    }
    
    pub fn format_category_modern_video(&self) -> &'static str {
        match self.language {
            Language::Chinese => "现代视频格式",
            Language::English => "Modern Video Formats",
        }
    }
    
    pub fn format_category_web_optimized(&self) -> &'static str {
        match self.language {
            Language::Chinese => "网络优化格式",
            Language::English => "Web Optimized Formats",
        }
    }
    
    pub fn format_category_professional(&self) -> &'static str {
        match self.language {
            Language::Chinese => "专业格式",
            Language::English => "Professional Formats",
        }
    }
    
    pub fn format_category_legacy_video(&self) -> &'static str {
        match self.language {
            Language::Chinese => "传统视频格式",
            Language::English => "Legacy Video Formats",
        }
    }
    
    pub fn format_category_high_quality_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "高质量音频格式",
            Language::English => "High Quality Audio Formats",
        }
    }
    
    pub fn format_category_compressed_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "压缩音频格式",
            Language::English => "Compressed Audio Formats",
        }
    }
    
    pub fn format_category_lossless_audio(&self) -> &'static str {
        match self.language {
            Language::Chinese => "无损音频格式",
            Language::English => "Lossless Audio Formats",
        }
    }
    
    pub fn format_category_specialized(&self) -> &'static str {
        match self.language {
            Language::Chinese => "专用格式",
            Language::English => "Specialized Formats",
        }
    }
    
    pub fn auto_recommended(&self) -> &'static str {
        match self.language {
            Language::Chinese => "🤖 自动 (推荐)",
            Language::English => "🤖 Auto (Recommended)",
        }
    }
    
    pub fn hardware_not_supported(&self) -> &'static str {
        match self.language {
            Language::Chinese => "硬件不支持",
            Language::English => "Hardware Not Supported",
        }
    }
    
    pub fn codec_not_compatible_with_format(&self) -> &'static str {
        match self.language {
            Language::Chinese => "编码器与格式不兼容",
            Language::English => "Codec not compatible with format",
        }
    }
}

// Detect system language
pub fn detect_system_language() -> Language {
    // Detect system language on Windows
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-Culture | Select-Object -ExpandProperty Name"])
            .output()
        {
            let locale = String::from_utf8_lossy(&output.stdout);
            if locale.starts_with("zh") {
                return Language::Chinese;
            }
        }
    }
    
    // Detect on Unix-like systems
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return Language::Chinese;
            }
        }
    }
    
    // Default to English
    Language::English
}