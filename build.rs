fn main() {
    // Add Windows icon resource
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;
        
        let mut res = winres::WindowsResource::new();
        
        // Set icon - use ICO format for best compatibility
        if Path::new("icon.ico").exists() {
            println!("cargo:warning=Found icon.ico, setting as application icon");
            res.set_icon("icon.ico");
        } else if Path::new("icon.png").exists() {
            println!("cargo:warning=Found icon.png, setting as application icon (fallback)");
            res.set_icon("icon.png");
        } else {
            println!("cargo:warning=No icon file found in project root");
        }
        
        // Set basic version info
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        res.set_version_info(winres::VersionInfo::FILEVERSION, 0x0001000000000000);
        
        // Compile the resource
        match res.compile() {
            Ok(_) => println!("cargo:warning=Windows icon resource compiled successfully"),
            Err(e) => {
                println!("cargo:warning=Failed to compile Windows icon resource: {}", e);
                println!("cargo:warning=The executable will use the default Windows icon");
            }
        }
    }
    
    println!("cargo:warning=Using bundled FFmpeg executables - no static library linking required");
}