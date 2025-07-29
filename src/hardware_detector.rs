use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub gpus: Vec<GpuInfo>,
    pub cpu: CpuInfo,
    pub supported_encoders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub architecture: Option<String>,
    pub driver_version: Option<String>,
    pub memory_mb: Option<u32>,
    pub supports_nvenc: bool,
    pub supports_amf: bool,
    pub supports_qsv: bool,
    pub supports_videotoolbox: bool,
    pub supports_vaapi: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    NVIDIA,
    AMD,
    Intel,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: CpuVendor,
    pub has_integrated_graphics: bool,
    pub supports_quicksync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CpuVendor {
    Intel,
    AMD,
    Other(String),
}

pub struct HardwareDetector;

impl HardwareDetector {
    /// Detect complete hardware information
    pub fn detect_hardware() -> HardwareInfo {
        
        let gpus = Self::detect_gpus();
        
        let cpu = Self::detect_cpu();
        
        let supported_encoders = Self::determine_supported_encoders(&gpus, &cpu);
        
        HardwareInfo {
            gpus,
            cpu,
            supported_encoders,
        }
    }
    
    /// Detect all GPUs
    fn detect_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            // Use wmic to get detailed GPU information
            match Command::new("wmic")
                .args(&["path", "win32_VideoController", "get", "Name,DriverVersion,AdapterRAM", "/format:csv"])
                .output() 
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    
                    for line in output_str.lines().skip(1) { // Skip header line
                        if line.trim().is_empty() { continue; }
                        
                        let parts: Vec<&str> = line.split(',').collect();
                        
                        if parts.len() >= 4 {
                            // wmic CSV format: Node,AdapterRAM,DriverVersion,Name
                            let name = parts[3].trim().to_string();
                            let driver_version = if parts[2].trim().is_empty() { None } else { Some(parts[2].trim().to_string()) };
                            let memory_str = parts[1].trim();
                            let memory_mb = memory_str.parse::<u64>().ok().map(|m| (m / 1024 / 1024) as u32);
                            
                            if !name.is_empty() && !name.to_lowercase().contains("microsoft") {
                                let gpu_info = Self::analyze_gpu(&name, driver_version, memory_mb);
                                gpus.push(gpu_info);
                            }
                        }
                    }
                },
                Err(_e) => {
                    // Silently fall back to simple detection
                }
            }
        }
        
        // If detection fails, fallback to simple detection
        if gpus.is_empty() {
            gpus = Self::detect_gpus_simple();
        }
        
        // Non-Windows platform GPU detection
        #[cfg(not(target_os = "windows"))]
        {
            gpus = Self::detect_gpus_unix();
        }
        
        gpus
    }
    
    /// Simple GPU detection (fallback method)
    fn detect_gpus_simple() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            match Command::new("wmic")
                .args(&["path", "win32_VideoController", "get", "name"])
                .output() 
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    
                    for line in output_str.lines() {
                        let name = line.trim();
                        
                        if !name.is_empty() && !name.to_lowercase().contains("name") && !name.to_lowercase().contains("microsoft") {
                            let gpu_info = Self::analyze_gpu(name, None, None);
                            gpus.push(gpu_info);
                        }
                    }
                },
                Err(_e) => {
                    // Silent fallback
                }
            }
        }
        gpus
    }
    
    /// Unix platform GPU detection (Linux/macOS)
    #[cfg(not(target_os = "windows"))]
    fn detect_gpus_unix() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        
        // Linux: Try multiple detection methods
        #[cfg(target_os = "linux")]
        {
            // Method 1: lspci
            if let Ok(output) = Command::new("lspci")
                .args(&["-nn"])
                .output() 
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                for line in output_str.lines() {
                    if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d") || line.to_lowercase().contains("display") {
                        // Extract GPU name
                        if let Some(start) = line.find(": ") {
                            let gpu_info = line[start + 2..].to_string();
                            if !gpu_info.is_empty() {
                                let gpu = Self::analyze_gpu(&gpu_info, None, None);
                                gpus.push(gpu);
                            }
                        }
                    }
                }
            }
            
            // Method 2: Check /proc/driver/nvidia/gpus (NVIDIA)
            if gpus.is_empty() {
                if let Ok(entries) = std::fs::read_dir("/proc/driver/nvidia/gpus") {
                    for entry in entries.flatten() {
                        if let Ok(info_path) = entry.path().join("information").canonicalize() {
                            if let Ok(content) = std::fs::read_to_string(info_path) {
                                for line in content.lines() {
                                    if line.starts_with("Model:") {
                                        let model = line.replace("Model:", "").trim().to_string();
                                        let gpu = Self::analyze_gpu(&model, None, None);
                                        gpus.push(gpu);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // macOS: Use system_profiler
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("system_profiler")
                .args(&["SPDisplaysDataType", "-json"])
                .output() 
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Simple JSON parsing (avoid dependency on serde_json)
                for line in output_str.lines() {
                    if line.contains("_name") || line.contains("spdisplays_device_name") {
                        if let Some(start) = line.find(":") {
                            let gpu_name = line[start + 1..]
                                .trim()
                                .trim_matches('"')
                                .trim_matches(',')
                                .to_string();
                            
                            if !gpu_name.is_empty() && gpu_name != "null" {
                                let gpu = Self::analyze_gpu(&gpu_name, None, None);
                                gpus.push(gpu);
                            }
                        }
                    }
                }
            }
        }
        gpus
    }
    
    /// Analyze GPU detailed information
    fn analyze_gpu(name: &str, driver_version: Option<String>, memory_mb: Option<u32>) -> GpuInfo {
        let name_lower = name.to_lowercase();
        
        let vendor = if name_lower.contains("nvidia") || name_lower.contains("geforce") || name_lower.contains("rtx") || name_lower.contains("gtx") {
            GpuVendor::NVIDIA
        } else if name_lower.contains("amd") || name_lower.contains("radeon") || name_lower.contains("rx") {
            GpuVendor::AMD
        } else if name_lower.contains("intel") || name_lower.contains("uhd") || name_lower.contains("iris") || name_lower.contains("xe") {
            GpuVendor::Intel
        } else {
            GpuVendor::Other(name.to_string())
        };
        
        let architecture = Self::detect_gpu_architecture(&name_lower, &vendor);
        
        // Determine encoder support based on GPU model and driver version
        let supports_nvenc = match vendor {
            GpuVendor::NVIDIA => Self::supports_nvenc(&name_lower, &driver_version),
            _ => false,
        };
        
        let supports_amf = match vendor {
            GpuVendor::AMD => Self::supports_amf(&name_lower),
            _ => false,
        };
        
        let supports_qsv = match vendor {
            GpuVendor::Intel => Self::supports_quicksync(&name_lower),
            _ => false,
        };
        
        GpuInfo {
            name: name.to_string(),
            vendor,
            architecture,
            driver_version,
            memory_mb,
            supports_nvenc,
            supports_amf,
            supports_qsv,
            supports_videotoolbox: false, // Not supported on Windows
            supports_vaapi: false, // Usually not used on Windows
        }
    }
    
    /// Detect GPU architecture
    fn detect_gpu_architecture(name_lower: &str, vendor: &GpuVendor) -> Option<String> {
        match vendor {
            GpuVendor::NVIDIA => {
                if name_lower.contains("rtx 40") { Some("Ada Lovelace".to_string()) }
                else if name_lower.contains("rtx 30") { Some("Ampere".to_string()) }
                else if name_lower.contains("rtx 20") || name_lower.contains("gtx 16") { Some("Turing".to_string()) }
                else if name_lower.contains("gtx 10") { Some("Pascal".to_string()) }
                else { None }
            },
            GpuVendor::AMD => {
                if name_lower.contains("rx 7") { Some("RDNA 3".to_string()) }
                else if name_lower.contains("rx 6") { Some("RDNA 2".to_string()) }
                else if name_lower.contains("rx 5") { Some("RDNA".to_string()) }
                else { None }
            },
            _ => None,
        }
    }
    
    /// Detect NVENC support
    fn supports_nvenc(name_lower: &str, driver_version: &Option<String>) -> bool {
        
        // RTX 50 series - Full support
        if name_lower.contains("rtx 50") { return true; }

        // RTX 40 series - Full support
        if name_lower.contains("rtx 40") { return true; }
        
        // RTX 30 series - Full support
        if name_lower.contains("rtx 30") { return true; }
        
        // RTX 20 series - Most support
        if name_lower.contains("rtx 20") { return true; }
        
        // GTX 16 series - Partial support
        if name_lower.contains("gtx 16") { return true; }
        
        // GTX 10 series - Basic support
        if name_lower.contains("gtx 10") { return true; }
        
        // Check driver version
        if let Some(version) = driver_version {
            if let Ok(version_num) = version.split('.').next().unwrap_or("0").parse::<u32>() {
                return version_num >= 460; // Requires newer driver
            }
        }
        
        false
    }
    
    /// Detect AMF support
    fn supports_amf(name_lower: &str) -> bool {
        // RX 7000 series - Full support
        if name_lower.contains("rx 7") { return true; }
        
        // RX 6000 series - Full support
        if name_lower.contains("rx 6") { return true; }
        
        // RX 5000 series - Basic support
        if name_lower.contains("rx 5") { return true; }
        
        // Vega series
        if name_lower.contains("vega") { return true; }
        
        // Integrated graphics
        if name_lower.contains("radeon") && (name_lower.contains("integrated") || name_lower.contains("graphics")) {
            return true;
        }
        
        false
    }
    
    /// Detect QuickSync support
    fn supports_quicksync(name_lower: &str) -> bool {
        // Intel Arc series - Full support
        if name_lower.contains("arc") { return true; }
        
        // UHD Graphics - Support
        if name_lower.contains("uhd") { return true; }
        
        // Iris series - Support
        if name_lower.contains("iris") { return true; }
        
        // Xe series - Support
        if name_lower.contains("xe") { return true; }
        
        true // Intel integrated graphics usually support QuickSync
    }
    
    /// Detect if CPU has integrated graphics by checking actual system devices
    fn detect_integrated_graphics(_vendor: &CpuVendor, _cpu_name_lower: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            Self::detect_integrated_graphics_windows()
        }
        
        #[cfg(target_os = "linux")]
        {
            Self::detect_integrated_graphics_linux()
        }
        
        #[cfg(target_os = "macos")]
        {
            Self::detect_integrated_graphics_macos()
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            false
        }
    }
    
    /// Parse a CSV line properly handling quoted values and commas within quotes
    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                },
                ',' if !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                },
                _ => {
                    current_field.push(ch);
                }
            }
        }
        
        // Add the last field
        fields.push(current_field.trim().to_string());
        fields
    }

    #[cfg(target_os = "windows")]
    fn detect_integrated_graphics_windows() -> bool {
        // Method 1: Check via Windows Device Manager (most reliable)
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-WmiObject -Class Win32_VideoController | Select-Object Name, VideoProcessor, AdapterCompatibility | ConvertTo-Json"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                let devices = if json.is_array() { json.as_array().unwrap() } else { &vec![json] };
                
                for device in devices {
                    if let (Some(name), Some(compatibility)) = (
                        device.get("Name").and_then(|v| v.as_str()),
                        device.get("AdapterCompatibility").and_then(|v| v.as_str())
                    ) {
                        let name_lower = name.to_lowercase();
                        let comp_lower = compatibility.to_lowercase();
                        
                        // Check for integrated graphics indicators
                        let is_integrated = 
                            // Intel integrated
                            (comp_lower.contains("intel") && (
                                name_lower.contains("uhd") || name_lower.contains("iris") || 
                                name_lower.contains("xe") || name_lower.contains("hd graphics")
                            )) ||
                            // AMD integrated (broader detection)
                            (comp_lower.contains("amd") || comp_lower.contains("advanced micro devices")) && (
                                (name_lower.contains("radeon") && name_lower.contains("graphics")) ||
                                name_lower.contains("vega") || name_lower.contains("rdna") ||
                                name_lower.contains("amd radeon(tm) graphics")
                            ) ||
                            // Generic integrated indicators
                            name_lower.contains("integrated") || name_lower.contains("onboard");
                        
                        if is_integrated {
                            return true;
                        }
                    }
                }
            }
        }
        
        // Method 2: Fallback using WMIC
        if let Ok(output) = Command::new("wmic")
            .args(&["path", "win32_VideoController", "get", "Name,PNPDeviceID,AdapterCompatibility", "/format:csv"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines().skip(1) {
                if line.trim().is_empty() { continue; }
                
                // Parse CSV properly handling quoted values
                let fields = Self::parse_csv_line(line);
                if fields.len() >= 4 {
                    // Format: Node,AdapterCompatibility,Name,PNPDeviceID
                    let compatibility = fields[1].trim();
                    let name = fields[2].trim();
                    let pnp_device_id = fields[3].trim();
                    
                    let compatibility_lower = compatibility.to_lowercase();
                    let name_lower = name.to_lowercase();
                    let pnp_lower = pnp_device_id.to_lowercase();
                    
                    let is_integrated = 
                        // Intel integrated
                        (compatibility_lower.contains("intel") && (
                            name_lower.contains("uhd") || name_lower.contains("iris") || 
                            name_lower.contains("xe") || name_lower.contains("hd graphics")
                        )) ||
                        // AMD integrated - broader detection
                        ((compatibility_lower.contains("amd") || compatibility_lower.contains("advanced micro devices")) && (
                            (name_lower.contains("radeon") && name_lower.contains("graphics")) ||
                            name_lower.contains("vega") || name_lower.contains("rdna") ||
                            pnp_lower.contains("&subsys_")
                        )) ||
                        // Generic integrated indicators
                        name_lower.contains("integrated") || name_lower.contains("onboard");
                    
                    if is_integrated {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    #[cfg(target_os = "linux")]
    fn detect_integrated_graphics_linux() -> bool {
        // Check /sys/class/drm for integrated graphics
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("card") && !name.contains("-") {
                        if let Ok(vendor) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/vendor", name)) {
                            // Intel: 0x8086, AMD: 0x1002
                            if vendor.trim() == "0x8086" || vendor.trim() == "0x1002" {
                                if let Ok(_device_path) = std::fs::read_to_string(format!("/sys/class/drm/{}/device/device", name)) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: Check lspci
        if let Ok(output) = Command::new("lspci").args(&["-v"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("vga") && (
                    line_lower.contains("intel") || line_lower.contains("amd")
                ) {
                    return true;
                }
            }
        }
        
        false
    }
    
    #[cfg(target_os = "macos")]
    fn detect_integrated_graphics_macos() -> bool {
        // Check system_profiler for integrated graphics
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-json"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                if let Some(displays) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
                    for display in displays {
                        if let Some(name) = display.get("_name").and_then(|v| v.as_str()) {
                            let name_lower = name.to_lowercase();
                            if name_lower.contains("intel") || name_lower.contains("integrated") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        
        false
    }
    
    /// Detect CPU information
    fn detect_cpu() -> CpuInfo {
        let mut name = String::new();
        let mut vendor = CpuVendor::Other("Unknown".to_string());
        let has_integrated_graphics;
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("wmic")
                .args(&["cpu", "get", "name"])
                .output() 
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                for line in output_str.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.to_lowercase().contains("name") {
                        name = line.to_string();
                        break;
                    }
                }
            }
        }
        
        // Linux: Read /proc/cpuinfo
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") {
                        if let Some(cpu_name) = line.split(':').nth(1) {
                            name = cpu_name.trim().to_string();
                            break;
                        }
                    }
                }
            }
        }
        
        // macOS: Use sysctl
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "machdep.cpu.brand_string"])
                .output() 
            {
                name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        
        let name_lower = name.to_lowercase();
        
        // Determine CPU vendor
        if name_lower.contains("intel") {
            vendor = CpuVendor::Intel;
        } else if name_lower.contains("amd") {
            vendor = CpuVendor::AMD;
        }
        
        // Detect integrated graphics by checking actual system
        has_integrated_graphics = Self::detect_integrated_graphics(&vendor, &name_lower);
        
        let supports_quicksync = vendor == CpuVendor::Intel && has_integrated_graphics;
        
        CpuInfo {
            name,
            vendor,
            has_integrated_graphics,
            supports_quicksync,
        }
    }
    
    /// Determine supported encoders based on hardware information
    fn determine_supported_encoders(gpus: &[GpuInfo], cpu: &CpuInfo) -> Vec<String> {
        let mut encoders = Vec::new();
        
        // Determine supported encoders based on detected hardware
        
        // Add software encoders (always available)
        encoders.extend_from_slice(&[
            "libx264".to_string(),
            "libx265".to_string(), 
            "libvpx".to_string(),
            "libvpx-vp9".to_string(),
            "libaom-av1".to_string(),
        ]);
        
        // Add hardware encoders
        for gpu in gpus {
            if gpu.supports_nvenc {
                encoders.extend_from_slice(&[
                    "h264_nvenc".to_string(),
                    "hevc_nvenc".to_string(),
                ]);
                
                // Newer NVIDIA cards support AV1 NVENC
                if let Some(arch) = &gpu.architecture {
                    if arch.contains("Ada Lovelace") || arch.contains("Ampere") {
                        encoders.push("av1_nvenc".to_string());
                    }
                }
            }
            
            if gpu.supports_amf {
                encoders.extend_from_slice(&[
                    "h264_amf".to_string(),
                    "hevc_amf".to_string(),
                ]);
                
                // Newer AMD cards support AV1 AMF
                if let Some(arch) = &gpu.architecture {
                    if arch.contains("RDNA") {
                        encoders.push("av1_amf".to_string());
                    }
                }
            }
            
            if gpu.supports_qsv {
                encoders.extend_from_slice(&[
                    "h264_qsv".to_string(),
                    "hevc_qsv".to_string(),
                    "vp9_qsv".to_string(), // Intel has good VP9 support
                ]);
                
                // Newer Intel GPUs support AV1
                if gpu.name.to_lowercase().contains("arc") || gpu.name.to_lowercase().contains("xe") {
                    encoders.push("av1_qsv".to_string());
                }
            }
        }
        
        // CPU integrated graphics
        if cpu.supports_quicksync {
            // Avoid duplicate additions
            for encoder in ["h264_qsv", "hevc_qsv", "vp9_qsv"] {
                if !encoders.contains(&encoder.to_string()) {
                    encoders.push(encoder.to_string());
                }
            }
        }
        encoders
    }
    
    /// Cache hardware information (avoid repeated detection)
    pub fn get_cached_hardware_info() -> HardwareInfo {
        use std::sync::{Arc, Mutex, OnceLock};
        
        static HARDWARE_CACHE: OnceLock<Arc<Mutex<Option<HardwareInfo>>>> = OnceLock::new();
        
        let cache = HARDWARE_CACHE.get_or_init(|| Arc::new(Mutex::new(None)));
        
        // Check cache
        if let Ok(mut cached) = cache.try_lock() {
            if cached.is_none() {
                *cached = Some(Self::detect_hardware());
            }
            return cached.as_ref().unwrap().clone();
        }
        
        // If unable to acquire lock, detect directly (fallback method)
        Self::detect_hardware()
    }
}