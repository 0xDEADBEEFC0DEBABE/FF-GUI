use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::app_state::*;
use crate::codec_manager::*;
use crate::comprehensive_command_builder::ComprehensiveCommandBuilder;
use crate::comprehensive_codec_registry::CodecType;
use crate::bundled_ffmpeg::{get_bundled_ffmpeg};
use anyhow::Result;
use crate::{log_debug, log_info, log_warn, log_error};

/// Helper function to create bundled FFmpeg command
fn create_ffmpeg_command() -> Result<Command> {
    let bundled_ffmpeg = get_bundled_ffmpeg()?;
    Ok(bundled_ffmpeg.command())
}

/// Helper function to create bundled FFprobe command
fn create_ffprobe_command() -> Result<Command> {
    let bundled_ffmpeg = get_bundled_ffmpeg()?;
    Ok(bundled_ffmpeg.probe_command())
}

pub struct TaskExecutor {
    tasks: Arc<Mutex<Vec<ProcessingTask>>>,
    running: Arc<Mutex<bool>>,
}

impl TaskExecutor {
    
    /// Start a simple progress simulation for operations that don't provide progress info
    fn start_progress_simulation(
        tasks: Arc<Mutex<Vec<ProcessingTask>>>, 
        task_id: usize,
        estimated_duration_secs: u64
    ) {
        thread::spawn(move || {
            let start_time = std::time::Instant::now();
            let total_duration = Duration::from_secs(estimated_duration_secs);
            
            loop {
                thread::sleep(Duration::from_millis(500));
                
                let elapsed = start_time.elapsed();
                let progress = (elapsed.as_secs_f32() / total_duration.as_secs_f32()).min(0.95); // Cap at 95%
                
                // Update task progress
                if let Ok(mut tasks_guard) = tasks.try_lock() {
                    if let Some(task) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
                        if task.status == TaskStatus::Cancelled {
                            break;
                        }
                        if task.status != TaskStatus::Running {
                            break; // Task completed, stop simulation
                        }
                        task.progress = progress;
                    }
                } else {
                    continue;
                }
                
                // Stop if we've reached our estimated time
                if elapsed >= total_duration {
                    break;
                }
            }
        });
    }
    
    /// Estimate duration for operations that don't provide progress info
    fn estimate_operation_duration(operation: &OperationType, file_size_mb: Option<f64>) -> u64 {
        let base_duration = match operation {
            // Fast operations (1-10 seconds base)
            OperationType::ExtractAudio => 5,
            OperationType::ExtractVideo => 8,
            OperationType::AudioTrim => 3,
            OperationType::AudioVolume => 4,
            OperationType::VideoAudioSplit => 6,
            
            // Medium operations (10-30 seconds base)  
            OperationType::AudioConvert => 15,
            OperationType::AudioMerge => 12,
            OperationType::VideoAudioMerge => 18,
            OperationType::AudioResample => 20,
            
            // Slow operations (30+ seconds base)
            OperationType::VideoConvert => 45,
            OperationType::VideoCompress => 60,
            OperationType::VideoResize => 35,
            OperationType::VideoCrop => 40,
            OperationType::VideoRotate => 30,
            OperationType::VideoFilter => 50,
            OperationType::AddSubtitle => 55,
            OperationType::AddWatermark => 50,
            OperationType::VideoToGif => 80,
            OperationType::FrameExtract => 25,
            OperationType::BatchConvert => 30,
            OperationType::GifResize => 25,
            OperationType::AudioCompress => 25,
        };
        
        // Adjust based on file size if available
        if let Some(size_mb) = file_size_mb {
            let size_factor = (size_mb / 100.0).max(0.1).min(10.0); // Scale factor between 0.1 and 10
            (base_duration as f64 * size_factor) as u64
        } else {
            base_duration
        }
    }
    
    fn normalize_output_path_with_container(output_path: &str, codec: &str, container_format: &str, is_audio: bool) -> String {
        let path = std::path::Path::new(output_path);
        
        // Determine target extension: prioritize user-specified container format
        let target_ext = if !container_format.is_empty() && container_format != "auto" {
            container_format
        } else {
            // Fallback to determining extension based on encoder
            if is_audio {
                match codec {
                    "libmp3lame" => "mp3",
                    "aac" => "aac", 
                    "flac" => "flac",
                    "pcm_s16le" => "wav",
                    "libvorbis" => "ogg",
                    "libopus" => "opus",
                    "ac3" => "ac3",
                    _ => "mp3", // Default mp3
                }
            } else {
                match codec {
                    "libx264" | "libx265" => "mp4",
                    "libvpx-vp9" => "webm", 
                    "libaom-av1" => "mp4",
                    _ => "mp4", // Default mp4
                }
            }
        };
        
        // Always use target extension, ignore user input extension
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Some(parent) = path.parent() {
                parent.join(format!("{}.{}", stem, target_ext)).display().to_string()
            } else {
                format!("{}.{}", stem, target_ext)
            }
        } else {
            // If filename cannot be obtained, use default name
            if let Some(parent) = path.parent() {
                parent.join(format!("output.{}", target_ext)).display().to_string()
            } else {
                format!("output.{}", target_ext)
            }
        }
    }

    /// Execute FFmpeg command and filter AAC warnings, with progress update functionality
    
    /// Execute FFmpeg command, suppress AAC warnings but retain error information
    fn execute_ffmpeg_command_with_progress(
        mut cmd: Command, 
        tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>,
        task_id: Option<usize>
    ) -> Result<()> {
        use std::io::{BufRead, BufReader};
        use std::process::Stdio;
        
        // Print the actual command being executed for debugging
        log_debug!("Executing FFmpeg command: {:?}", cmd);
        
        // Set output capture for real-time progress reading
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());
        
        // Set process creation flags on Windows
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        // Set environment variables
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        cmd.env("FFMPEG_HIDE_BANNER", "1");
        
        let mut child = cmd.spawn()?;
        
        // Get stderr for real-time progress reading
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let tasks_clone = tasks.clone();
            let error_lines = Arc::new(Mutex::new(Vec::new()));
            let error_lines_clone = error_lines.clone();
            
            // Read FFmpeg output and update progress in new thread
            let handle = thread::spawn(move || {
                let mut total_duration: Option<f32> = None;
                
                for line in reader.lines() {
                    if let Ok(line) = line {
                        // Also check cancellation status in progress thread
                        if let (Some(tasks), Some(id)) = (&tasks_clone, task_id) {
                            if let Ok(tasks_guard) = tasks.try_lock() {
                                if let Some(task) = tasks_guard.iter().find(|t| t.id == id) {
                                    if task.status == TaskStatus::Cancelled {
                                        log_debug!("Task {} cancelled in progress thread", id);
                                        break; // Exit reading loop
                                    }
                                }
                            }
                        }
                        
                        // Collect all output lines for error diagnosis
                        if let Ok(mut errors) = error_lines_clone.lock() {
                            errors.push(line.clone());
                            // Keep last 100 lines of error information
                            if errors.len() > 100 {
                                errors.remove(0);
                            }
                        }
                        
                        // Parse Duration line to get total duration
                        if line.contains("Duration:") && total_duration.is_none() {
                            if let Some(duration_str) = Self::extract_duration(&line) {
                                total_duration = Self::parse_time_to_seconds(&duration_str);
                            }
                        }
                        
                        // Parse time progress line
                        if line.contains("time=") {
                            if let Some(time_str) = Self::extract_time_progress(&line) {
                                if let Some(current_seconds) = Self::parse_time_to_seconds(&time_str) {
                                    if let Some(total) = total_duration {
                                        let progress = (current_seconds / total).min(0.99);
                                        
                                        // Update task progress
                                        if let (Some(tasks), Some(id)) = (&tasks_clone, task_id) {
                                            if let Ok(mut tasks_guard) = tasks.try_lock() {
                                                if let Some(task) = tasks_guard.iter_mut().find(|t| t.id == id) {
                                                    if task.status == TaskStatus::Cancelled {
                                                        log_debug!("Task {} cancelled during progress update", id);
                                                        break; // Exit loop, stop processing
                                                    }
                                                    
                                                    task.progress = progress;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
            
            // Wait for process completion while checking cancellation status
            let status = loop {
                // Check if task was cancelled
                if let (Some(tasks), Some(id)) = (&tasks, task_id) {
                    if let Ok(tasks_guard) = tasks.try_lock() {
                        if let Some(task) = tasks_guard.iter().find(|t| t.id == id) {
                            if task.status == TaskStatus::Cancelled {
                                log_debug!("Task {} was cancelled, terminating FFmpeg process", id);
                                // Force terminate FFmpeg process
                                let _ = child.kill();
                                handle.join().unwrap_or_default();
                                return Err(anyhow::anyhow!("Task was cancelled by user"));
                            }
                        }
                    }
                }
                
                // Non-blocking check of process status
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        break exit_status;
                    }
                    Ok(None) => {
                        // Process still running, sleep briefly then continue checking
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        handle.join().unwrap_or_default();
                        return Err(anyhow::anyhow!("Failed to wait for child process: {}", e));
                    }
                }
            };
            handle.join().unwrap_or_default();
            
            if !status.success() {
                // Get error output with improved formatting
                let error_output = if let Ok(errors) = error_lines.lock() {
                    if errors.is_empty() {
                        "FFmpeg operation failed".to_string()
                    } else {
                        // Extract only critical error lines
                        let critical_errors: Vec<String> = errors.iter()
                            .filter(|line| {
                                let line_lower = line.to_lowercase();
                                line_lower.contains("error") || line_lower.contains("failed") || 
                                line_lower.contains("impossible") || line_lower.contains("not implemented") ||
                                line_lower.contains("invalid") || line_lower.contains("no such")
                            })
                            .take(3) // Limit to first 3 critical errors
                            .map(|line| {
                                // Clean up the error line - remove timestamps and unnecessary prefixes
                                line.trim()
                                    .replace("[ERROR]", "")
                                    .replace("[error]", "")
                                    .trim()
                                    .to_string()
                            })
                            .filter(|line| !line.is_empty())
                            .collect();
                        
                        if critical_errors.is_empty() {
                            "FFmpeg operation failed".to_string()
                        } else {
                            critical_errors.join("; ")
                        }
                    }
                } else {
                    "FFmpeg operation failed".to_string()
                };
                
                log_error!("FFmpeg failed with exit code: {:?}", status.code());
                log_error!("Critical errors: {}", error_output);
                
                return Err(anyhow::anyhow!("FFmpeg execution failed: {}", error_output));
            }
        } else {
            // If stderr cannot be obtained, fallback to waiting for process completion
            let output = child.wait_with_output()?;
            if !output.status.success() {
                return Err(anyhow::anyhow!("FFmpeg execution failed"));
            }
        }
        
        Ok(())
    }
    
    /// Extract Duration string from FFmpeg output
    fn extract_duration(line: &str) -> Option<String> {
        if let Some(start) = line.find("Duration: ") {
            let duration_part = &line[start + 10..];
            if let Some(end) = duration_part.find(',') {
                return Some(duration_part[..end].to_string());
            }
        }
        None
    }
    
    /// Extract time string from FFmpeg progress line
    fn extract_time_progress(line: &str) -> Option<String> {
        if let Some(start) = line.find("time=") {
            let time_part = &line[start + 5..];
            if let Some(end) = time_part.find(' ') {
                return Some(time_part[..end].to_string());
            } else {
                return Some(time_part.to_string());
            }
        }
        None
    }
    
    /// Convert time string (HH:MM:SS.ms) to seconds
    fn parse_time_to_seconds(time_str: &str) -> Option<f32> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 3 {
            if let (Ok(hours), Ok(minutes), Ok(seconds)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>()
            ) {
                return Some(hours * 3600.0 + minutes * 60.0 + seconds);
            }
        }
        None
    }
    /// Generate preview FFmpeg command without execution
    pub fn preview_command(task: &ProcessingTask) -> Result<String> {
        match task.operation {
            OperationType::VideoConvert => Self::preview_video_convert(task),
            OperationType::AudioConvert => Self::preview_audio_convert(task),
            OperationType::VideoCompress => Self::preview_video_convert(task),
            OperationType::AudioCompress => Self::preview_audio_convert(task),
            OperationType::VideoResize => Self::preview_video_convert(task),
            OperationType::AudioResample => Self::preview_audio_convert(task),
            OperationType::AudioVolume => Self::preview_audio_convert(task),
            OperationType::VideoAudioMerge => Self::preview_video_audio_merge(task),
            OperationType::VideoAudioSplit => Self::preview_extract_audio(task),
            OperationType::ExtractAudio => Self::preview_extract_audio(task),
            OperationType::ExtractVideo => Self::preview_extract_video(task),
            OperationType::VideoCrop => Self::preview_video_crop(task),
            OperationType::VideoRotate => Self::preview_video_rotate(task),
            OperationType::VideoFilter => Self::preview_video_filter(task),
            OperationType::AudioTrim => Self::preview_audio_trim(task),
            OperationType::AudioMerge => Self::preview_audio_merge(task),
            OperationType::BatchConvert => Self::preview_batch_convert(task),
            OperationType::AddSubtitle => Self::preview_add_subtitle(task),
            OperationType::AddWatermark => Self::preview_add_watermark(task),
            OperationType::FrameExtract => Self::preview_frame_extract(task),
            OperationType::VideoToGif => Self::preview_video_to_gif(task),
            OperationType::GifResize => Self::preview_gif_resize(task),
        }
    }
    pub fn new(tasks: Arc<Mutex<Vec<ProcessingTask>>>) -> Self {
        Self {
            tasks,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let tasks = self.tasks.clone();
        let running = self.running.clone();
        
        // Set running status
        *running.lock().unwrap() = true;
        
        thread::spawn(move || {
            loop {
                // Check if should continue running
                if !*running.lock().unwrap() {
                    break;
                }
                
                // Find pending tasks
                let task_to_process = {
                    let mut tasks_guard = tasks.lock().unwrap();
                    let pending_task_index = tasks_guard.iter()
                        .position(|task| task.status == TaskStatus::Pending);
                    
                    if let Some(index) = pending_task_index {
                        // Set task status to running
                        tasks_guard[index].status = TaskStatus::Running;
                        tasks_guard[index].progress = 0.0;
                        tasks_guard[index].start_time = Some(std::time::Instant::now());
                        
                        
                        Some(tasks_guard[index].clone())
                    } else {
                        None
                    }
                };
                
                if let Some(mut task) = task_to_process {
                    let task_id = task.id;
                    
                    // Check if task was cancelled before execution
                    {
                        let tasks_guard = tasks.lock().unwrap();
                        if let Some(task_in_list) = tasks_guard.iter().find(|t| t.id == task_id) {
                            if task_in_list.status == TaskStatus::Cancelled {
                                log_debug!("Task {} was cancelled before execution", task_id);
                                continue;
                            }
                        }
                    }
                    
                    // Execute task with progress update
                    let tasks_for_progress = tasks.clone();
                    let result = Self::execute_task_with_progress(&mut task, tasks_for_progress);
                    
                    // Update task status
                    let mut tasks_guard = tasks.lock().unwrap();
                    if let Some(task_in_list) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
                        match result {
                            Ok(()) => {
                                task_in_list.status = TaskStatus::Completed;
                                task_in_list.progress = 1.0;
                                task_in_list.error_message = None;
                                // Store completion time when task completes
                                if let Some(start_time) = task_in_list.start_time {
                                    task_in_list.completion_time = Some(start_time.elapsed());
                                }
                            }
                            Err(e) => {
                                task_in_list.status = TaskStatus::Failed;
                                task_in_list.error_message = Some(e.to_string());
                            }
                        }
                    }
                }
                
                // Brief sleep to avoid excessive CPU usage
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
    
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
        self.terminate_all_ffmpeg_processes();
    }
    
    /// Terminate all FFmpeg processes
    pub fn terminate_all_ffmpeg_processes(&self) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(&["/F", "/IM", "ffmpeg.exe"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("pkill")
                .args(&["-f", "ffmpeg"])
                .output();
        }
    }
    
    /// Public method to execute a single task directly (for workflow use)
    pub fn execute_task(task: &mut ProcessingTask) -> Result<()> {
        // Create dummy params for the internal method
        let dummy_tasks = Arc::new(Mutex::new(Vec::new()));
        Self::execute_task_with_shared_progress(task, dummy_tasks, task.id)
    }

    fn execute_task_with_progress(task: &mut ProcessingTask, tasks: Arc<Mutex<Vec<ProcessingTask>>>) -> Result<()> {
        // No longer start old time estimation progress thread, directly use FFmpeg real-time progress
        let task_id = task.id;
        
        // Execute actual task, pass progress update parameters
        Self::execute_task_with_shared_progress(task, tasks, task_id)
    }

    fn execute_task_with_shared_progress(
        task: &mut ProcessingTask, 
        tasks: Arc<Mutex<Vec<ProcessingTask>>>,
        task_id: usize
    ) -> Result<()> {
        // Check if task was cancelled before starting
        {
            let tasks_guard = tasks.lock().unwrap();
            if let Some(task_in_list) = tasks_guard.iter().find(|t| t.id == task_id) {
                if task_in_list.status == TaskStatus::Cancelled {
                    return Err(anyhow::anyhow!("Task was cancelled"));
                }
            }
        }
        
        // Pass tasks and task_id to enable real FFmpeg progress tracking
        match task.operation {
            OperationType::VideoConvert | OperationType::VideoCompress | OperationType::VideoResize => {
                Self::execute_video_convert_with_progress(task, Some(tasks), Some(task_id))
            },
            OperationType::AudioConvert | OperationType::AudioCompress | OperationType::AudioResample | OperationType::AudioVolume => {
                Self::execute_audio_convert_with_progress(task, Some(tasks), Some(task_id))
            },
            OperationType::VideoAudioMerge => Self::execute_video_audio_merge_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::VideoAudioSplit => Self::execute_video_audio_split_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::ExtractAudio => Self::execute_extract_audio_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::ExtractVideo => Self::execute_extract_video_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::VideoCrop => Self::execute_video_crop_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::VideoRotate => Self::execute_video_rotate_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::VideoFilter => Self::execute_video_filter_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::AudioTrim => Self::execute_audio_trim_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::AudioMerge => Self::execute_audio_merge_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::BatchConvert => Self::execute_batch_convert_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::AddSubtitle => Self::execute_add_subtitle_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::AddWatermark => Self::execute_add_watermark_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::FrameExtract => Self::execute_frame_extract_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::VideoToGif => Self::execute_video_to_gif_with_progress(task, Some(tasks), Some(task_id)),
            OperationType::GifResize => Self::execute_gif_resize_with_progress(task, Some(tasks), Some(task_id)),
        }
    }

    // Create wrapper functions with progress tracking for other operations
    fn execute_audio_convert_with_progress(task: &mut ProcessingTask, tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, task_id: Option<usize>) -> Result<()> {
        // Start progress simulation for operations without real progress info
        if let (Some(tasks), Some(id)) = (tasks.clone(), task_id) {
            let file_size_mb = if !task.input_files.is_empty() {
                std::fs::metadata(&task.input_files[0])
                    .ok()
                    .map(|m| m.len() as f64 / 1024.0 / 1024.0)
            } else {
                None
            };
            let estimated_duration = Self::estimate_operation_duration(&task.operation, file_size_mb);
            Self::start_progress_simulation(tasks, id, estimated_duration);
        }
        Self::execute_audio_convert(task)
    }
    
    fn execute_video_audio_merge_with_progress(task: &mut ProcessingTask, tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, task_id: Option<usize>) -> Result<()> {
        if let (Some(tasks), Some(id)) = (tasks.clone(), task_id) {
            let file_size_mb = if !task.input_files.is_empty() {
                std::fs::metadata(&task.input_files[0])
                    .ok()
                    .map(|m| m.len() as f64 / 1024.0 / 1024.0)
            } else {
                None
            };
            let estimated_duration = Self::estimate_operation_duration(&task.operation, file_size_mb);
            Self::start_progress_simulation(tasks, id, estimated_duration);
        }
        Self::execute_video_audio_merge(task)
    }
    
    fn execute_video_audio_split_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_video_audio_split(task)
    }
    
    fn execute_extract_audio_with_progress(task: &mut ProcessingTask, tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, task_id: Option<usize>) -> Result<()> {
        if let (Some(tasks), Some(id)) = (tasks.clone(), task_id) {
            let file_size_mb = if !task.input_files.is_empty() {
                std::fs::metadata(&task.input_files[0])
                    .ok()
                    .map(|m| m.len() as f64 / 1024.0 / 1024.0)
            } else {
                None
            };
            let estimated_duration = Self::estimate_operation_duration(&task.operation, file_size_mb);
            Self::start_progress_simulation(tasks, id, estimated_duration);
        }
        Self::execute_extract_audio(task)
    }
    
    fn execute_extract_video_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_extract_video(task)
    }
    
    fn execute_video_crop_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_video_crop(task)
    }
    
    fn execute_video_rotate_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_video_rotate(task)
    }
    
    fn execute_video_filter_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_video_filter(task)
    }
    
    fn execute_audio_trim_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_audio_trim(task)
    }
    
    fn execute_audio_merge_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_audio_merge(task)
    }
    
    fn execute_batch_convert_with_progress(task: &mut ProcessingTask, tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, task_id: Option<usize>) -> Result<()> {
        if let (Some(tasks), Some(id)) = (tasks.clone(), task_id) {
            // For batch convert, estimate based on total file size
            let total_size_mb: f64 = task.input_files.iter()
                .filter_map(|file| std::fs::metadata(file).ok())
                .map(|m| m.len() as f64 / 1024.0 / 1024.0)
                .sum();
            let estimated_duration = Self::estimate_operation_duration(&task.operation, if total_size_mb > 0.0 { Some(total_size_mb) } else { None });
            Self::start_progress_simulation(tasks, id, estimated_duration);
        }
        Self::execute_batch_convert(task)
    }
    
    fn execute_add_subtitle_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_add_subtitle(task)
    }
    
    fn execute_add_watermark_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_add_watermark(task)
    }
    
    fn execute_frame_extract_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_frame_extract(task)
    }
    
    fn execute_video_to_gif_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_video_to_gif(task)
    }
    
    fn execute_gif_resize_with_progress(task: &mut ProcessingTask, _tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>, _task_id: Option<usize>) -> Result<()> {
        Self::execute_gif_resize(task)
    }


    fn execute_audio_convert(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart Codec
        let target_format = if !audio_settings.format.is_empty() && audio_settings.format != "auto" {
            &audio_settings.format
        } else {
            &output_ext
        };
        
        let mut codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(target_format)
        } else {
            audio_settings.codec.clone()
        };

        // Validate encoder and format compatibility, silent fix
        if CodecManager::validate_codec_format_compatibility(&codec, target_format, true).is_err() {
            codec = CodecManager::get_best_audio_codec_for_format(target_format);
        }

        // First try simple conversion method
        return Self::execute_simple_audio_convert(input_file, output_file, &codec, target_format, audio_settings);
    }

    /// Simplified audio conversion to avoid issues from complex parameters
    fn execute_simple_audio_convert(
        input_file: &str,
        output_file: &str,
        codec: &str,
        target_format: &str,
        audio_settings: &AudioSettings,
    ) -> Result<()> {
        log_info!("Using comprehensive command builder for audio conversion");
        
        // Create updated audio settings with current codec and format
        let mut updated_audio_settings = audio_settings.clone();
        updated_audio_settings.codec = codec.to_string();
        updated_audio_settings.format = target_format.to_string();
        
        // Validate codec-format compatibility
        if let Err(e) = ComprehensiveCommandBuilder::validate_codec_format_combination(&updated_audio_settings.codec, &updated_audio_settings.format) {
            log_warn!("Audio codec-format compatibility issue: {}", e);
            
            // Try to find a compatible codec
            if let Some(recommended_codec) = ComprehensiveCommandBuilder::get_recommended_codec(&updated_audio_settings.format, CodecType::Audio) {
                log_info!("Using recommended audio codec {} for format {}", recommended_codec, updated_audio_settings.format);
                updated_audio_settings.codec = recommended_codec;
            }
        }
        
        // Build comprehensive FFmpeg command using the comprehensive command builder
        let ffmpeg_args = ComprehensiveCommandBuilder::build_audio_conversion_command(
            input_file,
            output_file,
            &updated_audio_settings
        )?;
        
        // Create FFmpeg command
        let mut cmd = create_ffmpeg_command()?;
        
        // Add all the comprehensive FFmpeg arguments
        for arg in ffmpeg_args {
            cmd.arg(arg);
        }
        
        // Execute the command
        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("FFmpeg audio conversion failed: {}", stderr));
        }
        
        Ok(())
    }

    fn execute_video_convert_with_progress(
        task: &mut ProcessingTask, 
        tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>,
        task_id: Option<usize>
    ) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart video codec selection
        let mut video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        // Normalize output file path, prioritize user-specified container format
        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, &video_settings.container_format, false);
        
        // Get final output format (after normalization)
        let final_output_ext = std::path::Path::new(&normalized_output)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Validate video codec and format compatibility, silent fix (using final format)
        if CodecManager::validate_codec_format_compatibility(&video_codec, &final_output_ext, false).is_err() {
            video_codec = CodecManager::get_best_video_codec_for_format(&final_output_ext);
        }

        // Smart audio codec selection (using final format)
        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&final_output_ext)
            } else if audio_settings.codec == "copy" {
                "copy".to_string()
            } else {
                audio_settings.codec.clone()
            }
        } else {
            "copy".to_string()
        };


        // Use simplified video conversion method, pass normalized output path
        return Self::execute_simple_video_convert(
            input_file, 
            &normalized_output, 
            &video_codec, 
            &audio_codec, 
            &final_output_ext, 
            video_settings, 
            task.audio_settings.as_ref(),
            tasks,
            task_id,
            task.operation.clone()
        );
    }

    /// Detect if file has audio stream
    fn has_audio_stream(input_file: &str) -> bool {
        // Use ffprobe to detect audio stream
        let output = match create_ffprobe_command() {
            Ok(cmd) => cmd,
            Err(_) => return false,
        }
            .arg("-v").arg("error")
            .arg("-select_streams").arg("a:0")
            .arg("-show_entries").arg("stream=codec_type")
            .arg("-of").arg("default=noprint_wrappers=1:nokey=1")
            .arg(input_file)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.trim() == "audio"
            }
            Err(_) => {
                // If ffprobe fails, assume audio stream exists to avoid incorrectly skipping audio processing
                true
            }
        }
    }

    /// Simplified video conversion, ensure codec and format matching
    fn execute_simple_video_convert(
        input_file: &str,
        output_file: &str,
        video_codec: &str,
        audio_codec: &str,
        output_ext: &str,
        video_settings: &VideoSettings,
        audio_settings: Option<&AudioSettings>,
        tasks: Option<Arc<Mutex<Vec<ProcessingTask>>>>,
        task_id: Option<usize>,
        _operation_type: OperationType,
    ) -> Result<()> {
        log_info!("Using comprehensive command builder for video conversion");
        
        // Create updated video settings with current codec and format
        let mut updated_video_settings = video_settings.clone();
        updated_video_settings.codec = video_codec.to_string();
        updated_video_settings.container_format = output_ext.to_string();
        
        // Create updated audio settings if provided
        let updated_audio_settings = audio_settings.map(|as_ref| {
            let mut audio_copy = as_ref.clone();
            audio_copy.codec = audio_codec.to_string();
            audio_copy.format = output_ext.to_string();
            audio_copy
        });
        
        // Validate codec-format compatibility and fallback if needed
        if let Err(e) = ComprehensiveCommandBuilder::validate_codec_format_combination(&updated_video_settings.codec, &updated_video_settings.container_format) {
            log_warn!("Codec-format compatibility issue: {}", e);
            
            // Try to find a compatible codec
            if let Some(recommended_codec) = ComprehensiveCommandBuilder::get_recommended_codec(&updated_video_settings.container_format, CodecType::Video) {
                log_info!("Using recommended codec {} for format {}", recommended_codec, updated_video_settings.container_format);
                updated_video_settings.codec = recommended_codec;
            }
        }
        
        // Similarly validate audio codec if provided
        if let Some(ref audio_settings) = updated_audio_settings {
            if let Err(e) = ComprehensiveCommandBuilder::validate_codec_format_combination(&audio_settings.codec, &updated_video_settings.container_format) {
                log_warn!("Audio codec-format compatibility issue: {}", e);
            }
        }
        
        // Build comprehensive FFmpeg command using the comprehensive command builder
        let ffmpeg_args = ComprehensiveCommandBuilder::build_video_conversion_command(
            input_file,
            output_file,
            &updated_video_settings,
            updated_audio_settings.as_ref()
        )?;
        
        // Create FFmpeg command with hardware acceleration if needed
        let mut cmd = create_ffmpeg_command()?;
        
        // Add hardware acceleration settings before input (with input codec validation)
        Self::add_hardware_acceleration_args_with_validation(&mut cmd, input_file, &updated_video_settings.codec, updated_video_settings.use_hardware_acceleration);
        
        // Add progress tracking arguments
        cmd.arg("-v").arg("info");
        cmd.arg("-hide_banner");
        cmd.arg("-stats");
        cmd.arg("-nostdin");
        cmd.arg("-progress").arg("pipe:2");
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        
        // Add all the comprehensive FFmpeg arguments
        for arg in ffmpeg_args {
            cmd.arg(arg);
        }
        
        // Add custom arguments if specified
        if !updated_video_settings.custom_args.is_empty() {
            let custom_args: Vec<&str> = updated_video_settings.custom_args.split_whitespace().collect();
            for arg in custom_args {
                cmd.arg(arg);
            }
        }
        
        // Execute the command with progress tracking
        Self::execute_ffmpeg_command_with_progress(cmd, tasks, task_id)
    }

    /// Check if a codec is a hardware encoder
    fn is_hardware_encoder_codec(codec: &str) -> bool {
        matches!(codec, 
            "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" |
            "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" |
            "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" |
            "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" |
            "h264_amf" | "hevc_amf" | "av1_amf"
        )
    }
    
    /// Check if hardware decoder can handle the input video codec
    fn can_hardware_decode_codec(input_codec: &str, hw_accel_type: &str) -> bool {
        match hw_accel_type {
            "cuda" => {
                // NVIDIA CUDA decoders
                matches!(input_codec.to_lowercase().as_str(), 
                    "h264" | "h.264" | "avc1" | "avc" |
                    "hevc" | "h265" | "h.265" | "hvc1" |
                    "av1" | "av01" |
                    "vp8" | "vp9" |
                    "mpeg2video" | "mpeg2" |
                    "mpeg4"
                )
            },
            "qsv" => {
                // Intel QuickSync decoders
                matches!(input_codec.to_lowercase().as_str(),
                    "h264" | "h.264" | "avc1" | "avc" |
                    "hevc" | "h265" | "h.265" | "hvc1" |
                    "av1" | "av01" |
                    "vp9" |
                    "mpeg2video" | "mpeg2" |
                    "vc1"
                )
            },
            "videotoolbox" => {
                // Apple VideoToolbox decoders
                matches!(input_codec.to_lowercase().as_str(),
                    "h264" | "h.264" | "avc1" | "avc" |
                    "hevc" | "h265" | "h.265" | "hvc1" |
                    "prores" |
                    "mpeg4"
                )
            },
            "vaapi" => {
                // VAAPI (Linux) decoders
                matches!(input_codec.to_lowercase().as_str(),
                    "h264" | "h.264" | "avc1" | "avc" |
                    "hevc" | "h265" | "h.265" | "hvc1" |
                    "av1" | "av01" |
                    "vp8" | "vp9" |
                    "mpeg2video" | "mpeg2"
                )
            },
            "d3d11va" => {
                // AMD AMF/D3D11VA decoders
                matches!(input_codec.to_lowercase().as_str(),
                    "h264" | "h.264" | "avc1" | "avc" |
                    "hevc" | "h265" | "h.265" | "hvc1" |
                    "av1" | "av01" |
                    "vp9"
                )
            },
            _ => false
        }
    }

    /// Detect input video codec using ffprobe
    fn detect_input_video_codec(input_file: &str) -> Result<String> {
        let output = create_ffprobe_command()?
            .args(&[
                "-v", "quiet",
                "-select_streams", "v:0",
                "-show_entries", "stream=codec_name",
                "-of", "csv=p=0",
                input_file
            ])
            .output()?;

        if output.status.success() {
            let codec = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            if !codec.is_empty() {
                Ok(codec)
            } else {
                Err(anyhow::anyhow!("Could not detect video codec"))
            }
        } else {
            Err(anyhow::anyhow!("ffprobe failed to detect codec"))
        }
    }

    /// Add hardware acceleration arguments with input codec validation
    fn add_hardware_acceleration_args_with_validation(cmd: &mut Command, input_file: &str, codec: &str, use_hw_accel: bool) {
        // Detect input codec first
        let input_codec = match Self::detect_input_video_codec(input_file) {
            Ok(codec) => codec,
            Err(_) => {
                log_warn!("Could not detect input codec, falling back to software decoding");
                return;
            }
        };

        // Only add hardware decode acceleration if user explicitly enabled it or using hardware encoder
        if Self::is_hardware_encoder_codec(codec) {
            // Hardware encoder requires compatible hardware decode acceleration
            let hw_accel_type = match codec {
                "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" => "cuda",
                "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" => "qsv",
                "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" => "videotoolbox",
                "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" => "vaapi",
                "h264_amf" | "hevc_amf" | "av1_amf" => "d3d11va",
                _ => return,
            };

            // Check if hardware decoder can handle input codec
            if Self::can_hardware_decode_codec(&input_codec, hw_accel_type) {
                match codec {
                    "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" => {
                        cmd.arg("-hwaccel").arg("cuda");
                        cmd.arg("-hwaccel_output_format").arg("cuda");
                        log_info!("🚀 CUDA hardware acceleration enabled for {} input", input_codec);
                    },
                    "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" => {
                        cmd.arg("-hwaccel").arg("qsv");
                        cmd.arg("-hwaccel_output_format").arg("qsv");
                        log_info!("🚀 QSV hardware acceleration enabled for {} input", input_codec);
                    },
                    "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" => {
                        cmd.arg("-hwaccel").arg("videotoolbox");
                        log_info!("🚀 VideoToolbox hardware acceleration enabled for {} input", input_codec);
                    },
                    "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" => {
                        cmd.arg("-hwaccel").arg("vaapi");
                        cmd.arg("-hwaccel_output_format").arg("vaapi");
                        log_info!("🚀 VAAPI hardware acceleration enabled for {} input", input_codec);
                    },
                    "h264_amf" | "hevc_amf" | "av1_amf" => {
                        cmd.arg("-hwaccel").arg("d3d11va");
                        cmd.arg("-hwaccel_output_format").arg("d3d11");
                        log_info!("🚀 AMF hardware acceleration enabled for {} input", input_codec);
                    },
                    _ => {}
                }
            } else {
                log_warn!("⚠ Hardware encoder {} requested but input codec {} not supported by hardware decoder, using software decoding", codec, input_codec);
            }
        } else if use_hw_accel {
            // User explicitly enabled hardware acceleration for software encoder
            cmd.arg("-hwaccel").arg("auto");
            log_info!("🔧 Hardware decoding (auto) enabled for software encoder with {} input", input_codec);
        }
        // If neither condition is met, use software decoding (no hwaccel args)
    }

    /// Add hardware acceleration arguments based on codec and user preference
    fn add_hardware_acceleration_args(cmd: &mut Command, codec: &str, use_hw_accel: bool) {
        // Only add hardware decode acceleration if user explicitly enabled it or using hardware encoder
        if Self::is_hardware_encoder_codec(codec) {
            // Hardware encoder requires compatible hardware decode acceleration
            match codec {
                "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" => {
                    cmd.arg("-hwaccel").arg("cuda");
                    cmd.arg("-hwaccel_output_format").arg("cuda");
                },
                "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" => {
                    cmd.arg("-hwaccel").arg("qsv");
                    cmd.arg("-hwaccel_output_format").arg("qsv");
                },
                "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" => {
                    cmd.arg("-hwaccel").arg("videotoolbox");
                },
                "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" => {
                    cmd.arg("-hwaccel").arg("vaapi");
                    cmd.arg("-hwaccel_output_format").arg("vaapi");
                },
                "h264_amf" | "hevc_amf" | "av1_amf" => {
                    cmd.arg("-hwaccel").arg("d3d11va");
                    cmd.arg("-hwaccel_output_format").arg("d3d11");
                },
                _ => {}
            }
        } else if use_hw_accel {
            // User explicitly enabled hardware acceleration for software encoder
            cmd.arg("-hwaccel").arg("auto");
        }
        // If neither condition is met, use software decoding (no hwaccel args)
    }

    /// Calculate target bitrate based on file size and duration
    fn calculate_target_bitrate(input_file: &str, target_size_mb: i32) -> Result<i32> {
        // Get video duration using ffprobe
        let output = create_ffprobe_command()?
            .args(&[
                "-v", "quiet",
                "-show_entries", "format=duration",
                "-of", "csv=p=0",
                input_file
            ])
            .output();
            
        let duration_seconds = match output {
            Ok(result) => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                output_str.trim().parse::<f32>().unwrap_or(60.0) // Default to 60 seconds if parsing fails
            }
            Err(_) => 60.0 // Default duration
        };
        
        // Calculate target bitrate: (target_size_mb * 8 * 1024) / duration_seconds * 0.9 (leave 10% for audio)
        let target_bitrate = ((target_size_mb as f32 * 8.0 * 1024.0) / duration_seconds * 0.9) as i32;
        
        // Ensure minimum bitrate
        Ok(target_bitrate.max(100))
    }
    
    /// Convert CRF quality value to estimated bitrate
    fn crf_to_bitrate_estimate(crf: i32) -> String {
        // Experience-based CRF to bitrate conversion
        let bitrate_kbps = match crf {
            0..=18 => 8000,  // Very high quality
            19..=23 => 4000, // High quality 
            24..=28 => 2000, // Medium quality
            29..=35 => 1000, // Low quality
            36..=51 => 500,  // Very low quality
            _ => 2000,       // Default
        };
        format!("{}k", bitrate_kbps)
    }


    fn execute_video_audio_merge(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 files for video/audio merge"));
        }

        let video_file = &task.input_files[0];
        let audio_file = &task.input_files[1];
        let output_file = &task.output_file;

        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart audio codec selection
        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&output_ext)
            } else {
                audio_settings.codec.clone()
            }
        } else {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        };

        // Always use bundled FFmpeg approach

        // Fallback to external FFmpeg command
        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(video_file);
        cmd.arg("-i").arg(audio_file);
        cmd.arg("-y"); // Overwrite output file
        cmd.arg("-v").arg("error"); // Show error level information
        cmd.arg("-hide_banner"); // Hide banner information
        cmd.arg("-nostats"); // Disable statistics
        cmd.arg("-nostdin"); // Disable interactive input
        
        // Set environment variables
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        
        // Copy video stream (usually no re-encoding needed)
        cmd.arg("-c:v").arg("copy");
        
        // Set audio encoder
        cmd.arg("-c:a").arg(&audio_codec);
        
        // Audio parameters
        if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                cmd.arg("-b:a").arg(&audio_settings.bitrate);
            } else {
                // Use default bitrate
                let default_bitrate = match audio_codec.as_str() {
                    "libmp3lame" => "128k",
                    "aac" => "128k",
                    "libvorbis" => "128k",
                    "libopus" => "128k",
                    _ => "128k"
                };
                cmd.arg("-b:a").arg(default_bitrate);
            }
            
            if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
                cmd.arg("-ar").arg(&audio_settings.sample_rate);
            }
            
            if audio_settings.channels != "auto" && !audio_settings.channels.is_empty() {
                cmd.arg("-ac").arg(&audio_settings.channels);
            }
        }
        
        // Detect streams and map them intelligently
        // Use FFprobe to detect available streams
        let bundled_ffmpeg = get_bundled_ffmpeg()?;
        
        // Check first file for video/audio streams
        let first_file_args = vec!["-v", "quiet", "-show_entries", "stream=codec_type", "-of", "csv=p=0", video_file];
        let first_output = bundled_ffmpeg.run_ffprobe(&first_file_args)?;
        let first_streams = String::from_utf8_lossy(&first_output.stdout);
        let first_has_video = first_streams.lines().any(|line| line.trim() == "video");
        let first_has_audio = first_streams.lines().any(|line| line.trim() == "audio");
        
        // Check second file for video/audio streams  
        let second_file_args = vec!["-v", "quiet", "-show_entries", "stream=codec_type", "-of", "csv=p=0", audio_file];
        let second_output = bundled_ffmpeg.run_ffprobe(&second_file_args)?;
        let second_streams = String::from_utf8_lossy(&second_output.stdout);  
        let second_has_video = second_streams.lines().any(|line| line.trim() == "video");
        let second_has_audio = second_streams.lines().any(|line| line.trim() == "audio");
        
        log_debug!("First file streams - Video: {}, Audio: {}", first_has_video, first_has_audio);
        log_debug!("Second file streams - Video: {}, Audio: {}", second_has_video, second_has_audio);
        
        // Map video stream (prefer first file, fallback to second)
        if first_has_video {
            cmd.arg("-map").arg("0:v");
        } else if second_has_video {
            cmd.arg("-map").arg("1:v");
        } else {
            return Err(anyhow::anyhow!("No video stream found in either input file"));
        }
        
        // Map audio stream (prefer second file, fallback to first)
        if second_has_audio {
            cmd.arg("-map").arg("1:a");
        } else if first_has_audio {
            cmd.arg("-map").arg("0:a");
        } else {
            return Err(anyhow::anyhow!("No audio stream found in either input file"));
        }
        
        cmd.arg(output_file);
        
        // Use common FFmpeg execution function
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_video_audio_split(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }
        let input_file = &task.input_files[0];
        let output_dir = &task.output_file; // Output path treated as folder path
        
        // Ensure output directory exists
        std::fs::create_dir_all(output_dir)?;
        
        // Get input filename (without extension)
        let input_path = std::path::Path::new(input_file);
        let file_stem = input_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        
        // Smart codec and output format selection
        let (video_codec, video_ext) = if let Some(video_settings) = &task.video_settings {
            if video_settings.codec == "auto" {
                ("libx264".to_string(), "mp4".to_string())
            } else if video_settings.codec == "copy" {
                // Keep original format when copying, default mp4
                ("copy".to_string(), "mp4".to_string())
            } else {
                // Select appropriate container format based on encoder
                let ext = match video_settings.codec.as_str() {
                    "libx264" | "libx265" => "mp4",
                    "libvpx" | "libvpx-vp9" => "webm",
                    "libav1" => "mp4",
                    "prores" => "mov",
                    "mkv" => "mkv",
                    _ => "mp4", // Default mp4
                };
                (video_settings.codec.clone(), ext.to_string())
            }
        } else {
            ("copy".to_string(), "mp4".to_string())
        };
        
        let (audio_codec, audio_ext) = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                ("pcm_s16le".to_string(), "wav".to_string()) // Lossless audio
            } else if audio_settings.codec == "copy" {
                // Keep original format when copying, default wav
                ("copy".to_string(), "wav".to_string())
            } else {
                // Select appropriate container format based on encoder
                let ext = match audio_settings.codec.as_str() {
                    "pcm_s16le" | "pcm_s24le" | "pcm_s32le" => "wav",
                    "mp3" | "libmp3lame" => "mp3",
                    "aac" | "libfdk_aac" => "m4a",
                    "flac" => "flac",
                    "vorbis" | "libvorbis" => "ogg",
                    "opus" | "libopus" => "opus",
                    _ => "wav", // Default wav
                };
                (audio_settings.codec.clone(), ext.to_string())
            }
        } else {
            ("copy".to_string(), "wav".to_string())
        };
        
        // Build output file path (select extension based on encoder)
        let video_output = std::path::Path::new(output_dir).join(format!("{}_video.{}", file_stem, video_ext));
        let audio_output = std::path::Path::new(output_dir).join(format!("{}_audio.{}", file_stem, audio_ext));
        
        log_info!("Splitting video and audio to directory: {}", output_dir);
        log_info!("Video output: {} (codec: {})", video_output.display(), video_codec);
        log_info!("Audio output: {} (codec: {})", audio_output.display(), audio_codec);
        
        // Extract video (no audio)
        let mut video_cmd = create_ffmpeg_command()?;
        video_cmd.arg("-i").arg(input_file);
        video_cmd.arg("-y"); // Overwrite output file
        video_cmd.arg("-v").arg("error");
        video_cmd.arg("-hide_banner");
        video_cmd.arg("-nostats");
        video_cmd.arg("-nostdin");
        video_cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        
        video_cmd.arg("-an"); // Disable audio
        if video_codec == "copy" || video_codec == "mkv" {
            video_cmd.arg("-c:v").arg("copy");
        } else {
            video_cmd.arg("-c:v").arg(&video_codec);
            
            // Add quality parameters based on encoder
            if let Some(video_settings) = &task.video_settings {
                match video_codec.as_str() {
                    "libx264" | "libx265" => {
                        // Use CRF quality control
                        if video_settings.crf > 0 && video_settings.crf <= 51 {
                            video_cmd.arg("-crf").arg(video_settings.crf.to_string());
                        } else if video_settings.quality > 0 && video_settings.quality <= 51 {
                            video_cmd.arg("-crf").arg(video_settings.quality.to_string());
                        } else {
                            video_cmd.arg("-crf").arg("23"); // Default quality
                        }
                        
                        // Preset parameters
                        if video_settings.preset != "auto" && !video_settings.preset.is_empty() {
                            video_cmd.arg("-preset").arg(&video_settings.preset);
                        }
                    },
                    "libvpx" | "libvpx-vp9" => {
                        // VP8/VP9 use CRF or bitrate
                        if video_settings.crf > 0 && video_settings.crf <= 63 {
                            video_cmd.arg("-crf").arg(video_settings.crf.to_string());
                        } else if video_settings.bitrate != "auto" && !video_settings.bitrate.is_empty() {
                            video_cmd.arg("-b:v").arg(&video_settings.bitrate);
                        }
                    },
                    _ => {
                        // Other encoders use bitrate
                        if video_settings.bitrate != "auto" && !video_settings.bitrate.is_empty() {
                            video_cmd.arg("-b:v").arg(&video_settings.bitrate);
                        }
                    }
                }
            }
        }
        video_cmd.arg(&video_output);
        
        // Extract audio (no video)
        let mut audio_cmd = create_ffmpeg_command()?;
        audio_cmd.arg("-i").arg(input_file);
        audio_cmd.arg("-y"); // Overwrite output file
        audio_cmd.arg("-v").arg("error");
        audio_cmd.arg("-hide_banner");
        audio_cmd.arg("-nostats");
        audio_cmd.arg("-nostdin");
        audio_cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        
        audio_cmd.arg("-vn"); // Disable video
        if audio_codec == "copy" {
            audio_cmd.arg("-c:a").arg("copy");
        } else {
            audio_cmd.arg("-c:a").arg(&audio_codec);
            
            // Add quality parameters based on encoder
            if let Some(audio_settings) = &task.audio_settings {
                match audio_codec.as_str() {
                    "libmp3lame" | "mp3" => {
                        // MP3 quality control
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            audio_cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        } else {
                            audio_cmd.arg("-q:a").arg("2"); // VBR quality 2 (approx 190kbps)
                        }
                    },
                    "aac" | "libfdk_aac" => {
                        // AAC quality control
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            audio_cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        } else {
                            audio_cmd.arg("-b:a").arg("128k"); // Default 128kbps
                        }
                    },
                    "flac" => {
                        // FLAC compression level
                        audio_cmd.arg("-compression_level").arg("5"); // Default compression level
                    },
                    "libvorbis" | "vorbis" => {
                        // Vorbis quality control
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            audio_cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        } else {
                            audio_cmd.arg("-q:a").arg("5"); // VBR quality 5 (approx 160kbps)
                        }
                    },
                    "libopus" | "opus" => {
                        // Opus bitrate control
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            audio_cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        } else {
                            audio_cmd.arg("-b:a").arg("128k"); // Default 128kbps
                        }
                    },
                    "pcm_s16le" | "pcm_s24le" | "pcm_s32le" => {
                        // PCM lossless audio, no additional parameters needed
                    },
                    _ => {
                        // Other encoders use bitrate
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            audio_cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        }
                    }
                }
                
                // Sample rate settings
                if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
                    audio_cmd.arg("-ar").arg(&audio_settings.sample_rate);
                }
                
                // Channel settings
                if audio_settings.channels != "auto" && !audio_settings.channels.is_empty() {
                    audio_cmd.arg("-ac").arg(&audio_settings.channels);
                }
            }
        }
        audio_cmd.arg(&audio_output);
        
        // Execute video extraction
        log_info!("Extracting video without audio...");
        let video_output_result = video_cmd.output()?;
        if !video_output_result.status.success() {
            let stderr = String::from_utf8_lossy(&video_output_result.stderr);
            return Err(anyhow::anyhow!("Video extraction failed: {}", stderr));
        }
        
        // Execute audio extraction
        log_info!("Extracting audio without video...");
        let audio_output_result = audio_cmd.output()?;
        if !audio_output_result.status.success() {
            let stderr = String::from_utf8_lossy(&audio_output_result.stderr);
            return Err(anyhow::anyhow!("Audio extraction failed: {}", stderr));
        }
        
        log_info!("Video/Audio split completed successfully!");
        log_info!("  Video: {}", video_output.display());
        log_info!("  Audio: {}", audio_output.display());
        
        Ok(())
    }

    fn execute_extract_audio(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart audio codec selection
        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&output_ext)
            } else if audio_settings.codec == "copy" {
                "copy".to_string()
            } else {
                audio_settings.codec.clone()
            }
        } else {
            // Try to copy, convert if incompatible
            "copy".to_string()
        };

        // Always use bundled FFmpeg approach

        // Fallback to external FFmpeg command


        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y"); // Overwrite output file
        cmd.arg("-v").arg("error"); // Show error level information
        cmd.arg("-hide_banner"); // Hide banner information
        cmd.arg("-nostats"); // Disable statistics
        cmd.arg("-nostdin"); // Disable interactive input
        
        // Set environment variables
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        cmd.arg("-vn"); // Disable video
        
        if audio_codec == "copy" {
            cmd.arg("-c:a").arg("copy");
        } else {
            cmd.arg("-c:a").arg(&audio_codec);
            
            // Add audio parameters
            if let Some(audio_settings) = &task.audio_settings {
                match output_ext.as_str() {
                    "mp3" => {
                        let bitrate = if audio_settings.bitrate == "auto" || audio_settings.bitrate.is_empty() {
                            "128k"
                        } else {
                            &audio_settings.bitrate
                        };
                        cmd.arg("-b:a").arg(bitrate);
                    },
                    "aac" | "m4a" => {
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        }
                    },
                    "flac" => {
                        // FLAC doesn't need bitrate setting
                    },
                    _ => {
                        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                            cmd.arg("-b:a").arg(&audio_settings.bitrate);
                        }
                    }
                }
                
                if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
                    cmd.arg("-ar").arg(&audio_settings.sample_rate);
                }
                
                if audio_settings.channels != "auto" && !audio_settings.channels.is_empty() {
                    cmd.arg("-ac").arg(&audio_settings.channels);
                }
            }
        }
        
        cmd.arg(output_file);
        
        // Use common FFmpeg execution function
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_extract_video(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart video codec selection
        let video_codec = if let Some(video_settings) = &task.video_settings {
            if video_settings.codec == "auto" {
                CodecManager::get_best_video_codec_for_format(&output_ext)
            } else if video_settings.codec == "copy" {
                "copy".to_string()
            } else {
                video_settings.codec.clone()
            }
        } else {
            "copy".to_string()
        };

        
        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y"); // Overwrite output file
        cmd.arg("-v").arg("error"); // Show error level information
        cmd.arg("-hide_banner"); // Hide banner information
        cmd.arg("-nostats"); // Disable statistics
        cmd.arg("-nostdin"); // Disable interactive input
        
        // Set environment variables
        cmd.env("AV_LOG_FORCE_NOCOLOR", "1");
        cmd.arg("-an"); // Disable audio
        
        if video_codec == "copy" {
            cmd.arg("-c:v").arg("copy");
        } else {
            cmd.arg("-c:v").arg(&video_codec);
            
            // Add video parameters
            if let Some(video_settings) = &task.video_settings {
                if video_settings.quality > 0 && video_settings.quality <= 51 {
                    match video_codec.as_str() {
                        "libx264" | "libx265" => {
                            cmd.arg("-crf").arg(video_settings.quality.to_string());
                        },
                        _ => {
                            let bitrate = if video_settings.bitrate == "auto" {
                                Self::crf_to_bitrate_estimate(video_settings.quality)
                            } else {
                                video_settings.bitrate.clone()
                            };
                            cmd.arg("-b:v").arg(bitrate);
                        }
                    }
                } else if video_settings.bitrate != "auto" {
                    cmd.arg("-b:v").arg(&video_settings.bitrate);
                }
                
                if video_settings.resolution.0 > 0 && video_settings.resolution.1 > 0 {
                    cmd.arg("-s").arg(format!("{}x{}", video_settings.resolution.0, video_settings.resolution.1));
                }
                
                if video_settings.fps != "auto" {
                    cmd.arg("-r").arg(&video_settings.fps);
                }
            }
        }
        
        cmd.arg(output_file);
        
        // Use common FFmpeg execution function
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    // Preview command generation methods
    fn preview_audio_convert(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // auto codec
        let target_format = if !audio_settings.format.is_empty() && audio_settings.format != "auto" {
            &audio_settings.format
        } else {
            &output_ext
        };
        
        let mut codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(target_format)
        } else {
            audio_settings.codec.clone()
        };

        // Validate encoder and format compatibility, silent fix
        if CodecManager::validate_codec_format_compatibility(&codec, target_format, true).is_err() {
            codec = CodecManager::get_best_audio_codec_for_format(target_format);
        }

        // Normalize output file path
        let normalized_output = Self::normalize_output_path_with_container(output_file, &codec, target_format, true);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push(codec.clone());

        // Auto fill para
        match target_format.as_ref() {
            "mp3" => {
                let bitrate = if audio_settings.bitrate == "auto" || audio_settings.bitrate.is_empty() {
                    "128k"
                } else {
                    &audio_settings.bitrate
                };
                cmd_parts.push("-b:a".to_string());
                cmd_parts.push(bitrate.to_string());
                
                let sample_rate = if audio_settings.sample_rate == "auto" || audio_settings.sample_rate.is_empty() {
                    "44100"
                } else {
                    &audio_settings.sample_rate
                };
                cmd_parts.push("-ar".to_string());
                cmd_parts.push(sample_rate.to_string());
            },
            "aac" | "m4a" => {
                if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                    cmd_parts.push("-b:a".to_string());
                    cmd_parts.push(audio_settings.bitrate.clone());
                }
                if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
                    cmd_parts.push("-ar".to_string());
                    cmd_parts.push(audio_settings.sample_rate.clone());
                }
            },
            _ => {
                if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
                    cmd_parts.push("-b:a".to_string());
                    cmd_parts.push(audio_settings.bitrate.clone());
                }
                if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
                    cmd_parts.push("-ar".to_string());
                    cmd_parts.push(audio_settings.sample_rate.clone());
                }
            }
        }

        // Channel settings
        if audio_settings.channels != "auto" && !audio_settings.channels.is_empty() {
            cmd_parts.push("-ac".to_string());
            cmd_parts.push(audio_settings.channels.clone());
        }

        // Volume adjustment
        if (audio_settings.volume - 1.0).abs() > 0.01 {
            cmd_parts.push("-filter:a".to_string());
            cmd_parts.push(format!("volume={}", audio_settings.volume));
        }

        // Add user custom parameters
        if !audio_settings.custom_args.is_empty() {
            let custom_args: Vec<&str> = audio_settings.custom_args.split_whitespace().collect();
            for arg in custom_args {
                cmd_parts.push(arg.to_string());
            }
        }

        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_video_convert(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        // Get output format
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Smart video codec selection
        let mut video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        // Validate encoder and format compatibility, silent fix
        if CodecManager::validate_codec_format_compatibility(&video_codec, &output_ext, false).is_err() {
            video_codec = CodecManager::get_best_video_codec_for_format(&output_ext);
        }

        // Normalize output file path, prioritize user-specified container format
        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, &video_settings.container_format, false);

        // Smart audio codec selection
        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&output_ext)
            } else if audio_settings.codec == "copy" {
                "copy".to_string()
            } else {
                audio_settings.codec.clone()
            }
        } else {
            "copy".to_string()
        };

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec.clone());

        // Basic video parameters
        if video_settings.quality > 0 && video_settings.quality <= 51 {
            match video_codec.as_str() {
                "libx264" | "libx265" => {
                    cmd_parts.push("-crf".to_string());
                    cmd_parts.push(video_settings.quality.to_string());
                },
                _ => {
                    let bitrate = if video_settings.bitrate == "auto" {
                        Self::crf_to_bitrate_estimate(video_settings.quality)
                    } else {
                        video_settings.bitrate.clone()
                    };
                    cmd_parts.push("-b:v".to_string());
                    cmd_parts.push(bitrate);
                }
            }
        } else if video_settings.bitrate != "auto" {
            cmd_parts.push("-b:v".to_string());
            cmd_parts.push(video_settings.bitrate.clone());
        }

        // Preset settings
        if video_settings.preset != "auto" {
            match video_codec.as_str() {
                "libx264" | "libx265" => {
                    cmd_parts.push("-preset".to_string());
                    cmd_parts.push(video_settings.preset.clone());
                },
                _ => {}
            }
        }

        // Resolution settings
        if video_settings.resolution.0 > 0 && video_settings.resolution.1 > 0 {
            cmd_parts.push("-s".to_string());
            cmd_parts.push(format!("{}x{}", video_settings.resolution.0, video_settings.resolution.1));
        }

        // Frame rate settings
        if video_settings.fps != "auto" {
            cmd_parts.push("-r".to_string());
            cmd_parts.push(video_settings.fps.clone());
        }

        // Audio processing
        if audio_codec == "copy" {
            cmd_parts.push("-c:a".to_string());
            cmd_parts.push("copy".to_string());
        } else {
            cmd_parts.push("-c:a".to_string());
            cmd_parts.push(audio_codec);
        }

        // Add user custom parameters
        if !video_settings.custom_args.is_empty() {
            let custom_args: Vec<&str> = video_settings.custom_args.split_whitespace().collect();
            for arg in custom_args {
                cmd_parts.push(arg.to_string());
            }
        }

        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_video_audio_merge(task: &ProcessingTask) -> Result<String> {
        if task.input_files.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 files for video/audio merge"));
        }

        let video_file = &task.input_files[0];
        let audio_file = &task.input_files[1];
        let output_file = &task.output_file;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&output_ext)
            } else {
                audio_settings.codec.clone()
            }
        } else {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        };

        // Normalize output file path
        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", video_file));
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", audio_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push(audio_codec);
        cmd_parts.push("-map".to_string());
        cmd_parts.push("0:v:0".to_string());
        cmd_parts.push("-map".to_string());
        cmd_parts.push("1:a:0".to_string());
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_extract_audio(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if let Some(audio_settings) = &task.audio_settings {
            if audio_settings.codec == "auto" {
                CodecManager::get_best_audio_codec_for_format(&output_ext)
            } else if audio_settings.codec == "copy" {
                "copy".to_string()
            } else {
                audio_settings.codec.clone()
            }
        } else {
            "copy".to_string()
        };

        // Normalize output file path
        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-vn".to_string());
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push(audio_codec);
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_extract_video(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if let Some(video_settings) = &task.video_settings {
            if video_settings.codec == "auto" {
                CodecManager::get_best_video_codec_for_format(&output_ext)
            } else if video_settings.codec == "copy" {
                "copy".to_string()
            } else {
                video_settings.codec.clone()
            }
        } else {
            "copy".to_string()
        };

        // Normalize output file path
        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-an".to_string());
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    // Preview command generation for new operations

    fn preview_video_crop(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());

        // Video crop filter
        if video_settings.resolution.0 > 0 && video_settings.resolution.1 > 0 {
            cmd_parts.push("-filter:v".to_string());
            cmd_parts.push(format!("crop={}:{}", video_settings.resolution.0, video_settings.resolution.1));
        } else {
            cmd_parts.push("-filter:v".to_string());
            cmd_parts.push("crop=iw*0.8:ih*0.8:(iw-ow)/2:(ih-oh)/2".to_string());
        }

        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_video_rotate(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        // Build video filters for rotation and flipping (same logic as execute)
        let mut filters = Vec::new();
        
        // Handle rotation
        if video_settings.use_custom_rotation {
            // Use custom rotation angle with rotate filter
            if video_settings.custom_rotation_angle != 0.0 {
                // Convert degrees to radians for FFmpeg rotate filter
                let radians = video_settings.custom_rotation_angle * std::f32::consts::PI / 180.0;
                filters.push(format!("rotate={}", radians));
            }
        } else {
            // Use preset rotation angles with transpose filter (more efficient for 90-degree increments)
            match video_settings.rotation {
                90 => filters.push("transpose=1".to_string()),        // 90° clockwise
                180 => filters.push("transpose=2,transpose=2".to_string()), // 180°
                270 => filters.push("transpose=2".to_string()),       // 270° clockwise (90° counter-clockwise)
                _ => {} // No rotation for 0 or other values
            }
        }
        
        // Handle flipping
        if video_settings.flip_horizontal && video_settings.flip_vertical {
            filters.push("hflip,vflip".to_string());
        } else if video_settings.flip_horizontal {
            filters.push("hflip".to_string());
        } else if video_settings.flip_vertical {
            filters.push("vflip".to_string());
        }

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        
        // Apply filters if any
        if !filters.is_empty() {
            cmd_parts.push("-filter:v".to_string());
            cmd_parts.push(format!("\"{}\"", filters.join(",")));
        }
        
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_video_filter(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let filter = if !video_settings.custom_args.is_empty() {
            video_settings.custom_args.clone()
        } else {
            "unsharp=5:5:1.0:5:5:0.0,eq=contrast=1.1:brightness=0.05:saturation=1.1".to_string()
        };

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-filter:v".to_string());
        cmd_parts.push(filter);
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_audio_trim(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        } else {
            audio_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));

        // Audio trim parameters
        if !audio_settings.custom_args.is_empty() {
            let parts: Vec<&str> = audio_settings.custom_args.split(',').collect();
            if parts.len() >= 2 {
                cmd_parts.push("-ss".to_string());
                cmd_parts.push(parts[0].to_string());
                cmd_parts.push("-t".to_string());
                cmd_parts.push(parts[1].to_string());
            } else if parts.len() == 1 {
                cmd_parts.push("-t".to_string());
                cmd_parts.push(parts[0].to_string());
            }
        } else {
            cmd_parts.push("-t".to_string());
            cmd_parts.push("30".to_string());
        }

        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push(audio_codec);
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_audio_merge(task: &ProcessingTask) -> Result<String> {
        if task.input_files.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 files for audio merge"));
        }

        let output_file = &task.output_file;
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        } else {
            audio_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        
        for input_file in &task.input_files {
            cmd_parts.push("-i".to_string());
            cmd_parts.push(format!("\"{}\"", input_file));
        }

        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());

        let filter_complex = format!("{}concat=n={}:v=0:a=1[out]", 
            (0..task.input_files.len()).map(|i| format!("[{}:0]", i)).collect::<Vec<_>>().join(""),
            task.input_files.len()
        );
        cmd_parts.push("-filter_complex".to_string());
        cmd_parts.push(filter_complex);
        cmd_parts.push("-map".to_string());
        cmd_parts.push("[out]".to_string());
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push(audio_codec);
        cmd_parts.push(format!("\"{}\"", normalized_output));

        Ok(cmd_parts.join(" "))
    }

    fn preview_batch_convert(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let output_file = &task.output_file;
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut preview_lines = Vec::new();
        preview_lines.push("# Batch conversion will execute the following commands:".to_string());
        preview_lines.push("".to_string());

        for (index, input_file) in task.input_files.iter().enumerate() {
            let input_path = std::path::Path::new(input_file);
            let input_stem = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            
            let auto_output = if let Some(parent) = std::path::Path::new(output_file).parent() {
                parent.join(format!("{}_{}.{}", input_stem, index + 1, output_ext))
            } else {
                std::path::PathBuf::from(format!("{}_{}.{}", input_stem, index + 1, output_ext))
            };

            preview_lines.push(format!("# File {} of {}:", index + 1, task.input_files.len()));
            preview_lines.push(format!("ffmpeg -i \"{}\" -y -v info -hide_banner -nostats \"{}\"", 
                input_file, auto_output.display()));
            preview_lines.push("".to_string());
        }

        Ok(preview_lines.join("\n"))
    }

    fn preview_add_subtitle(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input video file specified"));
        }

        let video_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        if video_settings.subtitle_file.is_empty() {
            return Err(anyhow::anyhow!("No subtitle file specified"));
        }

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = CodecManager::get_best_video_codec_for_format(&output_ext);

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", video_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        
        // Get subtitle file from input_files if available, otherwise from video_settings
        let subtitle_file = if task.input_files.len() >= 2 {
            &task.input_files[1]
        } else {
            &video_settings.subtitle_file
        };
        
        // Escape the subtitle file path for the subtitles filter
        // Convert Windows paths to use forward slashes for FFmpeg compatibility
        let escaped_subtitle_path = subtitle_file.replace("\\", "/");
        
        if video_settings.subtitle_mode == "hard" {
            // Hard subtitle - burn into video with style
            let mut filter_parts = vec![format!("subtitles={}", escaped_subtitle_path)];
            
            // Add font style if specified
            if video_settings.subtitle_font_family != "Arial" {
                filter_parts.push(format!("force_style='FontName={}'", video_settings.subtitle_font_family));
            }
            if video_settings.subtitle_font_size != 16 {
                filter_parts.push(format!("force_style='FontSize={}'", video_settings.subtitle_font_size));
            }
            
            // Convert color names to ASS format
            let color_code = match video_settings.subtitle_font_color.as_str() {
                "white" => "&Hffffff&",
                "black" => "&H000000&",
                "red" => "&H0000ff&",
                "blue" => "&Hff0000&",
                "green" => "&H00ff00&",
                "yellow" => "&H00ffff&",
                _ => "&Hffffff&",
            };
            if color_code != "&Hffffff&" {
                filter_parts.push(format!("force_style='PrimaryColour={}'", color_code));
            }
            
            let filter = if filter_parts.len() > 1 {
                filter_parts.join(":")
            } else {
                filter_parts[0].clone()
            };
            
            cmd_parts.push("-filter:v".to_string());
            cmd_parts.push(filter);
        } else {
            // Soft subtitle - add as stream
            cmd_parts.push("-i".to_string());
            cmd_parts.push(format!("\"{}\"", subtitle_file));
            cmd_parts.push("-c:s".to_string());
            cmd_parts.push("copy".to_string());
        }
        
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push(format!("\"{}\"", output_file));

        Ok(cmd_parts.join(" "))
    }

    fn preview_add_watermark(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input video file specified"));
        }

        let video_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        if video_settings.watermark_file.is_empty() {
            return Err(anyhow::anyhow!("No watermark file specified"));
        }
        
        // Check if watermark file exists
        if !std::path::Path::new(&video_settings.watermark_file).exists() {
            return Err(anyhow::anyhow!("Watermark file not found: {}", video_settings.watermark_file));
        }

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = CodecManager::get_best_video_codec_for_format(&output_ext);

        // Build watermark position
        let position = match video_settings.watermark_position.as_str() {
            "top-left" => "10:10",
            "top-right" => "main_w-overlay_w-10:10",
            "bottom-left" => "10:main_h-overlay_h-10",
            "bottom-right" => "main_w-overlay_w-10:main_h-overlay_h-10",
            "center" => "(main_w-overlay_w)/2:(main_h-overlay_h)/2",
            _ => "main_w-overlay_w-10:10"
        };
        
        // Build a proper filter_complex chain for watermark preview
        // Handle scaling and opacity
        let opacity = video_settings.watermark_opacity;
        let scale = video_settings.watermark_scale;
        
        let filter_complex = if (scale - 1.0).abs() > 0.01 || (opacity - 1.0).abs() > 0.01 {
            // Apply scaling and/or opacity
            if (opacity - 1.0).abs() > 0.01 {
                // Apply opacity using alpha channel manipulation
                format!(
                    "[1:v]scale=iw*{}:ih*{},format=rgba,colorchannelmixer=aa={}[wm];[0:v][wm]overlay={}",
                    scale, scale, opacity, position
                )
            } else {
                // Only scale, no opacity change
                format!(
                    "[1:v]scale=iw*{}:ih*{}[wm];[0:v][wm]overlay={}",
                    scale, scale, position
                )
            }
        } else {
            // No scaling or opacity change
            format!("[0:v][1:v]overlay={}", position)
        };

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", video_file));
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", video_settings.watermark_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-filter_complex".to_string());
        cmd_parts.push(format!("\"{}\"", filter_complex));
        cmd_parts.push("-c:v".to_string());
        cmd_parts.push(video_codec);
        cmd_parts.push("-c:a".to_string());
        cmd_parts.push("copy".to_string());
        cmd_parts.push(format!("\"{}\"", output_file));

        Ok(cmd_parts.join(" "))
    }

    fn preview_frame_extract(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("error".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        cmd_parts.push("-nostdin".to_string());

        // Handle different extraction modes (preview)
        match video_settings.frame_extract_mode.as_str() {
            "all" => {
                // Extract all frames - no fps filter, just extract every frame
                cmd_parts.push("-vsync".to_string());
                cmd_parts.push("0".to_string());
            },
            "interval" => {
                // Extract every N frames using select filter for more precision
                if video_settings.frame_interval > 0 {
                    cmd_parts.push("-vf".to_string());
                    cmd_parts.push(format!("\"select='not(mod(n,{}))'\"", video_settings.frame_interval));
                    cmd_parts.push("-vsync".to_string());
                    cmd_parts.push("0".to_string());
                } else {
                    cmd_parts.push("-vf".to_string());
                    cmd_parts.push("\"select='not(mod(n,30))'\"".to_string());
                    cmd_parts.push("-vsync".to_string());
                    cmd_parts.push("0".to_string());
                }
            },
            "time" => {
                // Extract from time range
                if !video_settings.frame_start_time.is_empty() {
                    cmd_parts.push("-ss".to_string());
                    cmd_parts.push(video_settings.frame_start_time.clone());
                }
                if !video_settings.frame_end_time.is_empty() {
                    cmd_parts.push("-to".to_string());
                    cmd_parts.push(video_settings.frame_end_time.clone());
                }
                // Extract one frame per second in the time range
                cmd_parts.push("-vf".to_string());
                cmd_parts.push("fps=1".to_string());
            },
            _ => {
                // Default: extract every 30th frame
                cmd_parts.push("-vf".to_string());
                cmd_parts.push("\"select='not(mod(n,30))'\"".to_string());
                cmd_parts.push("-vsync".to_string());
                cmd_parts.push("0".to_string());
            }
        }

        // Set quality based on format
        match video_settings.frame_format.as_str() {
            "jpg" | "jpeg" => {
                cmd_parts.push("-q:v".to_string());
                cmd_parts.push(video_settings.frame_quality.to_string());
            },
            "png" => {
                cmd_parts.push("-pix_fmt".to_string());
                cmd_parts.push("rgb24".to_string());
            },
            "bmp" => {
                cmd_parts.push("-pix_fmt".to_string());
                cmd_parts.push("bgr24".to_string());
            },
            _ => {
                cmd_parts.push("-q:v".to_string());
                cmd_parts.push("2".to_string());
            }
        }

        // Build proper output pattern for multiple frames (same logic as execute)
        let output_with_extension = if output_file.contains("%") {
            // User specified a pattern, use it
            output_file.to_string()
        } else {
            // Create a pattern for multiple frame files
            let output_path = std::path::Path::new(output_file);
            
            // Get directory
            let dir = if let Some(parent) = output_path.parent() {
                parent
            } else {
                std::path::Path::new(".")
            };
            
            // Get base filename without extension
            let base = output_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("frame");
            
            // Create pattern: dir/filename_001.ext, dir/filename_002.ext, etc.
            format!("{}/{}_%03d.{}", 
                dir.display(), 
                base, 
                video_settings.frame_format
            )
        };

        cmd_parts.push(format!("\"{}\"", output_with_extension));

        Ok(cmd_parts.join(" "))
    }

    // Implementation of new specific operations

    fn execute_video_crop(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Video crop filter - use resolution settings as crop parameters
        if video_settings.resolution.0 > 0 && video_settings.resolution.1 > 0 {
            cmd.arg("-filter:v").arg(format!("crop={}:{}", video_settings.resolution.0, video_settings.resolution.1));
        } else {
            // Default crop to centered 16:9 ratio
            cmd.arg("-filter:v").arg("crop=iw*0.8:ih*0.8:(iw-ow)/2:(ih-oh)/2");
        }

        cmd.arg("-c:v").arg(&video_codec);
        cmd.arg("-c:a").arg("copy");

        if video_settings.quality > 0 && video_settings.quality <= 51 {
            if video_codec == "libx264" || video_codec == "libx265" {
                cmd.arg("-crf").arg(video_settings.quality.to_string());
            }
        }

        cmd.arg(&normalized_output);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_video_rotate(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Build video filters for rotation and flipping
        let mut filters = Vec::new();
        
        // Handle rotation
        if video_settings.use_custom_rotation {
            // Use custom rotation angle with rotate filter
            if video_settings.custom_rotation_angle != 0.0 {
                // Convert degrees to radians for FFmpeg rotate filter
                let radians = video_settings.custom_rotation_angle * std::f32::consts::PI / 180.0;
                filters.push(format!("rotate={}", radians));
            }
        } else {
            // Use preset rotation angles with transpose filter (more efficient for 90-degree increments)
            match video_settings.rotation {
                90 => filters.push("transpose=1".to_string()),        // 90° clockwise
                180 => filters.push("transpose=2,transpose=2".to_string()), // 180°
                270 => filters.push("transpose=2".to_string()),       // 270° clockwise (90° counter-clockwise)
                _ => {} // No rotation for 0 or other values
            }
        }
        
        // Handle flipping
        if video_settings.flip_horizontal && video_settings.flip_vertical {
            filters.push("hflip,vflip".to_string());
        } else if video_settings.flip_horizontal {
            filters.push("hflip".to_string());
        } else if video_settings.flip_vertical {
            filters.push("vflip".to_string());
        }
        
        // Apply filters if any
        if !filters.is_empty() {
            cmd.arg("-filter:v").arg(filters.join(","));
        }
        cmd.arg("-c:v").arg(&video_codec);
        cmd.arg("-c:a").arg("copy");

        if video_settings.quality > 0 && video_settings.quality <= 51 {
            if video_codec == "libx264" || video_codec == "libx265" {
                cmd.arg("-crf").arg(video_settings.quality.to_string());
            }
        }

        cmd.arg(&normalized_output);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_video_filter(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = if video_settings.codec == "auto" {
            CodecManager::get_best_video_codec_for_format(&output_ext)
        } else {
            video_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &video_codec, "auto", false);

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Build video filter chain
        let mut filters = Vec::new();
        
        // Add basic filters
        if video_settings.denoise {
            filters.push("hqdn3d=4:3:6:4.5".to_string()); // Enhanced noise reduction effect
        }
        
        if video_settings.deinterlace {
            filters.push("yadif=1".to_string());
        }
        
        if video_settings.stabilize {
            filters.push("vidstabdetect=stepsize=6:shakiness=8:accuracy=9".to_string());
        }
        
        // Color and brightness adjustments - use more visible effects
        let mut color_adjustments = Vec::new();
        
        if (video_settings.brightness - 0.0).abs() > 0.01 {
            color_adjustments.push(format!("brightness={}", video_settings.brightness));
        }
        
        if (video_settings.contrast - 1.0).abs() > 0.01 {
            color_adjustments.push(format!("contrast={}", video_settings.contrast));
        }
        
        if (video_settings.saturation - 1.0).abs() > 0.01 {
            color_adjustments.push(format!("saturation={}", video_settings.saturation));
        }
        
        if !color_adjustments.is_empty() {
            filters.push(format!("eq={}", color_adjustments.join(":")));
        }
        
        // If there are custom parameters, add them at the end
        if !video_settings.custom_args.is_empty() {
            filters.push(video_settings.custom_args.clone());
        }
        
        if !filters.is_empty() {
            let filter_chain = filters.join(",");
            cmd.arg("-filter:v").arg(&filter_chain);
            log_info!("Applying video filters: {}", filter_chain);
        }
        cmd.arg("-c:v").arg(&video_codec);
        cmd.arg("-c:a").arg("copy");

        if video_settings.quality > 0 && video_settings.quality <= 51 {
            if video_codec == "libx264" || video_codec == "libx265" {
                cmd.arg("-crf").arg(video_settings.quality.to_string());
            }
        }

        cmd.arg(&normalized_output);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_audio_trim(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        } else {
            audio_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Audio trimming - extract time range from custom parameters, default to first 30 seconds
        if !audio_settings.custom_args.is_empty() {
            // Expected format: "start_time,duration" e.g. "10,30" means start at 10s, duration 30s
            let parts: Vec<&str> = audio_settings.custom_args.split(',').collect();
            if parts.len() >= 2 {
                cmd.arg("-ss").arg(parts[0]); // Start time
                cmd.arg("-t").arg(parts[1]);  // Duration
            } else if parts.len() == 1 {
                cmd.arg("-t").arg(parts[0]);  // Only specify duration, start from beginning
            }
        } else {
            cmd.arg("-t").arg("30"); // Default first 30 seconds
        }

        cmd.arg("-c:a").arg(&audio_codec);

        // Basic audio parameters
        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
            cmd.arg("-b:a").arg(&audio_settings.bitrate);
        }
        if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
            cmd.arg("-ar").arg(&audio_settings.sample_rate);
        }

        cmd.arg(&normalized_output);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_audio_merge(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 files for audio merge"));
        }

        let output_file = &task.output_file;
        let audio_settings = task.audio_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio settings specified"))?;

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let audio_codec = if audio_settings.codec == "auto" {
            CodecManager::get_best_audio_codec_for_format(&output_ext)
        } else {
            audio_settings.codec.clone()
        };

        let normalized_output = Self::normalize_output_path_with_container(output_file, &audio_codec, "auto", true);

        let mut cmd = create_ffmpeg_command()?;
        
        // Add all input files
        for input_file in &task.input_files {
            cmd.arg("-i").arg(input_file);
        }

        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Use concat filter to merge audio
        let filter_complex = format!("{}concat=n={}:v=0:a=1[out]", 
            (0..task.input_files.len()).map(|i| format!("[{}:0]", i)).collect::<Vec<_>>().join(""),
            task.input_files.len()
        );
        cmd.arg("-filter_complex").arg(&filter_complex);
        cmd.arg("-map").arg("[out]");

        cmd.arg("-c:a").arg(&audio_codec);

        if audio_settings.bitrate != "auto" && !audio_settings.bitrate.is_empty() {
            cmd.arg("-b:a").arg(&audio_settings.bitrate);
        }
        if audio_settings.sample_rate != "auto" && !audio_settings.sample_rate.is_empty() {
            cmd.arg("-ar").arg(&audio_settings.sample_rate);
        }

        cmd.arg(&normalized_output);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_batch_convert(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let output_file = &task.output_file;
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or(&video_settings.container_format)
            .to_lowercase();

        // Determine operation type based on batch_operation_type
        let operation_type = match video_settings.batch_operation_type.as_str() {
            "convert" => OperationType::VideoConvert,
            "compress" => OperationType::VideoCompress,
            "resize" => OperationType::VideoResize,
            "rotate" => OperationType::VideoRotate,
            "crop" => OperationType::VideoCrop,
            "filter" => OperationType::VideoFilter,
            _ => OperationType::VideoConvert, // Default fallback
        };

        // Process each input file
        for (index, input_file) in task.input_files.iter().enumerate() {
            let input_path = std::path::Path::new(input_file);
            let input_stem = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            
            // Generate output filename using batch naming pattern
            let output_name = video_settings.batch_naming_pattern
                .replace("{name}", input_stem)
                .replace("{index}", &(index + 1).to_string())
                .replace("{date}", &format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()));
            
            // Generate full output path
            let auto_output = if let Some(parent) = std::path::Path::new(output_file).parent() {
                parent.join(format!("{}.{}", output_name, output_ext))
            } else {
                std::path::PathBuf::from(format!("{}.{}", output_name, output_ext))
            };

            // Create individual file task
            let mut single_task = ProcessingTask {
                id: task.id + index * 1000, // Avoid ID conflicts
                operation: operation_type.clone(),
                input_files: vec![input_file.clone()],
                output_file: auto_output.display().to_string(),
                video_settings: task.video_settings.clone(),
                audio_settings: task.audio_settings.clone(),
                progress: 0.0,
                status: TaskStatus::Running,
                error_message: None,
                start_time: Some(std::time::Instant::now()),
                estimated_total_time: None,
                completion_time: None,
            };

            // Execute the corresponding operation
            match operation_type {
                OperationType::VideoConvert | OperationType::VideoCompress | OperationType::VideoResize => {
                    Self::execute_video_convert_with_progress(&mut single_task, None, None)?;
                },
                OperationType::VideoRotate => {
                    Self::execute_video_rotate(&mut single_task)?;
                },
                OperationType::VideoCrop => {
                    Self::execute_video_crop(&mut single_task)?;
                },
                OperationType::VideoFilter => {
                    Self::execute_video_filter(&mut single_task)?;
                },
                _ => {
                    return Err(anyhow::anyhow!("Unsupported batch operation type: {:?}", operation_type));
                }
            }

            log_info!("Batch processing: {} → {} completed", input_file, auto_output.display());
        }

        Ok(())
    }

    fn execute_add_subtitle(task: &mut ProcessingTask) -> Result<() > {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No video file specified"));
        }

        let video_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        // Get subtitle file from video settings
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;
        
        if video_settings.subtitle_file.is_empty() {
            return Err(anyhow::anyhow!("No subtitle file specified"));
        }
        
        let subtitle_file = &video_settings.subtitle_file;
        
        // Check if files exist
        if !std::path::Path::new(video_file).exists() {
            return Err(anyhow::anyhow!("Video file not found: {}", video_file));
        }
        if !std::path::Path::new(subtitle_file).exists() {
            return Err(anyhow::anyhow!("Subtitle file not found: {}", subtitle_file));
        }
        
        // Check if the video file is actually a video (not a subtitle file)
        let video_ext = std::path::Path::new(video_file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
            
        if ["srt", "ass", "vtt", "sub"].contains(&video_ext.as_str()) {
            return Err(anyhow::anyhow!("First input must be a video file, not a subtitle file"));
        }
        
        // Check subtitle format compatibility with output container
        let subtitle_ext = std::path::Path::new(subtitle_file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
            
        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Get subtitle mode from video settings (needed for compatibility check)
        let subtitle_mode = task.video_settings.as_ref()
            .map(|s| s.subtitle_mode.as_str())
            .unwrap_or("hard");
        
        // Check subtitle compatibility with output container
        let is_mp4_output = output_ext == "mp4";
        let is_incompatible_subtitle = subtitle_ext == "vtt" || subtitle_ext == "ass" || subtitle_ext == "srt";
        
        // Handle subtitle format compatibility issues
        let (actual_subtitle_file, needs_cleanup, final_output_file) = if is_mp4_output && is_incompatible_subtitle {
            if subtitle_mode == "soft" {
                // For soft subtitles with incompatible formats, change output to MKV
                let mkv_output = output_file.replace(".mp4", ".mkv");
                log_warn!("⚠ MP4 doesn't support {} subtitles. Changing output format to MKV for compatibility.", subtitle_ext.to_uppercase());
                (subtitle_file.to_string(), false, mkv_output)
            } else {
                // For hard subtitles, convert subtitle format if needed
                match subtitle_ext.as_str() {
                    "vtt" => {
                        // Convert WebVTT to SRT for hard subtitle compatibility
                        let temp_srt = subtitle_file.replace(".vtt", "_temp.srt");
                        log_info!("Converting WebVTT to SRT for MP4 hard subtitle compatibility...");
                        
                        let mut convert_cmd = create_ffmpeg_command()?;
                        convert_cmd.arg("-i").arg(subtitle_file);
                        convert_cmd.arg("-y");
                        convert_cmd.arg("-v").arg("error");
                        convert_cmd.arg("-hide_banner");
                        convert_cmd.arg("-nostats");
                        convert_cmd.arg("-nostdin");
                        convert_cmd.arg(&temp_srt);
                        
                        if let Err(e) = convert_cmd.output() {
                            return Err(anyhow::anyhow!("Failed to convert WebVTT to SRT: {}", e));
                        }
                        
                        (temp_srt, true, output_file.to_string())
                    }
                    "ass" => {
                        // For ASS subtitles with hard mode, change to MKV for better compatibility
                        let mkv_output = output_file.replace(".mp4", ".mkv");
                        log_warn!("⚠ ASS subtitles work better with MKV format. Changing output to MKV.");
                        (subtitle_file.to_string(), false, mkv_output)
                    }
                    "srt" => {
                        // For SRT subtitles with hard mode, they work fine but we'll use MKV for consistency
                        if subtitle_mode == "hard" {
                            // SRT hard subtitles work with MP4, so keep original format
                            (subtitle_file.to_string(), false, output_file.to_string())
                        } else {
                            // This should not happen as soft mode is handled above
                            let mkv_output = output_file.replace(".mp4", ".mkv");
                            log_warn!("⚠ MP4 doesn't support SRT soft subtitles. Changing output format to MKV.");
                            (subtitle_file.to_string(), false, mkv_output)
                        }
                    }
                    _ => (subtitle_file.to_string(), false, output_file.to_string())
                }
            }
        } else {
            (subtitle_file.to_string(), false, output_file.to_string())
        };

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(video_file);
        
        if subtitle_mode == "soft" {
            // Soft subtitle - add as separate stream
            cmd.arg("-i").arg(&actual_subtitle_file);
        }
        
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        if subtitle_mode == "hard" {
            // Hard subtitle - use the exact format you specified
            let subtitle_path = actual_subtitle_file
                .replace("\\", "\\\\")
                .replace(":", "\\:");
            cmd.arg("-vf").arg(format!("subtitles='{}'", subtitle_path));
            
            // Copy audio stream
            cmd.arg("-c:a").arg("copy");

            // Get video codec based on output format
            let output_ext = std::path::Path::new(output_file)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();

            let video_codec = CodecManager::get_best_video_codec_for_format(&output_ext);
            cmd.arg("-c:v").arg(&video_codec);
        } else {
            // Soft subtitle - preserve as separate stream
            cmd.arg("-c:v").arg("copy");
            cmd.arg("-c:a").arg("copy");
            cmd.arg("-c:s").arg("copy");
            cmd.arg("-map").arg("0");
            cmd.arg("-map").arg("1");
        }

        cmd.arg(&final_output_file);
        
        // Debug print the complete command and inputs
        let cmd_args: Vec<String> = cmd.get_args().map(|s| s.to_string_lossy().to_string()).collect();
        let full_command = format!("ffmpeg {}", cmd_args.join(" "));
        log_debug!("Complete FFmpeg command: {}", full_command);
        log_debug!("Video file: {}", video_file);
        log_debug!("Subtitle file: {}", actual_subtitle_file);
        if final_output_file != *output_file {
            log_info!("Output format changed: {} -> {}", output_file, final_output_file);
        }
        
        let result = Self::execute_ffmpeg_command_with_progress(cmd, None, None);
        
        // Clean up temporary file if it was created
        if needs_cleanup {
            let _ = std::fs::remove_file(&actual_subtitle_file);
        }
        
        result
    }

    fn execute_add_watermark(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input video file specified"));
        }

        let video_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        if video_settings.watermark_file.is_empty() {
            return Err(anyhow::anyhow!("No watermark file specified"));
        }
        
        // Check if watermark file exists
        if !std::path::Path::new(&video_settings.watermark_file).exists() {
            return Err(anyhow::anyhow!("Watermark file not found: {}", video_settings.watermark_file));
        }

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(video_file);
        cmd.arg("-i").arg(&video_settings.watermark_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Build watermark position
        let position = match video_settings.watermark_position.as_str() {
            "top-left" => "10:10",
            "top-right" => "main_w-overlay_w-10:10",
            "bottom-left" => "10:main_h-overlay_h-10",
            "bottom-right" => "main_w-overlay_w-10:main_h-overlay_h-10",
            "center" => "(main_w-overlay_w)/2:(main_h-overlay_h)/2",
            _ => "main_w-overlay_w-10:10"
        };
        
        // Build a proper filter_complex chain for watermark execution
        // Handle scaling and opacity
        let opacity = video_settings.watermark_opacity;
        let scale = video_settings.watermark_scale;
        
        let filter_complex = if (scale - 1.0).abs() > 0.01 || (opacity - 1.0).abs() > 0.01 {
            // Apply scaling and/or opacity
            if (opacity - 1.0).abs() > 0.01 {
                // Apply opacity using alpha channel manipulation
                format!(
                    "[1:v]scale=iw*{}:ih*{},format=rgba,colorchannelmixer=aa={}[wm];[0:v][wm]overlay={}",
                    scale, scale, opacity, position
                )
            } else {
                // Only scale, no opacity change
                format!(
                    "[1:v]scale=iw*{}:ih*{}[wm];[0:v][wm]overlay={}",
                    scale, scale, position
                )
            }
        } else {
            // No scaling or opacity change
            format!("[0:v][1:v]overlay={}", position)
        };
        
        cmd.arg("-filter_complex").arg(&filter_complex);
        cmd.arg("-c:a").arg("copy");

        let output_ext = std::path::Path::new(output_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let video_codec = CodecManager::get_best_video_codec_for_format(&output_ext);
        cmd.arg("-c:v").arg(&video_codec);

        cmd.arg(output_file);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }

    fn execute_frame_extract(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input files specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");

        // Handle different extraction modes
        match video_settings.frame_extract_mode.as_str() {
            "all" => {
                // Extract all frames - no fps filter, just extract every frame
                // Use select filter to get every frame
                cmd.arg("-vsync").arg("0");
            },
            "interval" => {
                // Extract every N frames using select filter for more precision
                if video_settings.frame_interval > 0 {
                    cmd.arg("-vf").arg(format!("select='not(mod(n,{}))'", video_settings.frame_interval));
                    cmd.arg("-vsync").arg("0");
                } else {
                    cmd.arg("-vf").arg("select='not(mod(n,30))'");
                    cmd.arg("-vsync").arg("0");
                }
            },
            "time" => {
                // Extract from time range
                if !video_settings.frame_start_time.is_empty() {
                    cmd.arg("-ss").arg(&video_settings.frame_start_time);
                }
                if !video_settings.frame_end_time.is_empty() {
                    cmd.arg("-to").arg(&video_settings.frame_end_time);
                }
                // Extract one frame per second in the time range
                cmd.arg("-vf").arg("fps=1");
            },
            _ => {
                // Default: extract every 30th frame
                cmd.arg("-vf").arg("select='not(mod(n,30))'");
                cmd.arg("-vsync").arg("0");
            }
        }

        // Set quality based on format
        match video_settings.frame_format.as_str() {
            "jpg" | "jpeg" => {
                cmd.arg("-q:v").arg(video_settings.frame_quality.to_string());
            },
            "png" => {
                cmd.arg("-pix_fmt").arg("rgb24");
            },
            "bmp" => {
                cmd.arg("-pix_fmt").arg("bgr24");
            },
            _ => {
                cmd.arg("-q:v").arg("2");
            }
        }

        // Build proper output pattern for multiple frames
        let output_with_extension = if output_file.contains("%") {
            // User specified a pattern, use it
            output_file.to_string()
        } else {
            // Create a pattern for multiple frame files
            let output_path = std::path::Path::new(output_file);
            
            // Get directory (create if needed)
            let dir = if let Some(parent) = output_path.parent() {
                parent
            } else {
                std::path::Path::new(".")
            };
            
            // Create output directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(dir) {
                log_warn!("Could not create output directory: {}", e);
            }
            
            // Get base filename without extension
            let base = output_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("frame");
            
            // Create pattern: dir/filename_001.ext, dir/filename_002.ext, etc.
            format!("{}/{}_%03d.{}", 
                dir.display(), 
                base, 
                video_settings.frame_format
            )
        };

        cmd.arg(&output_with_extension);
        
        // Debug logging
        log_debug!("Frame extraction command: {:?}", cmd);
        log_debug!("Output pattern: {}", output_with_extension);
        
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }
    
    // Video to GIF conversion
    fn preview_video_to_gif(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input video file specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        
        // Build filter for GIF conversion
        let mut filter_chain = Vec::new();
        
        // Basic processing chain
        if (video_settings.gif_scale - 1.0).abs() > 0.01 {
            filter_chain.push(format!("scale=iw*{}:ih*{}", video_settings.gif_scale, video_settings.gif_scale));
        }
        filter_chain.push(format!("fps={}", video_settings.gif_fps));
        
        if video_settings.gif_optimize {
            // Use two-pass approach for better quality
            let basic_chain = filter_chain.join(",");
            let palette_filter = format!("{}[x];[x]split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither={}", 
                basic_chain,
                video_settings.gif_colors,
                match video_settings.gif_dither.as_str() {
                    "bayer" => "bayer:bayer_scale=5",
                    "floyd_steinberg" => "floyd_steinberg", 
                    _ => "none",
                }
            );
            cmd_parts.push("-filter_complex".to_string());
            cmd_parts.push(format!("\"{}\"", palette_filter));
        } else {
            // Simple chain without palette optimization
            if !filter_chain.is_empty() {
                cmd_parts.push("-vf".to_string());
                cmd_parts.push(format!("\"{}\"", filter_chain.join(",")));
            }
        }
        
        // GIF-specific options
        if video_settings.gif_loop {
            cmd_parts.push("-loop".to_string());
            cmd_parts.push("0".to_string()); // 0 = infinite loop
        }
        
        cmd_parts.push(format!("\"{}\"", output_file));
        Ok(cmd_parts.join(" "))
    }
    
    fn execute_video_to_gif(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input video file specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");
        
        // Build filter for GIF conversion
        let mut filter_chain = Vec::new();
        
        // Basic processing chain
        if (video_settings.gif_scale - 1.0).abs() > 0.01 {
            filter_chain.push(format!("scale=iw*{}:ih*{}", video_settings.gif_scale, video_settings.gif_scale));
        }
        filter_chain.push(format!("fps={}", video_settings.gif_fps));
        
        if video_settings.gif_optimize {
            // Use two-pass approach for better quality
            let basic_chain = filter_chain.join(",");
            let palette_filter = format!("{}[x];[x]split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither={}", 
                basic_chain,
                video_settings.gif_colors,
                match video_settings.gif_dither.as_str() {
                    "bayer" => "bayer:bayer_scale=5",
                    "floyd_steinberg" => "floyd_steinberg", 
                    _ => "none",
                }
            );
            cmd.arg("-filter_complex").arg(palette_filter);
        } else {
            // Simple chain without palette optimization
            if !filter_chain.is_empty() {
                cmd.arg("-vf").arg(filter_chain.join(","));
            }
        }
        
        // GIF-specific options
        if video_settings.gif_loop {
            cmd.arg("-loop").arg("0"); // 0 = infinite loop
        }
        
        cmd.arg(output_file);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }
    
    // GIF resize
    fn preview_gif_resize(task: &ProcessingTask) -> Result<String> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input GIF file specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd_parts = vec!["ffmpeg".to_string()];
        cmd_parts.push("-i".to_string());
        cmd_parts.push(format!("\"{}\"", input_file));
        cmd_parts.push("-y".to_string());
        cmd_parts.push("-v".to_string());
        cmd_parts.push("info".to_string());
        cmd_parts.push("-hide_banner".to_string());
        cmd_parts.push("-nostats".to_string());
        
        // Build filter for GIF resize
        let mut filter_chain = Vec::new();
        
        // Scale filter
        if (video_settings.gif_scale - 1.0).abs() > 0.01 {
            // Use scale ratio
            filter_chain.push(format!("scale=iw*{}:ih*{}", video_settings.gif_scale, video_settings.gif_scale));
        } else if let (Some(width), Some(height)) = (video_settings.width, video_settings.height) {
            // Use specific dimensions
            if video_settings.maintain_aspect_ratio {
                filter_chain.push(format!("scale={}:{}:force_original_aspect_ratio=decrease", width, height));
            } else {
                filter_chain.push(format!("scale={}:{}", width, height));
            }
        }
        
        if video_settings.gif_optimize {
            // Use palette optimization with correct filter chain syntax
            let basic_chain = filter_chain.join(",");
            let palette_filter = if basic_chain.is_empty() {
                format!("split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither=floyd_steinberg", video_settings.gif_colors)
            } else {
                format!("{}[x];[x]split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither=floyd_steinberg", 
                    basic_chain, video_settings.gif_colors)
            };
            cmd_parts.push("-filter_complex".to_string());
            cmd_parts.push(format!("\"{}\"", palette_filter));
        } else {
            // Simple chain without palette optimization
            if !filter_chain.is_empty() {
                cmd_parts.push("-vf".to_string());
                cmd_parts.push(format!("\"{}\"", filter_chain.join(",")));
            }
        }
        
        cmd_parts.push(format!("\"{}\"", output_file));
        Ok(cmd_parts.join(" "))
    }
    
    fn execute_gif_resize(task: &mut ProcessingTask) -> Result<()> {
        if task.input_files.is_empty() {
            return Err(anyhow::anyhow!("No input GIF file specified"));
        }

        let input_file = &task.input_files[0];
        let output_file = &task.output_file;
        
        let video_settings = task.video_settings.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video settings specified"))?;

        let mut cmd = create_ffmpeg_command()?;
        cmd.arg("-i").arg(input_file);
        cmd.arg("-y");
        cmd.arg("-v").arg("error");
        cmd.arg("-hide_banner");
        cmd.arg("-nostats");
        cmd.arg("-nostdin");
        
        // Build filter for GIF resize
        let mut filter_chain = Vec::new();
        
        // Scale filter
        if (video_settings.gif_scale - 1.0).abs() > 0.01 {
            // Use scale ratio
            filter_chain.push(format!("scale=iw*{}:ih*{}", video_settings.gif_scale, video_settings.gif_scale));
        } else if let (Some(width), Some(height)) = (video_settings.width, video_settings.height) {
            // Use specific dimensions
            if video_settings.maintain_aspect_ratio {
                filter_chain.push(format!("scale={}:{}:force_original_aspect_ratio=decrease", width, height));
            } else {
                filter_chain.push(format!("scale={}:{}", width, height));
            }
        }
        
        if video_settings.gif_optimize {
            // Use palette optimization with correct filter chain syntax
            let basic_chain = filter_chain.join(",");
            let palette_filter = if basic_chain.is_empty() {
                format!("split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither=floyd_steinberg", video_settings.gif_colors)
            } else {
                format!("{}[x];[x]split[s0][s1];[s0]palettegen=max_colors={}[p];[s1][p]paletteuse=dither=floyd_steinberg", 
                    basic_chain, video_settings.gif_colors)
            };
            cmd.arg("-filter_complex").arg(palette_filter);
        } else {
            // Simple chain without palette optimization
            if !filter_chain.is_empty() {
                cmd.arg("-vf").arg(filter_chain.join(","));
            }
        }
        
        cmd.arg(output_file);
        Self::execute_ffmpeg_command_with_progress(cmd, None, None)
    }
}