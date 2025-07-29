# FFmpeg GUI - Self-Contained Video Processing Tool

![FFmpeg GUI Interface](./images/screenshot.png)

A modern FFmpeg graphical user interface built with Rust and egui, designed to work without requiring FFmpeg installation. The project includes both traditional single-task processing and an experimental automation workflow system.

*基于Rust和egui开发的现代化FFmpeg图形界面工具，无需安装FFmpeg即可使用。项目包含传统的单任务处理和实验性的自动化工作流系统。*

## ⚠️ Project Status / 项目状态

**Automation Flow** - The automation workflow feature is currently in **beta/experimental stage**. While basic functionality works, the visual node editor and advanced workflow features are still under development.

**自动化流程** - 自动化工作流功能目前处于**测试/实验阶段**。虽然基本功能可用，但可视化节点编辑器和高级工作流功能仍在开发中。

## 🚀 Features / 特性

### Key Advantages / 主要优势
- **Self-Contained** - No need to install FFmpeg separately / 无需单独安装FFmpeg
- **Pure Rust** - No LLVM, C compilers, or static libraries required / 无需LLVM、C编译器或静态库
- **Hardware Acceleration** - Automatic detection of NVENC, QSV, AMF, VideoToolbox, VAAPI / 自动检测硬件加速
- **Dual Processing Modes** - Traditional single-task + experimental automation workflows / 传统单任务 + 实验性自动化工作流
- **Real-time Output** - Dynamic timestamp-based file naming / 实时输出文件名（基于动态时间戳）

### Tech Stack / 技术栈
- **Rust** - Pure Rust implementation with memory safety / 纯Rust实现，内存安全
- **egui** - Immediate mode GUI framework with drag-and-drop / 即时模式GUI框架，支持拖放
- **Bundled FFmpeg** - Self-contained executables (no static linking) / 自包含可执行文件（无静态链接）
- **Structured Logging** - Comprehensive logging system / 结构化日志系统

### Dependencies / 依赖库
- `anyhow` - Error handling / 错误处理
- `rfd` - Native file dialogs / 原生文件对话框
- `serde` - Serialization/deserialization / 序列化/反序列化
- `which` - Executable detection / 可执行文件检测
- `lazy_static` - Global state management / 全局状态管理

## 📁 Project Structure / 项目结构

```
ffmpeg-gui/
├── src/
│   ├── main.rs              # Main program entry / 主程序入口
│   ├── app_state.rs         # Application state management / 应用状态管理
│   ├── ui_components.rs     # UI components / UI组件
│   ├── bundled_ffmpeg.rs    # Bundled FFmpeg management / 内置FFmpeg管理
│   ├── ffmpeg_worker_simple.rs # Simplified FFmpeg worker / 简化FFmpeg工作器
│   ├── task_executor.rs     # Task executor / 任务执行器
│   ├── codec_manager.rs     # Codec management / 编解码器管理
│   ├── preset_manager.rs    # Preset management / 预设管理
│   ├── operation_settings.rs # Operation settings / 操作设置
│   ├── hardware_detector.rs # Hardware detection / 硬件检测
│   ├── automation_flow.rs   # Automation workflow (BETA) / 自动化流程(测试版)
│   └── language.rs          # Multi-language support / 多语言支持
├── ffmpeg-static/           # FFmpeg executables and libraries / FFmpeg可执行文件和库
│   ├── bin/                 # FFmpeg executables / FFmpeg可执行文件
│   ├── lib/                 # FFmpeg libraries / FFmpeg库文件
│   └── include/             # FFmpeg headers / FFmpeg头文件
├── build.rs                 # Build script (simplified) / 构建脚本(简化版)
├── build-direct.ps1         # Windows build script / Windows构建脚本
├── Cargo.toml              # Rust project configuration / Rust项目配置
├── icon.ico                # Application icon / 应用图标
└── README.md               # Documentation / 文档
```

## 🛠️ Build Requirements / 编译要求

### System Requirements / 系统要求
- **Rust** 1.70+ (Only dependency needed / 唯一需要的依赖)
- **Windows** 10/11 (x64) - Primary target / 主要目标平台
- No C compilers, LLVM, or FFmpeg development libraries required / 无需C编译器、LLVM或FFmpeg开发库

### Architecture Benefits / 架构优势
- **Pure Rust Implementation** - No unsafe FFmpeg bindings / 纯Rust实现 - 无unsafe FFmpeg绑定
- **Executable-based Architecture** - Uses process spawning instead of static linking / 基于可执行文件架构 - 使用进程生成而非静态链接
- **Simplified Build Process** - Single cargo build command / 简化构建流程 - 单一cargo构建命令
- **Cross-platform Potential** - Easy to port to other platforms / 跨平台潜力 - 易于移植到其他平台

### FFmpeg Resources / FFmpeg资源
The project includes complete FFmpeg distribution in `ffmpeg-static/`:

*项目在 `ffmpeg-static/` 中包含完整的FFmpeg分发版：*

- Executable files (`bin/`) - ffmpeg.exe, ffprobe.exe / 可执行文件
- Library files (`lib/`) - Runtime DLLs / 运行时库
- Header files (`include/`) - For reference / 头文件(仅供参考)

### Build Steps / 编译步骤

#### Windows (One-Click Build / 一键构建)
Use the provided PowerShell script:

*使用提供的PowerShell脚本：*

```powershell
# Run the build script / 运行构建脚本
.\build-direct.ps1
```

The script automatically / 该脚本自动：
1. Downloads FFmpeg if not present / 如不存在则下载FFmpeg
2. Cleans previous builds / 清理之前的构建
3. Copies FFmpeg executables and DLLs / 复制FFmpeg可执行文件和DLL
4. Compiles the project / 编译项目
5. Creates self-contained release / 创建自包含发布版

#### Manual Build / 手动编译
For manual compilation:

*手动编译：*

```bash
# Simply build with cargo / 直接使用cargo构建
cargo build --release

# Ensure FFmpeg executables are in target/release/
# 确保FFmpeg可执行文件在target/release/目录中
```

#### Output / 输出
After successful build / 构建成功后：
- Executable: `target/release/ffmpeg_gui.exe`
- All FFmpeg DLLs copied to release directory / 所有FFmpeg DLL已复制到发布目录
- Complete self-contained distribution / 完整的自包含分发版

## 🎯 Processing Modes / 处理模式

### 1. Traditional Single-Task Processing / 传统单任务处理
**Status: Stable ✅** / **状态：稳定 ✅**

#### Video Processing / 视频处理
- Format conversion (MP4, AVI, MKV, MOV, WebM, etc.) / 格式转换
- Resolution adjustment and cropping / 分辨率调整和裁剪
- Frame rate modification and video compression / 帧率修改和视频压缩
- Hardware acceleration (NVENC, QSV, AMF, VideoToolbox, VAAPI) / 硬件加速
- Video filters and effects / 视频滤镜和特效

#### Audio Processing / 音频处理
- Format conversion (MP3, AAC, FLAC, WAV, etc.) / 格式转换
- Sample rate and bitrate adjustment / 采样率和码率调整
- Volume control and audio normalization / 音量控制和音频标准化
- Audio trimming and merging / 音频裁剪和合并

### 2. Automation Workflow System / 自动化工作流系统
**Status: Beta/Experimental ⚠️** / **状态：测试/实验性 ⚠️**

#### Working Features / 可用功能
- Node-based workflow system / 基于节点的工作流系统
- Drag-and-drop file input / 拖放式文件输入
- Auto-fill output with dynamic timestamps / 动态时间戳自动填充输出
- Basic workflow execution / 基本工作流执行  
- Hardware acceleration compatibility / 硬件加速兼容性

#### Under Development / 开发中
- Visual node editor improvements / 可视化节点编辑器改进
- Advanced node types / 高级节点类型
- Workflow templates / 工作流模板
- Error handling enhancements / 错误处理增强

### Shared Features / 共享功能
- Real-time progress monitoring / 实时进度监控
- Comprehensive logging system / 全面日志系统
- Preset configuration management / 预设配置管理
- Multi-language support (English/Chinese) / 多语言支持（英中文）

## 📖 Usage / 使用方法

### Traditional Mode / 传统模式
1. **Launch Application** / **启动应用**: Run `ffmpeg_gui.exe` / 运行 `ffmpeg_gui.exe`
2. **Select Operation** / **选择操作**: Choose video/audio processing from tabs / 从标签页选择视频/音频处理
3. **Import Files** / **导入文件**: Click browse or drag files to interface / 点击浏览或拖拽文件到界面
4. **Configure Settings** / **配置设置**: Set format, quality, hardware acceleration / 设置格式、质量、硬件加速
5. **Start Processing** / **开始处理**: Click start button for conversion / 点击开始按钮进行转换

### Automation Workflow Mode (Beta) / 自动化工作流模式（测试版）
1. **Switch to Workflow Tab** / **切换到工作流标签**: Click "Automation Flow" tab / 点击"自动化流程"标签
2. **Create Workflow** / **创建工作流**: Add nodes by right-clicking / 右键点击添加节点
3. **Connect Nodes** / **连接节点**: Drag between node connection points / 在节点连接点间拖拽
4. **Configure Nodes** / **配置节点**: Double-click nodes to set parameters / 双击节点设置参数
5. **Execute Workflow** / **执行工作流**: Click "Execute Workflow" button / 点击"执行工作流"按钮

**Note**: Automation workflow is experimental. Use traditional mode for production work.

**注意**：自动化工作流为实验功能，生产环境请使用传统模式。

## 🔧 Build Options / 构建选项

### Release Build / 发布版本构建
```bash
cargo build --release
```

### Debug Build / 调试版本构建  
```bash
cargo build
```

### Clean Build Cache / 清理构建缓存
```bash
cargo clean
```

## 📋 Architecture / 架构说明

### Bundled FFmpeg Approach / 内置FFmpeg方案

This application uses bundled FFmpeg executables instead of static library linking:

*此应用程序使用内置FFmpeg可执行文件而非静态库链接：*

- **bundled_ffmpeg.rs** - Manages FFmpeg executable detection and execution / 管理FFmpeg可执行文件检测和执行
- **ffmpeg_worker_simple.rs** - Simplified worker using external commands / 使用外部命令的简化工作器
- **task_executor.rs** - High-level task processing / 高级任务处理

### Benefits / 优势
- **Simplicity** - No complex static library linking / 无复杂静态库链接
- **Reliability** - Uses proven FFmpeg executables / 使用经过验证的FFmpeg可执行文件
- **Self-contained** - Everything needed is included / 包含所需的一切
- **Cross-environment** - Works in various Windows environments / 在各种Windows环境中工作

## 🐛 Troubleshooting / 故障排除

### Common Issues / 常见问题

**1. FFmpeg executables not found / FFmpeg可执行文件未找到**
```
FFmpeg executables not found. Please ensure ffmpeg.exe and ffprobe.exe are in the same directory as the application
```
**Solution / 解决：** 
- Run the build script which automatically downloads FFmpeg / 运行构建脚本自动下载FFmpeg
- Manually copy `ffmpeg.exe` and `ffprobe.exe` to the release directory / 手动复制到发布目录

**2. DLL not found errors / DLL未找到错误**
```
avcodec-61.dll not found
```
**Solution / 解决：** Ensure all FFmpeg DLLs are copied to the same directory as the executable / 确保所有FFmpeg DLL复制到可执行文件同目录

**3. Permission errors / 权限错误**
```
Access denied when executing FFmpeg
```
**Solution / 解决：** 
- Run as administrator if needed / 如需要以管理员身份运行
- Check antivirus software blocking FFmpeg executables / 检查杀毒软件是否阻止FFmpeg

**4. Environment compatibility issues / 环境兼容性问题**
```
Command execution failed
```
**Solution / 解决：** Ensure FFmpeg executables have proper permissions and are not blocked by security software / 确保FFmpeg可执行文件有适当权限且未被安全软件阻止

### Verification Steps / 验证步骤

Check if FFmpeg is properly bundled:
```cmd
# Navigate to release directory / 进入发布目录
cd target\release

# Test FFmpeg / 测试FFmpeg
.\ffmpeg.exe -version
.\ffprobe.exe -version
```

Expected files in release directory / 发布目录中应有的文件：
- `ffmpeg_gui.exe` - Main application / 主应用程序
- `ffmpeg.exe` - FFmpeg executable / FFmpeg可执行文件  
- `ffprobe.exe` - FFprobe executable / FFprobe可执行文件
- `*.dll` - FFmpeg runtime libraries / FFmpeg运行时库

## 📄 License / 许可证

This project is licensed under an open source license. See LICENSE file for details.

*本项目采用开源许可证，具体信息请查看LICENSE文件。*

FFmpeg components follow their own license terms.

*FFmpeg组件遵循其自身的许可证条款。*
