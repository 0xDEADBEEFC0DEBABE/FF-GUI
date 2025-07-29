use crate::bundled_ffmpeg::{get_bundled_ffmpeg};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{Receiver, Sender};

// Communication structures for IPC
#[derive(Debug, Serialize, Deserialize)]
pub enum FFmpegRequest {
    GetFileInfo { path: String },
    ConvertVideo { 
        input: String, 
        output: String, 
        codec: String,
        bitrate: Option<u32>,
        resolution: Option<(u32, u32)>,
    },
    ConvertAudio {
        input: String,
        output: String, 
        codec: String,
        bitrate: Option<u32>,
    },
    AddHardSubtitle {
        video_input: String,
        subtitle_input: String,
        output: String,
    },
    AddSoftSubtitle {
        video_input: String,
        subtitle_input: String,
        output: String,
    },
    DetectStreams { path: String },
    ExecuteCommand { args: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FFmpegResponse {
    Success,
    Error(String),
    FileInfo(MediaInfo),
    StreamInfo {
        has_video: bool,
        has_audio: bool,
        resolution: Option<(u32, u32)>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaInfo {
    pub filename: String,
    pub duration: f64,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoStreamInfo {
    pub index: u32,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioStreamInfo {
    pub index: u32,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
}

pub struct FFmpegWorker {
    request_sender: Sender<FFmpegRequest>,
    response_receiver: Receiver<FFmpegResponse>,
    progress_callback: Option<Arc<Mutex<Box<dyn Fn(f32) + Send + Sync>>>>,
}

impl FFmpegWorker {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = crossbeam_channel::unbounded();
        let (response_sender, response_receiver) = crossbeam_channel::unbounded();

        // Spawn worker thread
        thread::spawn(move || {
            let mut worker = FFmpegWorkerImpl::new(response_sender);
            worker.run(request_receiver);
        });

        Self {
            request_sender,
            response_receiver,
            progress_callback: None,
        }
    }

    pub fn get_file_info(&self, path: &str) -> Result<MediaInfo> {
        self.request_sender.send(FFmpegRequest::GetFileInfo { 
            path: path.to_string() 
        })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::FileInfo(info) => Ok(info),
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn convert_video(
        &self, 
        input: &str, 
        output: &str, 
        codec: &str,
        bitrate: Option<u32>,
        resolution: Option<(u32, u32)>,
    ) -> Result<()> {
        self.request_sender.send(FFmpegRequest::ConvertVideo {
            input: input.to_string(),
            output: output.to_string(),
            codec: codec.to_string(),
            bitrate,
            resolution,
        })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::Success => Ok(()),
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn add_hard_subtitle(&self, video_input: &str, subtitle_input: &str, output: &str) -> Result<()> {
        self.request_sender.send(FFmpegRequest::AddHardSubtitle {
            video_input: video_input.to_string(),
            subtitle_input: subtitle_input.to_string(),
            output: output.to_string(),
        })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::Success => Ok(()),
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn add_soft_subtitle(&self, video_input: &str, subtitle_input: &str, output: &str) -> Result<()> {
        self.request_sender.send(FFmpegRequest::AddSoftSubtitle {
            video_input: video_input.to_string(),
            subtitle_input: subtitle_input.to_string(),
            output: output.to_string(),
        })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::Success => Ok(()),
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn detect_streams(&self, path: &str) -> Result<(bool, bool, Option<(u32, u32)>)> {
        self.request_sender.send(FFmpegRequest::DetectStreams { 
            path: path.to_string() 
        })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::StreamInfo { has_video, has_audio, resolution } => {
                Ok((has_video, has_audio, resolution))
            },
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn execute_command_args(&self, args: Vec<String>) -> Result<()> {
        self.request_sender.send(FFmpegRequest::ExecuteCommand { args })?;
        
        match self.response_receiver.recv()? {
            FFmpegResponse::Success => Ok(()),
            FFmpegResponse::Error(e) => Err(anyhow::anyhow!(e)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(Box::new(callback))));
    }
}

struct FFmpegWorkerImpl {
    response_sender: Sender<FFmpegResponse>,
}

impl FFmpegWorkerImpl {
    fn new(response_sender: Sender<FFmpegResponse>) -> Self {
        Self { response_sender }
    }

    fn run(&mut self, request_receiver: Receiver<FFmpegRequest>) {
        while let Ok(request) = request_receiver.recv() {
            match request {
                FFmpegRequest::GetFileInfo { path } => {
                    self.handle_get_file_info(&path);
                }
                FFmpegRequest::ConvertVideo { input, output, codec, bitrate, resolution } => {
                    self.handle_convert_video(&input, &output, &codec, bitrate, resolution);
                }
                FFmpegRequest::AddHardSubtitle { video_input, subtitle_input, output } => {
                    self.handle_add_hard_subtitle(&video_input, &subtitle_input, &output);
                }
                FFmpegRequest::AddSoftSubtitle { video_input, subtitle_input, output } => {
                    self.handle_add_soft_subtitle(&video_input, &subtitle_input, &output);
                }
                FFmpegRequest::DetectStreams { path } => {
                    self.handle_detect_streams(&path);
                }
                FFmpegRequest::ExecuteCommand { args } => {
                    self.handle_execute_command(args);
                }
                _ => {}
            }
        }
    }

    fn handle_get_file_info(&self, path: &str) {
        let result = self.get_file_info_internal(path);
        match result {
            Ok(info) => {
                let _ = self.response_sender.send(FFmpegResponse::FileInfo(info));
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn get_file_info_internal(&self, path: &str) -> Result<MediaInfo> {
        // Use bundled FFprobe to get file information
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        let args = vec![
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path
        ];
        
        let output = bundled_ffmpeg.run_ffprobe(&args)?;
        let json_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse the JSON response
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse FFprobe output: {}", e))?;
        
        let format = json["format"].as_object()
            .ok_or_else(|| anyhow::anyhow!("No format information found"))?;
        
        let duration_str = format["duration"].as_str().unwrap_or("0.0");
        let duration = duration_str.parse::<f64>().unwrap_or(0.0);
        
        let streams = json["streams"].as_array()
            .ok_or_else(|| anyhow::anyhow!("No streams found"))?;
        
        let mut video_streams = Vec::new();
        let mut audio_streams = Vec::new();
        
        for (i, stream) in streams.iter().enumerate() {
            let codec_type = stream["codec_type"].as_str().unwrap_or("");
            let codec_name = stream["codec_name"].as_str().unwrap_or("unknown").to_string();
            
            if codec_type == "video" {
                let width = stream["width"].as_u64().unwrap_or(0) as u32;
                let height = stream["height"].as_u64().unwrap_or(0) as u32;
                let fps = stream["avg_frame_rate"].as_str()
                    .and_then(|fps_str| {
                        let parts: Vec<&str> = fps_str.split('/').collect();
                        if parts.len() == 2 {
                            let num = parts[0].parse::<f64>().unwrap_or(0.0);
                            let den = parts[1].parse::<f64>().unwrap_or(1.0);
                            if den != 0.0 { Some(num / den) } else { None }
                        } else {
                            None
                        }
                    }).unwrap_or(0.0);
                
                video_streams.push(VideoStreamInfo {
                    index: i as u32,
                    codec: codec_name,
                    width,
                    height,
                    fps,
                });
            } else if codec_type == "audio" {
                let sample_rate = stream["sample_rate"].as_str()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
                let channels = stream["channels"].as_u64().unwrap_or(0) as u32;
                
                audio_streams.push(AudioStreamInfo {
                    index: i as u32,
                    codec: codec_name,
                    sample_rate,
                    channels,
                });
            }
        }
        
        Ok(MediaInfo {
            filename: std::path::Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            duration,
            video_streams,
            audio_streams,
        })
    }

    fn handle_detect_streams(&self, path: &str) {
        let result = self.detect_streams_internal(path);
        match result {
            Ok((has_video, has_audio, resolution)) => {
                let _ = self.response_sender.send(FFmpegResponse::StreamInfo {
                    has_video,
                    has_audio,
                    resolution,
                });
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn detect_streams_internal(&self, path: &str) -> Result<(bool, bool, Option<(u32, u32)>)> {
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        let args = vec![
            "-v", "quiet",
            "-show_entries", "stream=codec_type,width,height",
            "-of", "csv=p=0",
            path
        ];
        
        let output = bundled_ffmpeg.run_ffprobe(&args)?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        let mut has_video = false;
        let mut has_audio = false;
        let mut resolution = None;
        
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if !parts.is_empty() {
                match parts[0].trim() {
                    "video" => {
                        has_video = true;
                        if parts.len() >= 3 {
                            if let (Ok(w), Ok(h)) = (parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                                resolution = Some((w, h));
                            }
                        }
                    }
                    "audio" => {
                        has_audio = true;
                    }
                    _ => {}
                }
            }
        }
        
        Ok((has_video, has_audio, resolution))
    }

    fn handle_convert_video(&self, input: &str, output: &str, codec: &str, bitrate: Option<u32>, resolution: Option<(u32, u32)>) {
        let result = self.convert_video_internal(input, output, codec, bitrate, resolution);
        match result {
            Ok(_) => {
                let _ = self.response_sender.send(FFmpegResponse::Success);
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn convert_video_internal(&self, input: &str, output: &str, codec: &str, bitrate: Option<u32>, resolution: Option<(u32, u32)>) -> Result<()> {
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        let mut args = vec![
            "-i".to_string(),
            input.to_string(),
            "-c:v".to_string(),
            codec.to_string(),
        ];
        
        if let Some(br) = bitrate {
            args.push("-b:v".to_string());
            args.push(format!("{}k", br));
        }
        
        if let Some((w, h)) = resolution {
            args.push("-s".to_string());
            args.push(format!("{}x{}", w, h));
        }
        
        args.push("-y".to_string());
        args.push(output.to_string());
        
        bundled_ffmpeg.run_ffmpeg(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map(|_| ())
    }

    fn handle_add_hard_subtitle(&self, video_input: &str, subtitle_input: &str, output: &str) {
        let result = Self::add_hard_subtitle_with_bundled_ffmpeg(video_input, subtitle_input, output);
        match result {
            Ok(_) => {
                let _ = self.response_sender.send(FFmpegResponse::Success);
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn add_hard_subtitle_with_bundled_ffmpeg(video_input: &str, subtitle_input: &str, output: &str) -> Result<()> {
        log_info!("Attempting hard subtitle operation with bundled FFmpeg");
        
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        log_info!("Using FFmpeg for hard subtitle burning");
        log_debug!("Video: {}", video_input);
        log_debug!("Subtitle: {}", subtitle_input);
        log_debug!("Output: {}", output);
        
        // Build FFmpeg command for hard subtitle burning
        let args = vec![
            "-i".to_string(),
            video_input.to_string(),
            "-vf".to_string(),
            format!("subtitles='{}'", subtitle_input.replace("\\", "\\\\").replace(":", "\\:")),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output.to_string(),
        ];
        
        bundled_ffmpeg.run_ffmpeg(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map(|_| ())
    }

    fn handle_add_soft_subtitle(&self, video_input: &str, subtitle_input: &str, output: &str) {
        let result = Self::add_soft_subtitle_with_bundled_ffmpeg(video_input, subtitle_input, output);
        match result {
            Ok(_) => {
                let _ = self.response_sender.send(FFmpegResponse::Success);
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn add_soft_subtitle_with_bundled_ffmpeg(video_input: &str, subtitle_input: &str, output: &str) -> Result<()> {
        log_info!("Attempting soft subtitle operation with bundled FFmpeg");
        
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        log_info!("Using FFmpeg for soft subtitle muxing");
        log_debug!("Video: {}", video_input);
        log_debug!("Subtitle: {}", subtitle_input);
        log_debug!("Output: {}", output);
        
        // Build FFmpeg command for soft subtitle muxing
        let args = vec![
            "-i".to_string(),
            video_input.to_string(),
            "-i".to_string(),
            subtitle_input.to_string(),
            "-c:v".to_string(),
            "copy".to_string(),
            "-c:a".to_string(),
            "copy".to_string(),
            "-c:s".to_string(),
            "copy".to_string(),
            "-y".to_string(),
            output.to_string(),
        ];
        
        bundled_ffmpeg.run_ffmpeg(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map(|_| ())
    }

    fn handle_execute_command(&self, args: Vec<String>) {
        let result = Self::execute_ffmpeg_command_via_bundled_ffmpeg(args);
        match result {
            Ok(_) => {
                let _ = self.response_sender.send(FFmpegResponse::Success);
            }
            Err(e) => {
                let _ = self.response_sender.send(FFmpegResponse::Error(e.to_string()));
            }
        }
    }

    fn execute_ffmpeg_command_via_bundled_ffmpeg(args: Vec<String>) -> Result<()> {
        log_debug!("Executing FFmpeg command through bundled executable: ffmpeg {}", args.join(" "));
        
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        bundled_ffmpeg.run_ffmpeg(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map(|_| ())
    }
}