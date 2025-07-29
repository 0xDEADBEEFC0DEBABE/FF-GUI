use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;
use anyhow::{Result, anyhow};

/// Manages bundled FFmpeg executables
pub struct BundledFFmpeg {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
}

impl BundledFFmpeg {
    /// Create a new BundledFFmpeg instance
    pub fn new() -> Result<Self> {
        let (ffmpeg_path, ffprobe_path) = Self::find_ffmpeg_executables()?;
        
        // Verify executables work
        Self::verify_executable(&ffmpeg_path)?;
        Self::verify_executable(&ffprobe_path)?;
        
        
        Ok(Self {
            ffmpeg_path,
            ffprobe_path,
        })
    }
    
    /// Find FFmpeg executables in the following order:
    /// 1. Same directory as executable (bundled)
    /// 2. Relative to executable (./ffmpeg-static/bin/)
    /// 3. Environment PATH (as fallback)
    fn find_ffmpeg_executables() -> Result<(PathBuf, PathBuf)> {
        // Get the directory where our executable is located
        let exe_path = env::current_exe()
            .map_err(|e| anyhow!("Failed to get current executable path: {}", e))?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| anyhow!("Failed to get executable directory"))?;
        
        // Strategy 1: Look for bundled executables in the same directory
        let bundled_ffmpeg = exe_dir.join("ffmpeg.exe");
        let bundled_ffprobe = exe_dir.join("ffprobe.exe");
        
        if bundled_ffmpeg.exists() && bundled_ffprobe.exists() {
            return Ok((bundled_ffmpeg, bundled_ffprobe));
        }
        
        // Strategy 2: Look for FFmpeg in relative ffmpeg-static directory
        let static_bin_dir = exe_dir.join("ffmpeg-static").join("bin");
        let static_ffmpeg = static_bin_dir.join("ffmpeg.exe");
        let static_ffprobe = static_bin_dir.join("ffprobe.exe");
        
        if static_ffmpeg.exists() && static_ffprobe.exists() {
            return Ok((static_ffmpeg, static_ffprobe));
        }
        
        // Strategy 3: Look in development environment
        let dev_static_dir = exe_dir.parent()
            .and_then(|p| p.parent()) // Go up two levels from target/release
            .map(|p| p.join("ffmpeg-static").join("bin"));
        
        if let Some(dev_dir) = dev_static_dir {
            let dev_ffmpeg = dev_dir.join("ffmpeg.exe");
            let dev_ffprobe = dev_dir.join("ffprobe.exe");
            
            if dev_ffmpeg.exists() && dev_ffprobe.exists() {
                return Ok((dev_ffmpeg, dev_ffprobe));
            }
        }
        
        // Strategy 4: Try PATH as last resort (but verify it works)
        if let Ok(path_ffmpeg) = which::which("ffmpeg") {
            if let Ok(path_ffprobe) = which::which("ffprobe") {
                return Ok((path_ffmpeg, path_ffprobe));
            }
        }
        
        Err(anyhow!(
            "FFmpeg executables not found. Please ensure ffmpeg.exe and ffprobe.exe are in the same directory as the application, or in the ffmpeg-static/bin/ subdirectory."
        ))
    }
    
    /// Verify that an executable exists and can be run
    fn verify_executable(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow!("Executable not found: {}", path.display()));
        }
        
        // Test if the executable can run
        let result = Command::new(path)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        
        match result {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err(anyhow!("Executable failed to run: {}", path.display())),
            Err(e) => Err(anyhow!("Failed to execute {}: {}", path.display(), e)),
        }
    }
    
    /// Get the path to the FFmpeg executable
    pub fn ffmpeg_path(&self) -> &Path {
        &self.ffmpeg_path
    }
    
    /// Get the path to the FFprobe executable
    pub fn ffprobe_path(&self) -> &Path {
        &self.ffprobe_path
    }
    
    /// Create a new FFmpeg command
    pub fn command(&self) -> Command {
        Command::new(&self.ffmpeg_path)
    }
    
    /// Create a new FFprobe command
    pub fn probe_command(&self) -> Command {
        Command::new(&self.ffprobe_path)
    }
    
    /// Run FFmpeg with the given arguments and return the result
    pub fn run_ffmpeg(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = self.command()
            .args(args)
            .output()
            .map_err(|e| anyhow!("Failed to execute FFmpeg: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFmpeg failed: {}", stderr));
        }
        
        Ok(output)
    }
    
    /// Run FFprobe with the given arguments and return the result
    pub fn run_ffprobe(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = self.probe_command()
            .args(args)
            .output()
            .map_err(|e| anyhow!("Failed to execute FFprobe: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FFprobe failed: {}", stderr));
        }
        
        Ok(output)
    }
    
    /// Get FFmpeg version information
    pub fn get_version(&self) -> Result<String> {
        let output = self.command()
            .arg("-version")
            .output()
            .map_err(|e| anyhow!("Failed to get FFmpeg version: {}", e))?;
        
        if output.status.success() {
            let version_text = String::from_utf8_lossy(&output.stdout);
            // Extract just the first line which contains the version
            if let Some(first_line) = version_text.lines().next() {
                Ok(first_line.to_string())
            } else {
                Ok(version_text.to_string())
            }
        } else {
            Err(anyhow!("Failed to get FFmpeg version"))
        }
    }
    
    /// Check if hardware acceleration is available (combines FFmpeg support with actual hardware)
    pub fn check_hardware_acceleration(&self) -> Vec<String> {
        let mut available_encoders = Vec::new();
        
        // Get actual hardware info
        let hardware_info = crate::hardware_detector::HardwareDetector::detect_hardware();
        
        // Check for all hardware encoders
        let hw_encoders = vec![
            // NVIDIA NVENC
            "h264_nvenc",
            "hevc_nvenc", 
            "av1_nvenc",
            "vp9_nvenc",
            
            // Intel QuickSync
            "h264_qsv",
            "hevc_qsv",
            "av1_qsv", 
            "vp9_qsv",
            
            // AMD AMF
            "h264_amf",
            "hevc_amf",
            "av1_amf",
            
            // VA-API (Linux)
            "h264_vaapi",
            "hevc_vaapi",
            "av1_vaapi",
            "vp8_vaapi",
            "vp9_vaapi",
            "mjpeg_vaapi",
            "mpeg2_vaapi",
            
            // Apple VideoToolbox (macOS)
            "h264_videotoolbox",
            "hevc_videotoolbox", 
            "prores_videotoolbox",
        ];
        
        // Get FFmpeg supported encoders
        let ffmpeg_supported = if let Ok(output) = self.command()
            .args(&["-hide_banner", "-encoders"])
            .output() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        };
        
        for encoder in hw_encoders {
            // Check if FFmpeg supports this encoder
            if ffmpeg_supported.contains(encoder) {
                // Check if actual hardware supports this encoder
                let hardware_supports = match encoder {
                    // NVIDIA NVENC encoders
                    "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" | "vp9_nvenc" => {
                        hardware_info.gpus.iter().any(|gpu| gpu.supports_nvenc)
                    },
                    
                    // Intel QuickSync encoders
                    "h264_qsv" | "hevc_qsv" | "av1_qsv" | "vp9_qsv" => {
                        hardware_info.cpu.supports_quicksync || 
                        hardware_info.gpus.iter().any(|gpu| gpu.supports_qsv)
                    },
                    
                    // AMD AMF encoders  
                    "h264_amf" | "hevc_amf" | "av1_amf" => {
                        hardware_info.gpus.iter().any(|gpu| gpu.supports_amf)
                    },
                    
                    // VA-API encoders (Linux) - assume available if on Linux
                    "h264_vaapi" | "hevc_vaapi" | "av1_vaapi" | "vp8_vaapi" | "vp9_vaapi" | 
                    "mjpeg_vaapi" | "mpeg2_vaapi" => {
                        #[cfg(target_os = "linux")]
                        {
                            // On Linux, VA-API depends on driver support, assume true if FFmpeg supports it
                            true
                        }
                        #[cfg(not(target_os = "linux"))]
                        {
                            false
                        }
                    },
                    
                    // Apple VideoToolbox encoders (macOS) - assume available if on macOS
                    "h264_videotoolbox" | "hevc_videotoolbox" | "prores_videotoolbox" => {
                        #[cfg(target_os = "macos")]
                        {
                            true
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            false
                        }
                    },
                    
                    _ => false,
                };
                
                if hardware_supports {
                    available_encoders.push(encoder.to_string());
                }
            }
        }
        
        available_encoders
    }
}

/// Global instance for easy access
use lazy_static::lazy_static;

lazy_static! {
    static ref BUNDLED_FFMPEG: BundledFFmpeg = BundledFFmpeg::new().expect("Failed to initialize bundled FFmpeg");
}

/// Get the global BundledFFmpeg instance
pub fn get_bundled_ffmpeg() -> Result<&'static BundledFFmpeg> {
    Ok(&*BUNDLED_FFMPEG)
}