# Build with bundled FFmpeg executables (self-contained, no LLVM/static libs required)

$FFMPEG_DIR = "$PSScriptRoot\ffmpeg-static"

Write-Host "Building with bundled FFmpeg executables (no static linking required)..." -ForegroundColor Green

# Check if FFmpeg exists
if (-not (Test-Path "$FFMPEG_DIR\bin\ffmpeg.exe")) {
    Write-Host "FFmpeg not found. Running download script..." -ForegroundColor Yellow
    & "$PSScriptRoot\download-ffmpeg-full-zip.ps1"
    if ($LASTEXITCODE -ne 0) {
        exit 1
    }
}

# Set environment variables
$env:FFMPEG_DIR = $FFMPEG_DIR

# Note: LLVM/Clang no longer required since we use bundled executables instead of static linking
Write-Host "Using bundled FFmpeg approach - no C compiler, bindgen, or LLVM required" -ForegroundColor Green

# Clean build thoroughly
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean
Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "Removed target directory for fresh build" -ForegroundColor Yellow

# Copy DLLs to target directory for runtime
$targetDir = "$PSScriptRoot\target\release"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

Write-Host "Copying FFmpeg DLLs and executables to target directory..." -ForegroundColor Cyan
Copy-Item "$FFMPEG_DIR\bin\*.dll" $targetDir -Force
Copy-Item "$FFMPEG_DIR\bin\ffmpeg.exe" $targetDir -Force
Copy-Item "$FFMPEG_DIR\bin\ffprobe.exe" $targetDir -Force

# Build the project
Write-Host "Building pure Rust project with bundled FFmpeg executables..." -ForegroundColor Green
Write-Host "Icon file check:" -ForegroundColor Cyan
if (Test-Path "icon.ico") {
    Write-Host "  * icon.ico found (preferred)" -ForegroundColor Green
} elseif (Test-Path "icon.png") {
    Write-Host "  * icon.png found (fallback)" -ForegroundColor Yellow
} else {
    Write-Host "  * No icon file found" -ForegroundColor Red
}

cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild successful!" -ForegroundColor Green
    Write-Host "Executable: target\release\ffmpeg_gui.exe" -ForegroundColor Yellow
    
    Write-Host "`nBundled FFmpeg executables and libraries:" -ForegroundColor Cyan
    Get-ChildItem "$targetDir\*.dll" | Where-Object { $_.Name -like "*av*" -or $_.Name -like "*sw*" } | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
    Get-ChildItem "$targetDir\ffmpeg*.exe" | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor Green
    }
    
    Write-Host "`nThe release is completely self-contained and works without FFmpeg installation or LLVM/C compilers." -ForegroundColor Green
} else {
    Write-Host "`nBuild failed. Check error messages above." -ForegroundColor Red
}