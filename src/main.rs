use eframe::egui;
use anyhow::Result;

// Import logging macros from logging module
#[macro_use]
mod logging;

mod ffmpeg_worker_simple;
mod bundled_ffmpeg;
mod app_state;
mod ui_components;
mod language;
mod task_executor;
mod codec_manager;
mod comprehensive_codec_registry;
mod comprehensive_ui_components;
mod comprehensive_command_builder;
mod preset_manager;
mod operation_settings;
mod hardware_detector;
mod automation_flow;

use app_state::*;
use app_state::ProjectConfig;
use ui_components::*;
use language::*;
use task_executor::*;
use codec_manager::CodecManager;
use std::sync::{Arc, Mutex};


#[derive(Clone)]
enum CompatibilityWarningContext {
    None,
    StartProcessing(OperationType),
    PreviewCommand(OperationType),
}

struct FFmpegGui {
    current_operation: Option<OperationType>,
    
    input_files: Vec<String>,
    output_file: String,
    
    dedicated_video_file: String,
    
    video_input_files: Vec<String>,
    video_output_file: String,
    audio_input_files: Vec<String>,
    audio_output_file: String,
    batch_input_files: Vec<String>,
    batch_output_file: String,
    
    current_category: Option<String>,
    
    video_settings: VideoSettings,
    audio_settings: AudioSettings,
    
    tasks: Arc<Mutex<Vec<ProcessingTask>>>,
    tasks_for_ui: Vec<ProcessingTask>,
    next_task_id: usize,
    task_executor: Option<TaskExecutor>,
    
    status_message: String,
    file_info: String,
    
    left_panel_width: f32,
    right_panel_width: f32,
    
    current_language: Language,
    translations: Translations,
    
    dark_mode: bool,
    
    hardware_info: Option<crate::hardware_detector::HardwareInfo>,
    cached_hardware_encoders: Option<Vec<String>>,
    hardware_detection_receiver: Option<std::sync::mpsc::Receiver<(Option<crate::hardware_detector::HardwareInfo>, Option<Vec<String>>)>>,
    hardware_detection_started: bool,
    
    command_preview: String,
    
    show_about_dialog: bool,
    show_compatibility_warning: bool,
    compatibility_warning_message: String,
    compatibility_recommended_codec: Option<String>,
    compatibility_recommended_format: Option<String>,
    compatibility_warning_context: CompatibilityWarningContext,
    
    // Track container format changes for auto codec reset
    last_container_format: String,
    
    // Video orientation detection
    is_portrait_video: Option<bool>,
    detected_resolution: Option<(u32, u32)>,
    
    // Cache to avoid repeated file analysis
    last_analyzed_file: String,
    file_info_cache: String,
    
    show_automation_editor: bool,
    current_workflow: Option<automation_flow::AutomationWorkflow>,
    workflow_executor: automation_flow::WorkflowExecutor,
    selected_node: Option<String>,
    dragging_node: Option<String>,
    dragging_offset: egui::Vec2,
    workflow_canvas_offset: egui::Vec2,
    workflow_canvas_zoom: f32,
    
    // Workflow window state
    workflow_window_pos: Option<egui::Pos2>,
    workflow_window_size: Option<egui::Vec2>,
    workflow_window_context: Option<eframe::egui::Context>,
    open_workflow_window_requested: bool,
    workflow_window_open: bool,
    
    creating_connection: bool,
    connection_source_node: Option<String>,
    connection_source_port: Option<usize>,
    connection_target_pos: egui::Pos2,
    
    workflow_history: Vec<automation_flow::AutomationWorkflow>,
    history_index: isize,
    max_history_size: usize,
}

impl Default for FFmpegGui {
    fn default() -> Self {
        let detected_language = detect_system_language();
        let translations = Translations::new(detected_language.clone());
        let tasks = Arc::new(Mutex::new(Vec::new()));
        let task_executor = TaskExecutor::new(tasks.clone());
        task_executor.start();
        
        // Start hardware detection immediately
        log_debug!("Starting hardware detection at program startup...");
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let hardware_info = Some(crate::hardware_detector::HardwareDetector::detect_hardware());
            let encoders = match crate::codec_manager::CodecManager::detect_hardware_encoders() {
                Ok(encoders) => Some(encoders),
                Err(_) => Some(Vec::new()),
            };
            let _ = tx.send((hardware_info, encoders));
        });
        
        Self {
            current_operation: None,
            input_files: Vec::new(),
            output_file: String::new(),
            dedicated_video_file: String::new(),
            
            video_input_files: Vec::new(),
            video_output_file: String::new(),
            audio_input_files: Vec::new(),
            audio_output_file: String::new(),
            batch_input_files: Vec::new(),
            batch_output_file: String::new(),
            current_category: None,
            
            video_settings: VideoSettings::default(),
            audio_settings: AudioSettings::default(),
            tasks: tasks.clone(),
            tasks_for_ui: Vec::new(),
            next_task_id: 1,
            task_executor: Some(task_executor),
            status_message: translations.ready().to_string(),
            file_info: String::new(),
            left_panel_width: 250.0,
            right_panel_width: 300.0,
            current_language: detected_language,
            translations,
            dark_mode: true,
            hardware_info: None,
            cached_hardware_encoders: None,
            hardware_detection_receiver: Some(rx),
            hardware_detection_started: true,
            command_preview: String::new(),
            show_about_dialog: false,
            show_compatibility_warning: false,
            compatibility_warning_message: String::new(),
            compatibility_recommended_codec: None,
            compatibility_recommended_format: None,
            compatibility_warning_context: CompatibilityWarningContext::None,
            
            last_container_format: String::new(),
            
            is_portrait_video: None,
            detected_resolution: None,
            
            last_analyzed_file: String::new(),
            file_info_cache: String::new(),
            
            show_automation_editor: false,
            current_workflow: None,
            workflow_executor: automation_flow::WorkflowExecutor::new(),
            selected_node: None,
            dragging_node: None,
            dragging_offset: egui::Vec2::ZERO,
            workflow_canvas_offset: egui::Vec2::ZERO,
            workflow_canvas_zoom: 1.0,
            
            // Workflow window state
            workflow_window_pos: None,
            workflow_window_size: None,
            workflow_window_context: None,
            open_workflow_window_requested: false,
            workflow_window_open: false,
            
            creating_connection: false,
            connection_source_node: None,
            connection_source_port: None,
            connection_target_pos: egui::Pos2::ZERO,
            
            workflow_history: Vec::new(),
            history_index: -1,
            max_history_size: 50,
        }
    }
}

impl FFmpegGui {
    fn get_cached_hardware_encoders(&mut self) -> Vec<String> {
        // Check if async task is completed (detection started at program startup)
        if let Some(receiver) = &self.hardware_detection_receiver {
            if let Ok((hardware_info, encoders)) = receiver.try_recv() {
                self.hardware_info = hardware_info;
                self.cached_hardware_encoders = encoders.clone();
                self.hardware_detection_receiver = None;
                
                
                // Update workflow executor hardware cache
                if let Some(ref encoders) = encoders {
                    self.workflow_executor.update_hardware_cache(encoders.clone());
                    log_debug!("Hardware detection completed - updated workflow executor cache with {} encoders", encoders.len());
                } else {
                    log_debug!("Hardware detection completed - no encoders detected");
                }
            }
        }
        
        // Return empty vector if detection not yet complete
        self.cached_hardware_encoders.clone().unwrap_or_else(Vec::new)
    }
    
    fn refresh_hardware_cache(&mut self) {
        self.hardware_info = None;
        self.cached_hardware_encoders = None;
        self.hardware_detection_started = false;
        self.hardware_detection_receiver = None;
        
        // Clear workflow executor hardware cache
        self.workflow_executor.update_hardware_cache(Vec::new());
    }
}

impl eframe::App for FFmpegGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global hotkey handling
        ctx.input(|i| {
            // Ctrl+W to open Workflow Editor
            if i.modifiers.ctrl && i.key_pressed(egui::Key::W) {
                self.open_workflow_window_requested = true;
            }
        });
        
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        self.handle_drag_and_drop(ctx);
        
        self.sync_tasks();
        
        // Delayed hardware detection initialization to avoid blocking at startup
        // Hardware detection will be performed when first needed
        // if self.cached_hardware_encoders.is_none() {
        //     self.get_cached_hardware_encoders();
        // }
        
        // Check for container format changes and reset incompatible codecs
        self.check_and_reset_incompatible_codecs();
        
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(self.translations.menu_file(), |ui| {
                    if ui.button(self.translations.new_project()).clicked() {
                        self.reset_project();
                        ui.close_menu();
                    }
                    if ui.button(self.translations.save_project_as()).clicked() {
                        self.save_project_as();
                        ui.close_menu();
                    }
                    if ui.button(self.translations.load_project_from()).clicked() {
                        self.load_project_from();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(self.translations.exit()).clicked() {

                        if let Some(executor) = &self.task_executor {
                            executor.stop();
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("🔧 Automation", |ui| {
                    if ui.button("📊 Open Workflow Editor").clicked() {
                        self.open_workflow_window(ctx);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("💾 Save Workflow").clicked() {
                        self.save_workflow();
                        ui.close_menu();
                    }
                    
                    if ui.button("📁 Load Workflow").clicked() {
                        self.load_workflow();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("🚀 Execute Workflow").clicked() {
                        self.execute_workflow();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button(self.translations.menu_settings(), |ui| {
                    let theme_text = if self.dark_mode {
                        self.translations.light_mode()
                    } else {
                        self.translations.dark_mode()
                    };
                    if ui.button(format!("🌙 {}", theme_text)).clicked() {
                        self.dark_mode = !self.dark_mode;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button(self.translations.reset_all_settings()).clicked() {
                        self.video_settings = VideoSettings::default();
                        self.audio_settings = AudioSettings::default();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button(self.translations.menu_language(), |ui| {
                    let mut language_changed = false;
                    for lang in Language::all_languages() {
                        let is_current = lang == self.current_language;
                        if ui.selectable_label(is_current, lang.display_name()).clicked() {
                            if !is_current {
                                self.current_language = lang.clone();
                                self.translations = Translations::new(lang);
                                self.status_message = self.translations.ready().to_string();
                                language_changed = true;
                            }
                        }
                    }
                    if language_changed {
                        ui.close_menu();
                    }
                });
                
                ui.menu_button(self.translations.menu_help(), |ui| {
                    if ui.button(self.translations.about()).clicked() {
                        self.show_about_dialog = true;
                        ui.close_menu();
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.status_message);
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{}: {}", 
                    self.translations.active_tasks(),
                    self.tasks_for_ui.iter().filter(|t| t.status == TaskStatus::Running).count()));
                ui.separator();
                ui.label(format!("{}: {}", self.translations.total_tasks(), self.tasks_for_ui.len()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(self.translations.clear_completed_tasks()).clicked() {
                        self.clear_completed_tasks();
                    }
                });
            });
        });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(self.left_panel_width)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let operation_changed = OperationSelector::show(ui, &mut self.current_operation, &self.translations);

                    if operation_changed {
                        if let Some(operation) = self.current_operation.clone() {
                            self.switch_operation_category(&operation);
                        }
                    }
                });
            });

        let right_panel_response = egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(self.right_panel_width)
            .width_range(250.0..=500.0)
            .show_separator_line(true) 
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading(self.translations.task_list());
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .id_salt("task_scroll") 
                        .auto_shrink([false, false]) 
                        .show(ui, |ui| {
                            let old_len = self.tasks_for_ui.len();
                            TaskPanel::show(ui, &mut self.tasks_for_ui, &self.translations);
                            
                            if self.tasks_for_ui.len() != old_len {
                                if let Ok(mut tasks_guard) = self.tasks.try_lock() {
                                    *tasks_guard = self.tasks_for_ui.clone();
                                }
                            }
                        });
                });
            });
        
        let new_width = right_panel_response.response.rect.width();
        
        self.right_panel_width = new_width;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(operation) = self.current_operation.clone() {
                    ui.heading(self.translations.current_operation(&operation.display_name(&self.translations)));
                    ui.separator();
                    
                    self.show_file_selection(ui, &operation);
                    
                    ui.separator();
                    
                    if let Some(ref operation) = self.current_operation {
                        let show_encoding_features = match operation {
                            OperationType::VideoToGif | OperationType::GifResize => false,
                            _ => true,
                        };
                        
                        if show_encoding_features && !self.output_file.is_empty() {
                            let output_ext = std::path::Path::new(&self.output_file)
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            
                            if !output_ext.is_empty() {
                                ui.collapsing(self.translations.smart_recommendations(), |ui| {
                                    SettingsPanel::show_smart_recommendations(
                                        ui, 
                                        &output_ext, 
                                        &mut self.video_settings, 
                                        &mut self.audio_settings,
                                        &self.translations
                                    );
                                });
                                
                                ui.collapsing(self.translations.encoding_presets(), |ui| {
                                    SettingsPanel::show_preset_selector(
                                        ui, 
                                        &mut self.video_settings, 
                                        &mut self.audio_settings,
                                        &self.translations
                                    );
                                });
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    self.show_settings_panel(ui, &operation);
                    
                    ui.separator();
                    
                    if !self.file_info.is_empty() {
                        ui.group(|ui| {
                            ui.heading(format!("📊 {}", self.translations.file_info()));
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("file_info_scroll")
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    ui.monospace(&self.file_info);
                                });
                        });
                        ui.separator();
                    }
                    
                    if !self.command_preview.is_empty() {
                        ui.group(|ui| {
                            ui.heading(format!("💻 {}", self.translations.ffmpeg_command_preview()));
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("command_preview_scroll")
                                .max_height(120.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.text_edit_multiline(&mut self.command_preview.as_str());
                                        ui.horizontal(|ui| {
                                            let copy_button = egui::Button::new(self.translations.copy_command())
                                                .fill(egui::Color32::from_rgb(60, 179, 113));
                                            if ui.add(copy_button).clicked() {
                                                ui.output_mut(|o| o.copied_text = self.command_preview.clone());
                                            }
                                            
                                            let clear_button = egui::Button::new(self.translations.clear())
                                                .fill(egui::Color32::from_rgb(220, 20, 60));
                                            if ui.add(clear_button).clicked() {
                                                self.command_preview.clear();
                                            }
                                        });
                                    });
                                });
                        });
                        ui.separator();
                    }
                    
                    self.show_action_buttons(ui, &operation);
                    
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading(self.translations.welcome_title());
                        ui.add_space(20.0);
                        ui.label(self.translations.welcome_instruction());
                        ui.add_space(20.0);
                        
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.heading(self.translations.quick_start());
                                ui.label(self.translations.step1());
                                ui.label(self.translations.step2());
                                ui.label(self.translations.step3());
                                ui.label(self.translations.step4());
                            });
                        });
                    });
                }
            });
        });

        if self.show_about_dialog {
            egui::Window::new(self.translations.about())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("FF GUI");
                        ui.add_space(10.0);
                        
                        ui.label(self.translations.window_title());
                        ui.add_space(5.0);
                        ui.label("Version 1.0.0");
                        ui.add_space(15.0);
                        
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.label(format!("{}: 0xDEADBEEFC0DEBABE", self.translations.author()));
                        ui.add_space(5.0);
                        
                        ui.label(format!("{}: GPL v3.0", self.translations.license()));
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.label(self.translations.built_with_ffmpeg());
                        ui.label(self.translations.developed_with_rust());
                        ui.label(self.translations.third_party_notice());
                        
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.heading(self.translations.open_source_disclaimer());
                        ui.add_space(5.0);
                        
                        ui.label(self.translations.license_notice());
                        ui.add_space(5.0);
                        
                        egui::ScrollArea::vertical()
                            .id_salt("about_dialog_scroll")
                            .max_height(80.0)
                            .show(ui, |ui| {
                                ui.label(self.translations.disclaimer_text());
                            });
                        
                        ui.add_space(15.0);
                        
                        if ui.button(self.translations.close()).clicked() {
                            self.show_about_dialog = false;
                        }
                    });
                });
        }

        if self.show_compatibility_warning {
            let window_title = if self.translations.language == crate::language::Language::Chinese {
                "⚠ 兼容性警告"
            } else {
                "⚠ Compatibility Warning"
            };
            
            egui::Window::new(window_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(if self.translations.language == crate::language::Language::Chinese {
                            "编码器/容器兼容性问题"
                        } else {
                            "Codec/Container Compatibility Issue"
                        });
                        ui.add_space(10.0);
                        
                        ui.label(&self.compatibility_warning_message);
                        ui.add_space(15.0);
                        
                        ui.separator();
                        ui.add_space(10.0);
                        
                        ui.heading(if self.translations.language == crate::language::Language::Chinese {
                            "推荐解决方案:"
                        } else {
                            "Recommended Solution:"
                        });
                        ui.add_space(5.0);
                        
                        if let Some(ref codec) = self.compatibility_recommended_codec {
                            ui.label(format!("• {} {}", 
                                if self.translations.language == crate::language::Language::Chinese { "使用编码器:" } else { "Use codec:" },
                                codec
                            ));
                        }
                        
                        if let Some(ref format) = self.compatibility_recommended_format {
                            ui.label(format!("• {} {}", 
                                if self.translations.language == crate::language::Language::Chinese { "使用容器格式:" } else { "Use container format:" },
                                format
                            ));
                        }
                        
                        ui.add_space(15.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button(if self.translations.language == crate::language::Language::Chinese { 
                                "应用推荐设置" 
                            } else { 
                                "Apply Recommendation" 
                            }).clicked() {
                                self.apply_compatibility_recommendation();
                                self.show_compatibility_warning = false;
                                // Continue with the task using the recommended settings
                                match &self.compatibility_warning_context {
                                    CompatibilityWarningContext::StartProcessing(operation) => {
                                        let op = operation.clone();
                                        self.compatibility_warning_context = CompatibilityWarningContext::None;
                                        self.force_start_processing(&op);
                                    }
                                    CompatibilityWarningContext::PreviewCommand(operation) => {
                                        let op = operation.clone();
                                        self.compatibility_warning_context = CompatibilityWarningContext::None;
                                        self.force_show_command_preview(&op);
                                    }
                                    CompatibilityWarningContext::None => {}
                                }
                            }
                            
                            if ui.button(if self.translations.language == crate::language::Language::Chinese { 
                                "继续使用当前设置" 
                            } else { 
                                "Continue Anyway" 
                            }).clicked() {
                                self.show_compatibility_warning = false;
                                // Continue with the task despite the warning
                                match &self.compatibility_warning_context {
                                    CompatibilityWarningContext::StartProcessing(operation) => {
                                        let op = operation.clone();
                                        self.compatibility_warning_context = CompatibilityWarningContext::None;
                                        self.force_start_processing(&op);
                                    }
                                    CompatibilityWarningContext::PreviewCommand(operation) => {
                                        let op = operation.clone();
                                        self.compatibility_warning_context = CompatibilityWarningContext::None;
                                        self.force_show_command_preview(&op);
                                    }
                                    CompatibilityWarningContext::None => {}
                                }
                            }
                            
                            if ui.button(if self.translations.language == crate::language::Language::Chinese { 
                                "取消" 
                            } else { 
                                "Cancel" 
                            }).clicked() {
                                self.show_compatibility_warning = false;
                                self.compatibility_warning_context = CompatibilityWarningContext::None;
                            }
                        });
                    });
                });
        }
        
        // Handle request to open workflow window
        if self.open_workflow_window_requested {
            self.open_workflow_window_requested = false;
            self.open_workflow_window(ctx);
        }
        
        // Display persistent workflow window
        if self.workflow_window_open {
            self.show_persistent_workflow_window(ctx);
        }
        
        // Original embedded workflow editor has been replaced with standalone window
        // if self.show_automation_editor {
        //     self.show_automation_editor_window(ctx);
        // }
    }
}

impl FFmpegGui {
    fn get_operation_category(&self, operation: &OperationType) -> String {
        match operation {
            OperationType::VideoConvert | OperationType::VideoCompress | 
            OperationType::VideoResize | OperationType::VideoCrop | 
            OperationType::VideoRotate | OperationType::VideoFilter |
            OperationType::ExtractVideo => "video".to_string(),
            
            OperationType::AudioConvert | OperationType::AudioCompress | 
            OperationType::AudioResample | OperationType::AudioVolume | 
            OperationType::AudioTrim | OperationType::AudioMerge |
            OperationType::ExtractAudio => "audio".to_string(),
            
            OperationType::VideoAudioMerge | OperationType::VideoAudioSplit => "video_audio".to_string(),
            
            OperationType::BatchConvert => "batch".to_string(),
            
            _ => "advanced".to_string(),
        }
    }
    
    fn switch_operation_category(&mut self, new_operation: &OperationType) {
        let new_category = self.get_operation_category(new_operation);
        
        if let Some(current_category) = &self.current_category {
            if current_category != &new_category {

                match current_category.as_str() {
                    "video" => {
                        self.video_input_files = self.input_files.clone();
                        self.video_output_file = self.output_file.clone();
                    }
                    "audio" => {
                        self.audio_input_files = self.input_files.clone();
                        self.audio_output_file = self.output_file.clone();
                    }
                    "batch" => {
                        self.batch_input_files = self.input_files.clone();
                        self.batch_output_file = self.output_file.clone();
                    }
                    _ => {}
                }
                
                match new_category.as_str() {
                    "video" => {
                        self.input_files = self.video_input_files.clone();
                        self.output_file = self.video_output_file.clone();
                    }
                    "audio" => {
                        self.input_files = self.audio_input_files.clone();
                        self.output_file = self.audio_output_file.clone();
                    }
                    "batch" => {
                        self.input_files = self.batch_input_files.clone();
                        self.output_file = self.batch_output_file.clone();
                    }
                    _ => {
                        self.input_files.clear();
                        self.output_file.clear();
                    }
                }
            }
        } else {
            match new_category.as_str() {
                "video" => {
                    self.input_files = self.video_input_files.clone();
                    self.output_file = self.video_output_file.clone();
                }
                "audio" => {
                    self.input_files = self.audio_input_files.clone();
                    self.output_file = self.audio_output_file.clone();
                }
                "batch" => {
                    self.input_files = self.batch_input_files.clone();
                    self.output_file = self.batch_output_file.clone();
                }
                _ => {}
            }
        }
        
        self.current_category = Some(new_category);
    }

    fn sync_tasks(&mut self) {
        if let Ok(tasks_guard) = self.tasks.try_lock() {
            self.tasks_for_ui = tasks_guard.clone();
        }
    }
    
    fn clear_completed_tasks(&mut self) {
        if let Ok(mut tasks_guard) = self.tasks.try_lock() {
            tasks_guard.retain(|t| t.status != TaskStatus::Completed && t.status != TaskStatus::Failed);
        }
    }
    
    fn reset_project(&mut self) {
        self.current_operation = None;
        self.input_files.clear();
        self.output_file.clear();
        self.video_settings = VideoSettings::default();
        self.audio_settings = AudioSettings::default();
        self.file_info.clear();
        self.status_message = self.translations.project_reset().to_string();
    }
    
    fn save_project_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("FF GUI Project", &["ffcfg"])
            .set_title("Save Project As")
            .save_file() {
                
            let config = ProjectConfig::from_app_state(
                self.current_operation.clone(),
                self.input_files.clone(),
                self.output_file.clone(),
                self.video_settings.clone(),
                self.audio_settings.clone(),
                None,
            );
            
            match config.save_to_file(&path.display().to_string()) {
                Ok(_) => {
                    self.status_message = self.translations.project_saved().to_string();
                },
                Err(e) => {
                    self.status_message = format!("{}: {}", self.translations.save_error(), e);
                }
            }
        }
    }
    
    fn load_project_from(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("FF GUI Project", &["ffcfg"])
            .set_title("Open Project")
            .pick_file() {
                
            match ProjectConfig::load_from_file(&path.display().to_string()) {
                Ok(config) => {
                    self.current_operation = config.current_operation;
                    self.input_files = config.input_files;
                    self.output_file = config.output_file;
                    self.video_settings = config.video_settings;
                    self.audio_settings = config.audio_settings;
                    self.status_message = self.translations.project_loaded().to_string();
                },
                Err(e) => {
                    self.status_message = format!("{}: {}", self.translations.load_error(), e);
                }
            }
        }
    }
    
    fn show_file_selection(&mut self, ui: &mut egui::Ui, operation: &OperationType) {
        // For subtitle and watermark operations, show completely separate video file selector
        match operation {
            OperationType::AddSubtitle | OperationType::AddWatermark => {
                // Show dedicated video file selector with drag-and-drop support
                ui.group(|ui| {
                    ui.label("📹 Video File (Required):");
                    
                    // Create drag-and-drop area for video file
                    let video_area_height = 60.0;
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), video_area_height),
                        egui::Sense::click()
                    );
                    
                    // Handle drag and drop for video file
                    let is_being_dragged = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());
                    let can_accept_drop = is_being_dragged && ui.rect_contains_pointer(rect);
                    
                    // Draw the drop area
                    let bg_color = if can_accept_drop {
                        egui::Color32::from_rgba_unmultiplied(100, 200, 100, 100)
                    } else if self.dedicated_video_file.is_empty() {
                        egui::Color32::from_rgba_unmultiplied(100, 100, 100, 50)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(50, 150, 50, 50)
                    };
                    
                    ui.painter().rect_filled(rect, 8.0, bg_color);
                    ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                    
                    // Handle click to open file dialog
                    if response.clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Video files", &["mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "m4v"])
                            .pick_file() {
                            self.dedicated_video_file = path.display().to_string();
                        }
                    }
                    
                    // Process dropped files for video
                    if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
                        let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
                        if let Some(file) = dropped_files.first() {
                            if let Some(path) = &file.path {
                                let ext = path.extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                                    .to_lowercase();
                                if ["mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "m4v"].contains(&ext.as_str()) {
                                    self.dedicated_video_file = path.display().to_string();
                                    // Auto-generate output file name based on video file
                                    self.generate_output_filename_from_video();
                                }
                            }
                        }
                    }
                    
                    // Draw text in the drop area
                    let text = if self.dedicated_video_file.is_empty() {
                        if can_accept_drop {
                            "📹 Drop video file here"
                        } else {
                            "📹 Click to select video file or drag & drop here"
                        }
                    } else {
                        // Show file name
                        let file_name = std::path::Path::new(&self.dedicated_video_file)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");
                        &format!("📹 {}", file_name)
                    };
                    
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::default(),
                        if self.dedicated_video_file.is_empty() { egui::Color32::GRAY } else { egui::Color32::WHITE }
                    );
                    
                    ui.add_space(5.0);
                    
                    // Show controls and status
                    ui.horizontal(|ui| {
                        if !self.dedicated_video_file.is_empty() {
                            if ui.button("🚮 Clear").clicked() {
                                self.dedicated_video_file.clear();
                            }
                            if ui.button("📁 Change").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Video files", &["mp4", "avi", "mkv", "mov", "webm", "flv", "wmv", "m4v"])
                                    .pick_file() {
                                    self.dedicated_video_file = path.display().to_string();
                                }
                            }
                        }
                    });
                    
                    // Show status for video file
                    if self.dedicated_video_file.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                            "⚠ Please select a video file first");
                    } else {
                        let is_video = self.dedicated_video_file.to_lowercase().ends_with(".mp4") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".avi") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".mkv") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".mov") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".webm") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".flv") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".wmv") ||
                                      self.dedicated_video_file.to_lowercase().ends_with(".m4v");
                        
                        if is_video {
                            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "✅ Video file ready");
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(255, 150, 100), 
                                "⚠ Selected file is not a video file");
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                // Show output file selector for these operations
                OutputSelector::show(
                    ui, 
                    &mut self.output_file, 
                    operation,
                    Some(&self.video_settings),
                    Some(&self.audio_settings),
                    &self.translations
                );
                
                // Don't show regular input files section for these operations
                return;
            }
            _ => {
                // Normal file selection for other operations
                let allow_multiple = matches!(operation, 
                    OperationType::AudioMerge | 
                    OperationType::BatchConvert | 
                    OperationType::VideoAudioMerge
                );
                
                let label = match operation {
                    OperationType::VideoAudioMerge => self.translations.input_files_video_audio(),
                    OperationType::AudioMerge => self.translations.input_audio_files(),
                    OperationType::BatchConvert => self.translations.batch_input_files(),
                    _ => self.translations.input_files(),
                };
                
                let old_files_len = self.input_files.len();
                let old_first_file = self.input_files.first().cloned();
                
                FileSelector::show(ui, &mut self.input_files, allow_multiple, label, &self.translations, || {});
                
                // Check if files changed and auto-generate output filename
                let files_changed = self.input_files.len() != old_files_len || 
                                   self.input_files.first() != old_first_file.as_ref();
                
                if files_changed && !self.input_files.is_empty() {
                    // Only auto-generate if output is empty or already auto-generated
                    if self.output_file.is_empty() || self.is_auto_generated_filename(&self.output_file) {
                        self.generate_output_filename_from_input();
                    }
                }
            }
        }
        
        if !self.input_files.is_empty() && self.input_files.len() == 1 {
            let current_file = &self.input_files[0];
            
            // Only analyze file if it's different from the last analyzed file
            if current_file != &self.last_analyzed_file {
                // Use bundled FFmpeg worker for file analysis
                let worker = ffmpeg_worker_simple::FFmpegWorker::new();
                if let Ok(info) = worker.get_file_info(current_file) {
                    let video_info = if info.video_streams.is_empty() { 
                        "None".to_string() 
                    } else { 
                        format!("{} streams", info.video_streams.len()) 
                    };
                    let audio_info = if info.audio_streams.is_empty() { 
                        "None".to_string() 
                    } else { 
                        format!("{} streams", info.audio_streams.len()) 
                    };
                    self.file_info = format!(
                        "File: {}\nDuration: {:.2}s\nVideo: {}\nAudio: {}",
                        info.filename,
                        info.duration,
                        video_info,
                        audio_info
                    );
                    self.file_info_cache = self.file_info.clone();
                    self.last_analyzed_file = current_file.clone();
                    
                    // Detect streams and resolution from info
                    if let Some(video) = info.video_streams.first() {
                        let width = video.width as u32;
                        let height = video.height as u32;
                        self.is_portrait_video = Some(height > width);
                        self.detected_resolution = Some((width, height));
                    }
                } else {
                    // If analysis fails, use cached info
                    self.file_info = self.file_info_cache.clone();
                }
                
                // Auto-generate output filename when input file is selected
                // Only auto-generate if output is empty or already auto-generated
                if self.output_file.is_empty() || self.is_auto_generated_filename(&self.output_file) {
                    self.generate_output_filename_from_input();
                }
            }
        }
        
        ui.add_space(10.0);
        OutputSelector::show(
            ui, 
            &mut self.output_file, 
            operation,
            Some(&self.video_settings),
            Some(&self.audio_settings),
            &self.translations
        );
    }
    
    fn show_settings_panel(&mut self, ui: &mut egui::Ui, operation: &OperationType) {
        // Show operation-specific settings
        let ctx = ui.ctx().clone();
        let cached_hw_encoders = self.get_cached_hardware_encoders();
        operation_settings::OperationSettings::show(
            &ctx,
            ui, 
            operation, 
            &mut self.video_settings, 
            &mut self.audio_settings, 
            &self.translations,
            self.detected_resolution,
            self.is_portrait_video,
            &cached_hw_encoders
        );
        
        ui.add_space(10.0);
        
        // Show common encoding settings for operations that need them
        let needs_video_settings = matches!(operation,
            OperationType::VideoConvert | OperationType::VideoCompress | 
            OperationType::VideoResize | OperationType::VideoCrop | 
            OperationType::VideoRotate | OperationType::VideoFilter |
            OperationType::VideoAudioMerge | OperationType::ExtractVideo
        );
        
        let needs_audio_settings = matches!(operation,
            OperationType::AudioConvert | OperationType::AudioCompress | 
            OperationType::AudioResample | OperationType::AudioVolume | 
            OperationType::AudioTrim | OperationType::AudioMerge |
            OperationType::VideoAudioMerge | OperationType::ExtractAudio
        );
        
        let cached_hw_encoders = self.get_cached_hardware_encoders();
        
        if *operation == OperationType::VideoConvert {
            if needs_video_settings {
                SettingsPanel::show_video_settings(ui, &mut self.video_settings, &self.translations, &cached_hw_encoders);
            }
            
            if needs_audio_settings {
                ui.add_space(10.0);
                SettingsPanel::show_audio_settings(ui, &mut self.audio_settings, &self.translations);
            }
        } else {
            ui.horizontal(|ui| {
                if needs_video_settings {
                    ui.vertical(|ui| {
                        SettingsPanel::show_video_settings(ui, &mut self.video_settings, &self.translations, &cached_hw_encoders);
                    });
                }
                
                if needs_audio_settings {
                    ui.vertical(|ui| {
                        SettingsPanel::show_audio_settings(ui, &mut self.audio_settings, &self.translations);
                    });
                }
            });
        }
        
    }
    
    
    fn validate_operation_requirements(&self, operation: &OperationType) -> (bool, Option<String>) {
        match operation {
            OperationType::AddSubtitle => {
                if self.dedicated_video_file.is_empty() {
                    return (false, Some(self.translations.video_file_required().to_string()));
                }
                if self.video_settings.subtitle_file.is_empty() {
                    return (false, Some(self.translations.subtitle_file_required().to_string()));
                }
                if self.output_file.is_empty() {
                    return (false, Some("Please select output file".to_string()));
                }
                // Add informative message about subtitle operation
                let info = "Note: Subtitle operations are experimental in static library mode. If the operation fails, the system will fallback to safe file copy.";
                (true, Some(info.to_string()))
            }
            OperationType::AddWatermark => {
                if self.dedicated_video_file.is_empty() {
                    return (false, Some(self.translations.video_file_required().to_string()));
                }
                if self.video_settings.watermark_file.is_empty() {
                    return (false, Some(self.translations.watermark_file_required().to_string()));
                }
                if self.output_file.is_empty() {
                    return (false, Some("Please select output file".to_string()));
                }
                (true, None)
            }
            _ => {
                let basic_requirements = !self.input_files.is_empty() && !self.output_file.is_empty();
                (basic_requirements, None)
            }
        }
    }

    fn show_action_buttons(&mut self, ui: &mut egui::Ui, operation: &OperationType) {
        ui.add_space(10.0);
        
        let (can_start, validation_message) = self.validate_operation_requirements(operation);
        
        ui.horizontal(|ui| {
            let start_button = egui::Button::new(
                egui::RichText::new(self.translations.start_processing())
                    .color(egui::Color32::WHITE)
                    .strong()
            ).min_size(egui::vec2(120.0, 35.0));
            
            if can_start {
                if ui.add(start_button.fill(egui::Color32::from_rgb(34, 139, 34))
                    .rounding(egui::Rounding::same(6.0))).clicked() {
                    self.start_processing(operation);
                }
            } else {
                ui.add_enabled(false, start_button.fill(egui::Color32::DARK_GRAY)
                    .rounding(egui::Rounding::same(6.0)));
            }
            
            ui.add_space(10.0);
            
            let preview_button = egui::Button::new(
                egui::RichText::new(self.translations.preview_command())
                    .color(egui::Color32::WHITE)
                    .strong()
            ).min_size(egui::vec2(120.0, 35.0));
            
            if can_start {
                if ui.add(preview_button.fill(egui::Color32::from_rgb(70, 130, 180))
                    .rounding(egui::Rounding::same(6.0))).clicked() {
                    self.show_command_preview(operation);
                }
            } else {
                ui.add_enabled(false, preview_button.fill(egui::Color32::DARK_GRAY)
                    .rounding(egui::Rounding::same(6.0)));
            }
            
            ui.add_space(10.0);
            
            let reset_button = egui::Button::new(
                egui::RichText::new(self.translations.reset_settings())
                    .color(egui::Color32::WHITE)
                    .strong()
            ).min_size(egui::vec2(100.0, 35.0))
                .fill(egui::Color32::from_rgb(255, 140, 0))
                .rounding(egui::Rounding::same(6.0));
            
            if ui.add(reset_button).clicked() {
                self.video_settings = VideoSettings::default();
                self.audio_settings = AudioSettings::default();
            }
        });
        
        // Show validation message if exists
        if let Some(message) = validation_message {
            ui.add_space(5.0);
            // Use different colors for errors vs warnings
            let (color, _can_proceed) = if can_start {
                (egui::Color32::from_rgb(255, 165, 0), true) // Orange for warnings
            } else {
                (egui::Color32::from_rgb(255, 100, 100), false) // Red for errors
            };
            
            ui.colored_label(color, format!("⚠ {}", message));
        }
        
        ui.add_space(10.0);
    }
    
    fn start_processing(&mut self, operation: &OperationType) {
        // Check compatibility before starting processing
        if !self.check_compatibility_and_warn() {
            // Store context so we can continue if user chooses to
            self.compatibility_warning_context = CompatibilityWarningContext::StartProcessing(operation.clone());
            return; // Show warning dialog, don't start processing
        }

        self.force_start_processing(operation);
    }

    fn force_start_processing(&mut self, operation: &OperationType) {
        // Prepare input files based on operation type
        let input_files = match operation {
            OperationType::AddSubtitle | OperationType::AddWatermark => {
                // Use dedicated video file for these operations
                vec![self.dedicated_video_file.clone()]
            }
            _ => {
                // Use regular input files for other operations
                self.input_files.clone()
            }
        };
        
        // Generate unique output filename for each task
        let unique_output_file = if self.is_auto_generated_filename(&self.output_file) {
            // If it's an auto-generated filename, create a new unique filename
            let path = std::path::Path::new(&self.output_file);
            if let Some(parent) = path.parent() {
                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("mp4");
                let timestamp = self.generate_timestamp();
                parent.join(format!("output_{}.{}", timestamp, extension))
                    .display().to_string()
            } else {
                self.output_file.clone()
            }
        } else {
            // If it's a user-specified filename, add task ID to ensure uniqueness
            let path = std::path::Path::new(&self.output_file);
            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("mp4");
            
            if let Some(parent) = path.parent() {
                parent.join(format!("{}_{}.{}", stem, self.next_task_id, extension))
                    .display().to_string()
            } else {
                format!("{}_{}.{}", stem, self.next_task_id, extension)
            }
        };
        
        let task = ProcessingTask {
            id: self.next_task_id,
            operation: operation.clone(),
            input_files,
            output_file: unique_output_file,
            video_settings: Some(self.video_settings.clone()),
            audio_settings: Some(self.audio_settings.clone()),
            progress: 0.0,
            status: TaskStatus::Pending,
            error_message: None,
            start_time: None,
            estimated_total_time: None,
            completion_time: None,
        };
        
        if let Ok(mut tasks_guard) = self.tasks.try_lock() {
            log_info!("Adding task {} ({}) to queue", task.id, task.operation.display_name(&self.translations));
            tasks_guard.push(task);
            log_info!("Task queue now has {} tasks", tasks_guard.len());
        } else {
            log_error!("Failed to lock tasks queue for adding task");
        }
        
        self.next_task_id += 1;
        self.status_message = self.translations.task_added(&operation.display_name(&self.translations));
    }
    
    fn check_compatibility_and_warn(&mut self) -> bool {
        // Skip compatibility check for GIF operations
        if let Some(ref operation) = self.current_operation {
            if matches!(operation, OperationType::VideoToGif | OperationType::GifResize) {
                return true;
            }
        }

        // Check if input file has audio/video streams when needed
        if !self.input_files.is_empty() && self.input_files.len() == 1 {
            if let Some(ref operation) = self.current_operation {
                // Check stream availability based on operation type
                let needs_video = matches!(operation,
                    OperationType::VideoConvert | OperationType::VideoCompress |
                    OperationType::VideoResize | OperationType::VideoCrop |
                    OperationType::VideoRotate | OperationType::VideoFilter |
                    OperationType::ExtractVideo | OperationType::VideoToGif
                );
                
                let needs_audio = matches!(operation,
                    OperationType::AudioConvert | OperationType::AudioCompress |
                    OperationType::AudioResample | OperationType::AudioVolume |
                    OperationType::AudioTrim | OperationType::ExtractAudio
                );
                
                // Use cached detection results or detect streams
                let (has_video, has_audio) = if let Some((_width, _height)) = self.detected_resolution {
                    // Use cached resolution data - assume file has video if we have resolution
                    (true, true) // Assume has audio for compatibility - could be improved
                } else {
                    // Fallback to detection if no cached data
                    // Use bundled FFmpeg worker to detect streams
                    let worker = ffmpeg_worker_simple::FFmpegWorker::new();
                    if let Ok((video, audio, _)) = worker.detect_streams(&self.input_files[0]) {
                        (video, audio)
                    } else {
                        // Last resort: parse file_info text
                        let info_lower = self.file_info.to_lowercase();
                        let has_video = info_lower.contains("video:") || info_lower.contains("stream #") && info_lower.contains("video");
                        let has_audio = info_lower.contains("audio:") || info_lower.contains("stream #") && info_lower.contains("audio");
                        (has_video, has_audio)
                    }
                };
                
                if needs_video && !has_video {
                    self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                        "所选文件不包含视频流。此操作需要视频输入。".to_string()
                    } else {
                        "The selected file does not contain a video stream. This operation requires video input.".to_string()
                    };
                    self.compatibility_recommended_codec = None;
                    self.compatibility_recommended_format = None;
                    self.show_compatibility_warning = true;
                    return false;
                }
                
                if needs_audio && !has_audio {
                    self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                        "所选文件不包含音频流。此操作需要音频输入。".to_string()
                    } else {
                        "The selected file does not contain an audio stream. This operation requires audio input.".to_string()
                    };
                    self.compatibility_recommended_codec = None;
                    self.compatibility_recommended_format = None;
                    self.show_compatibility_warning = true;
                    return false;
                }
            }
        }

        // Determine the target container format
        let container_format = if !self.video_settings.container_format.is_empty() && self.video_settings.container_format != "auto" {
            &self.video_settings.container_format
        } else if !self.audio_settings.format.is_empty() {
            &self.audio_settings.format
        } else {
            ""
        };

        if container_format.is_empty() {
            return true; // No format specified, nothing to check
        }

        // Check video codec compatibility
        if !self.video_settings.codec.is_empty() && self.video_settings.codec != "auto" {
            if let Err(_) = CodecManager::validate_codec_format_compatibility(
                &self.video_settings.codec, 
                container_format, 
                false
            ) {
                let recommended_codec = CodecManager::get_best_video_codec_for_format(container_format);
                self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                    format!("视频编码器 '{}' 与容器格式 '{}' 不兼容。这可能导致编码失败。", self.video_settings.codec, container_format)
                } else {
                    format!("Video codec '{}' is not compatible with container format '{}'. This may cause encoding to fail.", self.video_settings.codec, container_format)
                };
                self.compatibility_recommended_codec = Some(recommended_codec);
                self.compatibility_recommended_format = None;
                self.show_compatibility_warning = true;
                return false;
            }
        }

        // Check audio codec compatibility
        // IMPORTANT: Also check when audio codec is "auto" but would result in incompatible codec
        let audio_codec_to_check = if self.audio_settings.codec == "auto" || self.audio_settings.codec.is_empty() {
            // Predict what codec would be used for "auto"
            // Check if the default audio codec for this format would be incompatible
            let default_audio_codec = match container_format {
                "mp4" | "mov" => "aac",
                "webm" => "libopus", // WebM should default to Opus or Vorbis, not AAC
                "ogg" => "libvorbis",
                "mkv" => "aac", // MKV supports many codecs, AAC is common
                _ => "aac"
            };
            
            // Special case: if converting to WebM and audio codec is auto, check if source would use AAC
            if container_format == "webm" && (self.audio_settings.codec == "auto" || self.audio_settings.codec.is_empty()) {
                // WebM doesn't support AAC, warn user
                self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                    format!("WebM格式不支持AAC音频编码器。音频将自动转换为{}编码器。", default_audio_codec)
                } else {
                    format!("WebM format does not support AAC audio codec. The audio will be automatically converted to {} codec.", default_audio_codec)
                };
                self.compatibility_recommended_codec = Some(default_audio_codec.to_string());
                self.compatibility_recommended_format = None;
                self.show_compatibility_warning = true;
                return false;
            }
            default_audio_codec.to_string()
        } else {
            self.audio_settings.codec.clone()
        };

        if !audio_codec_to_check.is_empty() && audio_codec_to_check != "auto" {
            if let Err(_) = CodecManager::validate_codec_format_compatibility(
                &audio_codec_to_check, 
                container_format, 
                true
            ) {
                let recommended_codec = CodecManager::get_best_audio_codec_for_format(container_format);
                self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                    format!("音频编码器 '{}' 与容器格式 '{}' 不兼容。这可能导致编码失败。", audio_codec_to_check, container_format)
                } else {
                    format!("Audio codec '{}' is not compatible with container format '{}'. This may cause encoding to fail.", audio_codec_to_check, container_format)
                };
                self.compatibility_recommended_codec = Some(recommended_codec);
                self.compatibility_recommended_format = None;
                self.show_compatibility_warning = true;
                return false;
            }
        }

        // Check subtitle compatibility for AddSubtitle operations
        if let Some(ref operation) = self.current_operation {
            if matches!(operation, OperationType::AddSubtitle) {
                // Check if we have the required files
                if !self.dedicated_video_file.is_empty() && !self.video_settings.subtitle_file.is_empty() {
                    // Get output file extension
                    let output_ext = std::path::Path::new(&self.output_file)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    // Get subtitle file extension  
                    let subtitle_ext = std::path::Path::new(&self.video_settings.subtitle_file)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    // Check MP4 + SRT/VTT/ASS soft subtitle incompatibility
                    if output_ext == "mp4" && 
                       (subtitle_ext == "srt" || subtitle_ext == "vtt" || subtitle_ext == "ass") &&
                       self.video_settings.subtitle_mode == "soft" {
                        
                        self.compatibility_warning_message = if self.translations.language == crate::language::Language::Chinese {
                            format!("MP4容器格式不支持{}软字幕。建议更改输出格式为MKV以获得更好的兼容性。", 
                                subtitle_ext.to_uppercase())
                        } else {
                            format!("MP4 container format does not support {} soft subtitles. Recommend changing output format to MKV for better compatibility.", 
                                subtitle_ext.to_uppercase())
                        };
                        self.compatibility_recommended_codec = None;
                        self.compatibility_recommended_format = Some("mkv".to_string());
                        self.show_compatibility_warning = true;
                        return false;
                    }
                }
            }
        }

        true // No compatibility issues found
    }

    fn check_and_reset_incompatible_codecs(&mut self) {
        let current_format = &self.video_settings.container_format;
        
        // Only check if format has actually changed
        if self.last_container_format.is_empty() {
            self.last_container_format = current_format.clone();
            return;
        }
        
        if &self.last_container_format != current_format {
            log_info!("Container format changed from {} to {}, checking codec compatibility...", 
                     self.last_container_format, current_format);
            
            // Check if current audio codec is compatible with new format
            if !self.audio_settings.codec.is_empty() && self.audio_settings.codec != "auto" {
                if let Err(_) = crate::codec_manager::CodecManager::validate_codec_format_compatibility(
                    &self.audio_settings.codec, 
                    current_format, 
                    true
                ) {
                    log_warn!("Audio codec '{}' is incompatible with '{}', selecting best audio codec", 
                             self.audio_settings.codec, current_format);
                    
                    // Select the best audio codec for this format
                    let recommended_audio_codec = crate::codec_manager::CodecManager::get_best_audio_codec_for_format(current_format);
                    self.audio_settings.codec = recommended_audio_codec.clone();
                    
                    // Show a brief status message with translation support
                    self.status_message = if self.translations.language == crate::language::Language::Chinese {
                        format!("音频编码器已自动调整为{}（兼容{}格式）", recommended_audio_codec, current_format)
                    } else {
                        format!("Audio codec automatically adjusted to {} for {} format compatibility", recommended_audio_codec, current_format)
                    };
                }
            }
            
            // Check if current video codec is compatible with new format
            if !self.video_settings.codec.is_empty() && self.video_settings.codec != "auto" {
                if let Err(_) = crate::codec_manager::CodecManager::validate_codec_format_compatibility(
                    &self.video_settings.codec, 
                    current_format, 
                    false
                ) {
                    log_warn!("Video codec '{}' is incompatible with '{}', selecting best video codec", 
                             self.video_settings.codec, current_format);
                    
                    // Use smart encoder recommendation if hardware encoders are available
                    let (recommended_video_codec, _reason) = if let Some(ref hardware_encoders) = self.cached_hardware_encoders {
                        crate::codec_manager::CodecManager::get_smart_encoder_recommendation(
                            current_format,
                            "Balanced", // Default quality preset
                            false, // Not speed priority
                            hardware_encoders,
                            self.translations.language == crate::language::Language::Chinese
                        )
                    } else {
                        (crate::codec_manager::CodecManager::get_best_video_codec_for_format(current_format), String::new())
                    };
                    
                    self.video_settings.codec = recommended_video_codec.clone();
                    
                    // Show a brief status message with translation support
                    self.status_message = if self.translations.language == crate::language::Language::Chinese {
                        format!("视频编码器已自动调整为{}（兼容{}格式）", recommended_video_codec, current_format)
                    } else {
                        format!("Video codec automatically adjusted to {} for {} format compatibility", recommended_video_codec, current_format)
                    };
                }
            }
            
            // Update the tracked format
            self.last_container_format = current_format.clone();
        }
    }

    fn apply_compatibility_recommendation(&mut self) {
        if let Some(ref codec) = self.compatibility_recommended_codec {
            // Check if we're dealing with video or audio based on the current codec being checked
            // Support both English and Chinese messages
            if self.compatibility_warning_message.contains("Video codec") || 
               self.compatibility_warning_message.contains("视频编码器") {
                self.video_settings.codec = codec.clone();
            } else if self.compatibility_warning_message.contains("Audio codec") || 
                      self.compatibility_warning_message.contains("音频编码器") ||
                      self.compatibility_warning_message.contains("WebM") {
                // WebM format warning is about audio codec compatibility
                self.audio_settings.codec = codec.clone();
            }
        }

        if let Some(ref format) = self.compatibility_recommended_format {
            // Apply recommended format
            if self.compatibility_warning_message.contains("Video") || 
               self.compatibility_warning_message.contains("视频") {
                self.video_settings.container_format = format.clone();
            } else if self.compatibility_warning_message.contains("Audio") || 
                      self.compatibility_warning_message.contains("音频") {
                self.audio_settings.format = format.clone();
            } else if self.compatibility_warning_message.contains("MP4") && 
                      (self.compatibility_warning_message.contains("subtitle") || 
                       self.compatibility_warning_message.contains("字幕")) {
                // Handle subtitle compatibility format change
                self.video_settings.container_format = format.clone();
                
                // Also update the output file extension if it ends with .mp4
                if self.output_file.ends_with(".mp4") {
                    self.output_file = self.output_file.replace(".mp4", &format!(".{}", format));
                }
            }
        }
        
        // Update status message to show the recommendation was applied
        self.status_message = if self.translations.language == crate::language::Language::Chinese {
            "已应用推荐设置".to_string()
        } else {
            "Applied recommended settings".to_string()
        };
    }

    fn show_command_preview(&mut self, operation: &OperationType) {
        // Check compatibility before generating preview
        if !self.check_compatibility_and_warn() {
            // Store context so we can continue if user chooses to
            self.compatibility_warning_context = CompatibilityWarningContext::PreviewCommand(operation.clone());
            return; // Show warning dialog, don't generate preview
        }

        self.force_show_command_preview(operation);
    }

    fn generate_output_filename_from_video(&mut self) {
        if !self.dedicated_video_file.is_empty() {
            let video_path = std::path::Path::new(&self.dedicated_video_file);
            if let Some(parent) = video_path.parent() {
                // Generate timestamp in YYYYMMDD_HHMMSS format
                let timestamp = self.generate_timestamp();
                
                // Get appropriate extension based on current operation and settings
                let extension = self.get_output_extension_for_current_operation();
                
                // Create output filename with timestamp
                let output_filename = format!("output_{}.{}", timestamp, extension);
                self.output_file = parent.join(output_filename).display().to_string();
            }
        }
    }
    
    fn generate_output_filename_from_input(&mut self) {
        if !self.input_files.is_empty() {
            let input_path = std::path::Path::new(&self.input_files[0]);
            if let Some(parent) = input_path.parent() {
                // Generate timestamp in YYYYMMDD_HHMMSS format
                let timestamp = self.generate_timestamp();
                
                // Get appropriate extension based on current operation and settings
                let extension = self.get_output_extension_for_current_operation();
                
                // Create output filename with timestamp
                let output_filename = format!("output_{}.{}", timestamp, extension);
                self.output_file = parent.join(output_filename).display().to_string();
            }
        }
    }
    
    fn generate_timestamp(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        // Use millisecond timestamp to ensure uniqueness
        let millis = now.as_millis();
        
        // Add a random suffix to further ensure uniqueness
        let random_suffix = uuid::Uuid::new_v4().as_u128() % 1000;
        
        format!("{}_{}", millis, random_suffix)
    }
    
    fn is_auto_generated_filename(&self, filename: &str) -> bool {
        // Check if filename looks like our auto-generated format: output_timestamp.ext
        if let Some(file_name) = std::path::Path::new(filename).file_name().and_then(|n| n.to_str()) {
            file_name.starts_with("output_") && file_name.contains('.')
        } else {
            false
        }
    }
    
    fn get_output_extension_for_current_operation(&self) -> String {
        match &self.current_operation {
            Some(operation) => {
                match operation {
                    OperationType::VideoConvert | OperationType::VideoCompress | 
                    OperationType::VideoResize | OperationType::VideoCrop | 
                    OperationType::VideoRotate | OperationType::VideoFilter |
                    OperationType::AddSubtitle | OperationType::AddWatermark => {
                        self.video_settings.container_format.clone()
                    },
                    OperationType::AudioConvert | OperationType::AudioCompress | 
                    OperationType::AudioResample | OperationType::AudioVolume | 
                    OperationType::AudioTrim | OperationType::AudioMerge => {
                        self.audio_settings.format.clone()
                    },
                    OperationType::FrameExtract => {
                        self.video_settings.frame_format.clone()
                    },
                    OperationType::VideoToGif | OperationType::GifResize => {
                        "gif".to_string()
                    },
                    _ => "mp4".to_string()
                }
            },
            None => "mp4".to_string()
        }
    }

    fn force_show_command_preview(&mut self, operation: &OperationType) {
        // Prepare input files based on operation type
        let input_files = match operation {
            OperationType::AddSubtitle | OperationType::AddWatermark => {
                // Use dedicated video file for these operations
                vec![self.dedicated_video_file.clone()]
            }
            _ => {
                // Use regular input files for other operations
                self.input_files.clone()
            }
        };
        
        let task = ProcessingTask {
            id: 0,
            operation: operation.clone(),
            input_files,
            output_file: self.output_file.clone(),
            video_settings: Some(self.video_settings.clone()),
            audio_settings: Some(self.audio_settings.clone()),
            progress: 0.0,
            status: TaskStatus::Pending,
            error_message: None,
            start_time: None,
            estimated_total_time: None,
            completion_time: None,
        };
        
        match TaskExecutor::preview_command(&task) {
            Ok(command) => {
                self.command_preview = command;
                self.status_message = self.translations.command_generated();
            }
            Err(e) => {
                self.command_preview = format!("{}: {}", self.translations.command_generation_error(), e);
                self.status_message = self.translations.command_generation_failed();
            }
        }
    }
    

    fn open_workflow_window(&mut self, _ctx: &egui::Context) {
        log_debug!("Opening workflow window...");
        
        // Ensure there is a workflow to display
        if self.current_workflow.is_none() {
            log_debug!("Creating new workflow...");
            self.current_workflow = Some(automation_flow::AutomationWorkflow::new("New Workflow".to_string()));
        }
        
        // Set window open state
        self.workflow_window_open = true;
        self.show_automation_editor = true;
        
        log_debug!("Workflow window state set to open");
    }
    
    fn show_persistent_workflow_window(&mut self, ctx: &egui::Context) {
        if !self.workflow_window_open {
            return;
        }
        
        let viewport_id = egui::ViewportId::from_hash_of("workflow_editor");
        
        let workflow_name = if let Some(ref workflow) = self.current_workflow {
            format!("🔧 Automation Workflow Editor - {}", workflow.name)
        } else {
            "🔧 Automation Workflow Editor - No Workflow".to_string()
        };
        
        let viewport_builder = egui::ViewportBuilder::default()
            .with_title(workflow_name)
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_position([100.0, 100.0]);  // Set initial position to avoid completely covering main window
        
        let mut should_close = false;
        
        ctx.show_viewport_immediate(
            viewport_id,
            viewport_builder,
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Toolbar
                    ui.horizontal(|ui| {
                        if ui.button("🆕 New Workflow").clicked() {
                            self.save_workflow_state();
                            self.current_workflow = Some(automation_flow::AutomationWorkflow::new("New Workflow".to_string()));
                        }
                        
                        if ui.button("💾 Save").clicked() {
                            self.save_workflow();
                        }
                        
                        if ui.button("📁 Load").clicked() {
                            self.load_workflow();
                        }
                        
                        ui.separator();
                        
                        let can_undo = self.history_index > 0;
                        let can_redo = self.history_index < (self.workflow_history.len() as isize - 1);
                        
                        if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                            self.undo_workflow();
                        }
                        
                        if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                            self.redo_workflow();
                        }
                        
                        ui.separator();
                        
                        let has_selection = self.selected_node.is_some();
                        if ui.add_enabled(has_selection, egui::Button::new("🚮 Delete")).clicked() {
                            self.delete_selected_node();
                        }
                        
                        ui.separator();
                        
                        if ui.button("🚀 Execute").clicked() {
                            self.execute_workflow();
                        }
                        
                        ui.separator();
                        
                        if ui.button("❌ Close").clicked() {
                            should_close = true;
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("💡 Press Ctrl+W to toggle this window");
                        });
                    });
                    
                    ui.separator();
                    
                    // Add node menu - comprehensive version
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Add Nodes:");
                        
                        ui.menu_button("📁 Input/Output", |ui| {
                            if ui.button(automation_flow::NodeType::InputFile.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::InputFile, egui::pos2(100.0, 100.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::OutputFile.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::OutputFile, egui::pos2(600.0, 100.0));
                                ui.close_menu();
                            }
                        });
                        
                        ui.menu_button("🎵 Audio Processing", |ui| {
                            ui.vertical(|ui| {
                                ui.label("Basic Operations:");
                                if ui.small_button(automation_flow::NodeType::ExtractAudio.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::ExtractAudio, egui::pos2(200.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioConvert.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioConvert, egui::pos2(200.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioCompress.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioCompress, egui::pos2(200.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioResample.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioResample, egui::pos2(200.0, 300.0));
                                    ui.close_menu();
                                }
                                ui.separator();
                                ui.label("Audio Effects:");
                                if ui.small_button(automation_flow::NodeType::AudioVolume.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioVolume, egui::pos2(200.0, 350.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioTrim.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioTrim, egui::pos2(200.0, 400.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioMerge.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioMerge, egui::pos2(200.0, 450.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioNormalize.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioNormalize, egui::pos2(200.0, 500.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioDeNoise.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioDeNoise, egui::pos2(200.0, 550.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioEqualizer.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioEqualizer, egui::pos2(200.0, 600.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioFade.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioFade, egui::pos2(200.0, 650.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioEcho.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioEcho, egui::pos2(200.0, 700.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::AudioSpeed.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioSpeed, egui::pos2(200.0, 750.0));
                                    ui.close_menu();
                                }
                            });
                        });
                        
                        ui.menu_button("📺 Video Processing", |ui| {
                            ui.vertical(|ui| {
                                ui.label("Basic Operations:");
                                if ui.small_button(automation_flow::NodeType::ExtractVideo.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::ExtractVideo, egui::pos2(300.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoConvert.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoConvert, egui::pos2(300.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoCompress.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoCompress, egui::pos2(300.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoRecode.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoRecode, egui::pos2(300.0, 300.0));
                                    ui.close_menu();
                                }
                                ui.separator();
                                ui.label("Video Transformations:");
                                if ui.small_button(automation_flow::NodeType::VideoResize.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoResize, egui::pos2(300.0, 350.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoCrop.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoCrop, egui::pos2(300.0, 400.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoRotate.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoRotate, egui::pos2(300.0, 450.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoFilter.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoFilter, egui::pos2(300.0, 500.0));
                                    ui.close_menu();
                                }
                                ui.separator();
                                ui.label("Video Effects:");
                                if ui.small_button(automation_flow::NodeType::FrameExtract.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::FrameExtract, egui::pos2(300.0, 550.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoFPS.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoFPS, egui::pos2(300.0, 600.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoStabilize.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoStabilize, egui::pos2(300.0, 650.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoDeinterlace.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoDeinterlace, egui::pos2(300.0, 700.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoColorCorrect.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoColorCorrect, egui::pos2(300.0, 750.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoBrightness.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoBrightness, egui::pos2(300.0, 800.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoSaturation.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoSaturation, egui::pos2(300.0, 850.0));
                                    ui.close_menu();
                                }
                                if ui.small_button(automation_flow::NodeType::VideoGamma.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoGamma, egui::pos2(300.0, 900.0));
                                    ui.close_menu();
                                }
                            });
                        });
                        
                        ui.menu_button("🎭 Audio/Video Operations", |ui| {
                            ui.vertical(|ui| {
                                if ui.button(automation_flow::NodeType::Combine.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::Combine, egui::pos2(400.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::SplitAudioVideo.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::SplitAudioVideo, egui::pos2(400.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoOverlay.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoOverlay, egui::pos2(400.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoPiP.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoPiP, egui::pos2(400.0, 300.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoSideBySide.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoSideBySide, egui::pos2(400.0, 350.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::AudioVideoSync.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AudioVideoSync, egui::pos2(400.0, 400.0));
                                    ui.close_menu();
                                }
                            });
                        });
                        
                        ui.menu_button("🎨 Text & Graphics", |ui| {
                            ui.vertical(|ui| {
                                if ui.button(automation_flow::NodeType::AddSubtitle.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AddSubtitle, egui::pos2(500.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::AddWatermark.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AddWatermark, egui::pos2(500.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::AddText.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AddText, egui::pos2(500.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::AddLogo.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AddLogo, egui::pos2(500.0, 300.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::AddTimecode.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::AddTimecode, egui::pos2(500.0, 350.0));
                                    ui.close_menu();
                                }
                            });
                        });
                        
                        ui.menu_button("🔄 Format Conversion", |ui| {
                            ui.vertical(|ui| {
                                if ui.button(automation_flow::NodeType::VideoToGif.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoToGif, egui::pos2(600.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::GifResize.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::GifResize, egui::pos2(600.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoToImages.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoToImages, egui::pos2(600.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::ImagesToVideo.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::ImagesToVideo, egui::pos2(600.0, 300.0));
                                    ui.close_menu();
                                }
                            });
                        });
                        
                        ui.menu_button("📦 Batch & Advanced", |ui| {
                            ui.vertical(|ui| {
                                if ui.button(automation_flow::NodeType::BatchConvert.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::BatchConvert, egui::pos2(700.0, 150.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::BatchProcess.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::BatchProcess, egui::pos2(700.0, 200.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::MultiPassEncode.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::MultiPassEncode, egui::pos2(700.0, 250.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::QualityAnalysis.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::QualityAnalysis, egui::pos2(700.0, 300.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::FormatValidation.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::FormatValidation, egui::pos2(700.0, 350.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::StreamPrep.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::StreamPrep, egui::pos2(700.0, 400.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoEncrypt.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoEncrypt, egui::pos2(700.0, 450.0));
                                    ui.close_menu();
                                }
                                if ui.button(automation_flow::NodeType::VideoDecrypt.display_name()).clicked() {
                                    self.add_automation_node(automation_flow::NodeType::VideoDecrypt, egui::pos2(700.0, 500.0));
                                    ui.close_menu();
                                }
                            });
                        });
                    });
                    
                    ui.separator();
                    
                    // Main canvas area - reserve space for UI elements below
                    let mut canvas_rect = ui.available_rect_before_wrap();
                    canvas_rect.max.y -= 80.0;  // Reserve 80 pixels for status bar/hints below
                    let canvas_response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());
                    
                    // Draw grid
                    self.draw_grid(ui, canvas_rect);
                    
                    // Handle canvas interaction
                    self.handle_canvas_interaction(ui, canvas_rect, &canvas_response);
                    
                    // Handle keyboard shortcuts
                    ctx.input(|i| {
                        if i.key_pressed(egui::Key::Escape) {
                            if self.creating_connection {
                                self.creating_connection = false;
                                self.connection_source_node = None;
                                self.connection_source_port = None;
                                log_debug!("Connection cancelled by ESC key");
                            }
                        }
                        
                        if i.key_pressed(egui::Key::Delete) {
                            self.delete_selected_node();
                        }
                        
                        if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                            self.undo_workflow();
                        }
                        
                        if (i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) || 
                           (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
                            self.redo_workflow();
                        }
                    });
                    
                    // Draw nodes
                    let workflow_nodes = if let Some(ref workflow) = self.current_workflow {
                        workflow.nodes.values().cloned().collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };
                    
                    for node in workflow_nodes {
                        self.draw_single_node_with_canvas(ui, &node, canvas_rect);
                    }
                    
                    // Draw connection lines
                    if let Some(ref workflow) = self.current_workflow {
                        let workflow_clone = workflow.clone();
                        self.draw_connections_with_canvas(ui, &workflow_clone, canvas_rect);
                    }
                    
                    // Status bar - display important hint information
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("💡 Tips:");
                        if self.creating_connection {
                            ui.label("Click input port (left side) to complete connection | ESC to cancel");
                        } else {
                            ui.label("Click output ports (right) to start connections | Del=Delete | Ctrl+Z=Undo");
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(ref workflow) = self.current_workflow {
                                ui.label(format!("Nodes: {} | Connections: {}", 
                                    workflow.nodes.len(), 
                                    workflow.connections.len()));
                            }
                        });
                    });
                    
                    // Bottom information and properties panel
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                ui.label("Shortcuts:");
                                ui.label("Del=Delete | Ctrl+Z=Undo | Ctrl+Y=Redo | ESC=Cancel");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Instructions:");
                                ui.label("Click output ports (right side) to start connections, click input ports (left side) to complete them.");
                            });
                            
                            // Node property editing
                            if let Some(ref selected_id) = self.selected_node.clone() {
                                // Ensure we have the latest hardware encoders BEFORE borrowing workflow
                                let current_hw_encoders = self.get_cached_hardware_encoders();
                                
                                if let Some(ref mut workflow) = self.current_workflow {
                                    if let Some(ref mut node) = workflow.nodes.get_mut(selected_id) {
                                        ui.separator();
                                        ui.heading("Node Properties");
                                        ui.label(format!("Type: {}", node.node_type.display_name()));
                                        
                                        // Use comprehensive parameter UI
                                        node.show_comprehensive_parameters_ui(ui, &self.translations, &current_hw_encoders);
                                        
                                        // Auto-fill output filenames when input files change
                                        if node.node_type == automation_flow::NodeType::InputFile {
                                            if let Some(ref mut workflow) = self.current_workflow {
                                                let workflow_clone = workflow.clone();
                                                let mut changed = false;
                                                for (_, output_node) in workflow.nodes.iter_mut() {
                                                    if output_node.node_type == automation_flow::NodeType::OutputFile {
                                                        if output_node.auto_fill_output_from_input(&workflow_clone) {
                                                            changed = true;
                                                        }
                                                    }
                                                }
                                                if changed {
                                                    self.save_workflow_state();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        });
                });
                
                // Check system window close request
                if ctx.input(|i| i.viewport().close_requested()) {
                    should_close = true;
                }
            },
        );
        
        // Handle closing outside viewport callback
        if should_close {
            self.workflow_window_open = false;
            self.show_automation_editor = false;
            log_debug!("Workflow window closed");
        }
    }

    fn show_automation_editor_window(&mut self, ctx: &egui::Context) {
        let mut window_open = true;
        let workflow_name = if let Some(ref workflow) = self.current_workflow {
            format!("🔧 Automation Workflow Editor - {}", workflow.name)
        } else {
            "🔧 Automation Workflow Editor - No Workflow".to_string()
        };
        
        let mut window = egui::Window::new(workflow_name)
            .default_size(self.workflow_window_size.unwrap_or(egui::vec2(1200.0, 800.0)))
            .min_size([800.0, 600.0])
            .max_size([1600.0, 1200.0])
            .resizable(true)
            .collapsible(false)
            .scroll([false, false])
            .open(&mut window_open);
            
        if let Some(pos) = self.workflow_window_pos {
            window = window.default_pos(pos);
        }
        
        let response = window
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("🆕 New Workflow").clicked() {
                        self.save_workflow_state();
                        self.current_workflow = Some(automation_flow::AutomationWorkflow::new("New Workflow".to_string()));
                    }
                    
                    if ui.button("💾 Save").clicked() {
                        self.save_workflow();
                    }
                    
                    if ui.button("📁 Load").clicked() {
                        self.load_workflow();
                    }
                    
                    ui.separator();
                    
                    let can_undo = self.history_index > 0;
                    let can_redo = self.history_index < (self.workflow_history.len() as isize - 1);
                    
                    if ui.add_enabled(can_undo, egui::Button::new("Undo")).clicked() {
                        self.undo_workflow();
                    }
                    
                    if ui.add_enabled(can_redo, egui::Button::new("Redo")).clicked() {
                        self.redo_workflow();
                    }
                    
                    ui.separator();
                    
                    let has_selection = self.selected_node.is_some();
                    if ui.add_enabled(has_selection, egui::Button::new("🚮 Delete")).clicked() {
                        self.delete_selected_node();
                    }
                    
                    ui.separator();
                    
                    if ui.button("🚀 Execute").clicked() {
                        self.execute_workflow();
                    }
                    
                    ui.separator();
                    
                    if ui.button("❌ Close Editor").clicked() {
                        self.show_automation_editor = false;
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("💡 Press Ctrl+W to toggle this window");
                    });
                });
                
                ui.separator();
                
                ui.horizontal_wrapped(|ui| {
                    ui.label("Add Nodes:");
                    
                    ui.menu_button("📁 Input/Output", |ui| {
                        if ui.button(automation_flow::NodeType::InputFile.display_name()).clicked() {
                            self.add_automation_node(automation_flow::NodeType::InputFile, egui::pos2(100.0, 100.0));
                            ui.close_menu();
                        }
                        if ui.button(automation_flow::NodeType::OutputFile.display_name()).clicked() {
                            self.add_automation_node(automation_flow::NodeType::OutputFile, egui::pos2(600.0, 100.0));
                            ui.close_menu();
                        }
                    });
                    
                    ui.menu_button("🎵 Audio Processing", |ui| {
                        ui.vertical(|ui| {
                            ui.label("Basic Operations:");
                            if ui.small_button(automation_flow::NodeType::ExtractAudio.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::ExtractAudio, egui::pos2(200.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioConvert.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioConvert, egui::pos2(200.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioCompress.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioCompress, egui::pos2(200.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioResample.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioResample, egui::pos2(200.0, 300.0));
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label("Audio Effects:");
                            if ui.small_button(automation_flow::NodeType::AudioVolume.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioVolume, egui::pos2(200.0, 350.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioTrim.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioTrim, egui::pos2(200.0, 400.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioMerge.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioMerge, egui::pos2(200.0, 450.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioNormalize.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioNormalize, egui::pos2(200.0, 500.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioDeNoise.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioDeNoise, egui::pos2(200.0, 550.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioEqualizer.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioEqualizer, egui::pos2(200.0, 600.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioFade.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioFade, egui::pos2(200.0, 650.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioEcho.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioEcho, egui::pos2(200.0, 700.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::AudioSpeed.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioSpeed, egui::pos2(200.0, 750.0));
                                ui.close_menu();
                            }
                        });
                    });
                    
                    ui.menu_button("📺 Video Processing", |ui| {
                        ui.vertical(|ui| {
                            ui.label("Basic Operations:");
                            if ui.small_button(automation_flow::NodeType::ExtractVideo.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::ExtractVideo, egui::pos2(300.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoConvert.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoConvert, egui::pos2(300.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoCompress.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoCompress, egui::pos2(300.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoRecode.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoRecode, egui::pos2(300.0, 300.0));
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label("Video Transformations:");
                            if ui.small_button(automation_flow::NodeType::VideoResize.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoResize, egui::pos2(300.0, 350.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoCrop.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoCrop, egui::pos2(300.0, 400.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoRotate.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoRotate, egui::pos2(300.0, 450.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoFilter.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoFilter, egui::pos2(300.0, 500.0));
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label("Video Effects:");
                            if ui.small_button(automation_flow::NodeType::FrameExtract.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::FrameExtract, egui::pos2(300.0, 550.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoFPS.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoFPS, egui::pos2(300.0, 600.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoStabilize.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoStabilize, egui::pos2(300.0, 650.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoDeinterlace.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoDeinterlace, egui::pos2(300.0, 700.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoColorCorrect.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoColorCorrect, egui::pos2(300.0, 750.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoBrightness.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoBrightness, egui::pos2(300.0, 800.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoSaturation.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoSaturation, egui::pos2(300.0, 850.0));
                                ui.close_menu();
                            }
                            if ui.small_button(automation_flow::NodeType::VideoGamma.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoGamma, egui::pos2(300.0, 900.0));
                                ui.close_menu();
                            }
                        });
                    });
                    
                    ui.menu_button("🎭 Audio/Video Operations", |ui| {
                        ui.vertical(|ui| {
                            if ui.button(automation_flow::NodeType::Combine.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::Combine, egui::pos2(400.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::SplitAudioVideo.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::SplitAudioVideo, egui::pos2(400.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoOverlay.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoOverlay, egui::pos2(400.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoPiP.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoPiP, egui::pos2(400.0, 300.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoSideBySide.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoSideBySide, egui::pos2(400.0, 350.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::AudioVideoSync.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AudioVideoSync, egui::pos2(400.0, 400.0));
                                ui.close_menu();
                            }
                        });
                    });
                    
                    ui.menu_button("🎨 Text & Graphics", |ui| {
                        ui.vertical(|ui| {
                            if ui.button(automation_flow::NodeType::AddSubtitle.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AddSubtitle, egui::pos2(500.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::AddWatermark.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AddWatermark, egui::pos2(500.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::AddText.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AddText, egui::pos2(500.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::AddLogo.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AddLogo, egui::pos2(500.0, 300.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::AddTimecode.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::AddTimecode, egui::pos2(500.0, 350.0));
                                ui.close_menu();
                            }
                        });
                    });
                    
                    ui.menu_button("🔄 Format Conversion", |ui| {
                        ui.vertical(|ui| {
                            if ui.button(automation_flow::NodeType::VideoToGif.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoToGif, egui::pos2(600.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::GifResize.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::GifResize, egui::pos2(600.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoToImages.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoToImages, egui::pos2(600.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::ImagesToVideo.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::ImagesToVideo, egui::pos2(600.0, 300.0));
                                ui.close_menu();
                            }
                        });
                    });
                    
                    ui.menu_button("📦 Batch & Advanced", |ui| {
                        ui.vertical(|ui| {
                            if ui.button(automation_flow::NodeType::BatchConvert.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::BatchConvert, egui::pos2(700.0, 150.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::BatchProcess.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::BatchProcess, egui::pos2(700.0, 200.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::MultiPassEncode.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::MultiPassEncode, egui::pos2(700.0, 250.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::QualityAnalysis.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::QualityAnalysis, egui::pos2(700.0, 300.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::FormatValidation.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::FormatValidation, egui::pos2(700.0, 350.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::StreamPrep.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::StreamPrep, egui::pos2(700.0, 400.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoEncrypt.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoEncrypt, egui::pos2(700.0, 450.0));
                                ui.close_menu();
                            }
                            if ui.button(automation_flow::NodeType::VideoDecrypt.display_name()).clicked() {
                                self.add_automation_node(automation_flow::NodeType::VideoDecrypt, egui::pos2(700.0, 500.0));
                                ui.close_menu();
                            }
                        });
                    });
                });
                
                ui.separator();
                
                let available_rect = ui.available_rect_before_wrap();
                let (canvas_rect, response) = ui.allocate_exact_size(available_rect.size(), egui::Sense::click_and_drag());
                
                ui.painter().rect_filled(
                    canvas_rect,
                    egui::Rounding::same(0.0),
                    egui::Color32::from_gray(45)
                );
                
                let grid_spacing = 25.0;
                let grid_color = egui::Color32::from_gray(60);
                
                let mut x = canvas_rect.min.x;
                while x <= canvas_rect.max.x {
                    ui.painter().line_segment(
                        [egui::pos2(x, canvas_rect.min.y), egui::pos2(x, canvas_rect.max.y)],
                        egui::Stroke::new(0.5, grid_color)
                    );
                    x += grid_spacing;
                }
                
                let mut y = canvas_rect.min.y;
                while y <= canvas_rect.max.y {
                    ui.painter().line_segment(
                        [egui::pos2(canvas_rect.min.x, y), egui::pos2(canvas_rect.max.x, y)],
                        egui::Stroke::new(0.5, grid_color)
                    );
                    y += grid_spacing;
                }
                
                self.handle_canvas_interaction(ui, canvas_rect, &response);
                
                let workflow_nodes = if let Some(ref workflow) = self.current_workflow {
                    workflow.nodes.values().cloned().collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                
                for node in workflow_nodes {
                    self.draw_single_node_with_canvas(ui, &node, canvas_rect);
                }
                
                if let Some(ref workflow) = self.current_workflow {
                    let workflow_clone = workflow.clone();
                    self.draw_connections_with_canvas(ui, &workflow_clone, canvas_rect);
                }
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.label("Shortcuts:");
                            ui.label("Del=Delete | Ctrl+Z=Undo | Ctrl+Y=Redo | Ctrl+A=Select All | ESC=Cancel");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Instructions:");
                            ui.label("Click output ports (right side) to start connections, click input ports (left side) to complete them.");
                        });
                        
                        if let Some(ref selected_id) = self.selected_node.clone() {
                            // Ensure we have the latest hardware encoders BEFORE borrowing workflow
                            let current_hw_encoders = self.get_cached_hardware_encoders();
                            
                            if let Some(ref mut workflow) = self.current_workflow {
                                if let Some(ref mut node) = workflow.nodes.get_mut(selected_id) {
                                    ui.separator();
                                    ui.heading("Node Properties");
                                    ui.label(format!("Type: {}", node.node_type.display_name()));
                                    
                                    // Use comprehensive parameter UI
                                    node.show_comprehensive_parameters_ui(ui, &self.translations, &current_hw_encoders);
                                    
                                    // Auto-fill output filenames when input files change
                                    if node.node_type == automation_flow::NodeType::InputFile {
                                        let workflow_clone = workflow.clone();
                                        let mut changed = false;
                                        for (_, output_node) in workflow.nodes.iter_mut() {
                                            if output_node.node_type == automation_flow::NodeType::OutputFile {
                                                if output_node.auto_fill_output_from_input(&workflow_clone) {
                                                    changed = true;
                                                }
                                            }
                                        }
                                        if changed {
                                            self.save_workflow_state();
                                        }
                                    }
                                }
                            }
                        } else {
                            ui.separator();
                            ui.label("Select a node to edit its properties");
                        }
                    });
            });
        
        // Save window state and handle window close events
        if let Some(response) = response {
            let rect = response.response.rect;
            self.workflow_window_pos = Some(rect.min);
            self.workflow_window_size = Some(rect.size());
        }
        
        if !window_open {
            self.show_automation_editor = false;
        }
    }
    
    fn add_automation_node(&mut self, node_type: automation_flow::NodeType, position: egui::Pos2) {
        self.save_workflow_state();
        
        if let Some(ref mut workflow) = self.current_workflow {
            let node_id = format!("node_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
            let node = automation_flow::AutomationNode::new(node_id, node_type, position);
            workflow.add_node(node);
        }
    }
    
    fn draw_grid(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        let grid_size = 20.0 * self.workflow_canvas_zoom;
        
        let start_x = (rect.min.x / grid_size).floor() * grid_size;
        let start_y = (rect.min.y / grid_size).floor() * grid_size;
        
        let mut x = start_x;
        while x <= rect.max.x {
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(50))
            );
            x += grid_size;
        }
        
        let mut y = start_y;
        while y <= rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(50))
            );
            y += grid_size;
        }
    }
    
    fn draw_connections_with_canvas(&mut self, ui: &mut egui::Ui, workflow: &automation_flow::AutomationWorkflow, canvas_rect: egui::Rect) {
        let painter = ui.painter();
        
        let mut connections_to_delete = Vec::new();
        for connection in workflow.connections.values() {
            if let (Some(from_node), Some(to_node)) = (
                workflow.nodes.get(&connection.from_node),
                workflow.nodes.get(&connection.to_node)
            ) {
                let from_pos = egui::pos2(
                    canvas_rect.min.x + from_node.position.x + from_node.size.x + 5.0,
                    canvas_rect.min.y + from_node.position.y + 35.0 + (connection.from_port as f32 * 25.0)
                );
                let to_pos = egui::pos2(
                    canvas_rect.min.x + to_node.position.x - 5.0,
                    canvas_rect.min.y + to_node.position.y + 35.0 + (connection.to_port as f32 * 25.0)
                );
                
                self.draw_connection_line(&painter, from_pos, to_pos, connection.data_type.get_color());
                
                if self.is_point_near_connection_line(from_pos, to_pos, ui.ctx().input(|i| i.pointer.latest_pos().unwrap_or_default())) {
                    if ui.ctx().input(|i| i.pointer.secondary_clicked()) {
                        connections_to_delete.push(connection.id.clone());
                    }
                    
                    if ui.ctx().input(|i| i.pointer.latest_pos().map_or(false, |pos| 
                        self.is_point_near_connection_line(from_pos, to_pos, pos))) {
                        let mid_point = egui::pos2((from_pos.x + to_pos.x) / 2.0, (from_pos.y + to_pos.y) / 2.0);
                        painter.circle_filled(
                            mid_point,
                            4.0,
                            egui::Color32::from_rgb(255, 100, 100)
                        );
                    }
                }
            }
        }
        
        if let Some(ref mut workflow_mut) = self.current_workflow {
            for conn_id in connections_to_delete {
                workflow_mut.remove_connection(&conn_id);
            }
        }
        
        if self.creating_connection {
            if let (Some(ref source_node_id), Some(source_port_idx)) = 
                (&self.connection_source_node, self.connection_source_port) {
                if let Some(source_node) = workflow.nodes.get(source_node_id) {
                    let from_pos = egui::pos2(
                        canvas_rect.min.x + source_node.position.x + source_node.size.x + 5.0,
                        canvas_rect.min.y + source_node.position.y + 35.0 + (source_port_idx as f32 * 25.0)
                    );
                    
                    let port_data_type = &source_node.output_ports[source_port_idx].data_type;
                    self.draw_connection_line(&painter, from_pos, self.connection_target_pos, port_data_type.get_color());
                    
                    painter.circle_filled(
                        self.connection_target_pos,
                        4.0,
                        port_data_type.get_color()
                    );
                }
            }
        }
    }
    
    fn draw_connections(&mut self, ui: &mut egui::Ui, workflow: &automation_flow::AutomationWorkflow) {
        let painter = ui.painter();
        
        let mut connections_to_delete = Vec::new();
        for connection in workflow.connections.values() {
            if let (Some(from_node), Some(to_node)) = (
                workflow.nodes.get(&connection.from_node),
                workflow.nodes.get(&connection.to_node)
            ) {
                if let (Some(from_pos), Some(to_pos)) = (
                    from_node.get_output_port_position(connection.from_port),
                    to_node.get_input_port_position(connection.to_port)
                ) {
                    self.draw_connection_line(&painter, from_pos, to_pos, connection.data_type.get_color());
                    
                    if self.is_point_near_connection_line(from_pos, to_pos, ui.ctx().input(|i| i.pointer.latest_pos().unwrap_or_default())) {
                        if ui.ctx().input(|i| i.pointer.secondary_clicked()) {
                            connections_to_delete.push(connection.id.clone());
                        }
                        
                        if ui.ctx().input(|i| i.pointer.latest_pos().map_or(false, |pos| 
                            self.is_point_near_connection_line(from_pos, to_pos, pos))) {
                            let mid_point = egui::pos2((from_pos.x + to_pos.x) / 2.0, (from_pos.y + to_pos.y) / 2.0);
                            painter.circle_filled(
                                mid_point,
                                4.0,
                                egui::Color32::from_rgb(255, 100, 100)
                            );
                        }
                    }
                }
            }
        }
        
        if let Some(ref mut workflow_mut) = self.current_workflow {
            for conn_id in connections_to_delete {
                workflow_mut.remove_connection(&conn_id);
            }
        }
        
        if self.creating_connection {
            if let (Some(ref source_node_id), Some(source_port_idx)) = 
                (&self.connection_source_node, self.connection_source_port) {
                if let Some(source_node) = workflow.nodes.get(source_node_id) {
                    if let Some(from_pos) = source_node.get_output_port_position(source_port_idx) {
                        let port_data_type = &source_node.output_ports[source_port_idx].data_type;
                        self.draw_connection_line(&painter, from_pos, self.connection_target_pos, port_data_type.get_color());
                        
                        painter.circle_filled(
                            self.connection_target_pos,
                            4.0,
                            port_data_type.get_color()
                        );
                    }
                }
            }
        }
    }
    
    fn draw_connection_line(&self, painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, color: egui::Color32) {

        let dx = to.x - from.x;
        let control_offset = dx.abs().min(100.0).max(30.0);
        
        let control1 = egui::pos2(from.x + control_offset, from.y);
        let control2 = egui::pos2(to.x - control_offset, to.y);
        
        let mut points = Vec::new();
        let num_segments = 20;
        
        for i in 0..=num_segments {
            let t = i as f32 / num_segments as f32;
            let point = cubic_bezier(from, control1, control2, to, t);
            points.push(point);
        }
        
        for i in 0..points.len() - 1 {
            painter.line_segment(
                [points[i], points[i + 1]],
                egui::Stroke::new(3.0, color)
            );
        }
    }
    
    
    fn draw_single_node_with_canvas(&mut self, ui: &mut egui::Ui, node: &automation_flow::AutomationNode, canvas_rect: egui::Rect) {
        let painter = ui.painter();
        let node_screen_pos = egui::pos2(
            canvas_rect.min.x + node.position.x,
            canvas_rect.min.y + node.position.y
        );
        let node_rect = egui::Rect::from_min_size(node_screen_pos, node.size);
        
        let is_selected = self.selected_node.as_ref() == Some(&node.id);
        let mut bg_color = if is_selected {
            node.node_type.get_color().gamma_multiply(1.3)
        } else {
            node.node_type.get_color()
        };
        
        if node.node_type == automation_flow::NodeType::InputFile {
            let has_file = node.parameters.get("file_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            if !has_file {
                bg_color = bg_color.gamma_multiply(0.6);
            }
        } else if node.node_type == automation_flow::NodeType::OutputFile {
            let has_path = node.parameters.get("output_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            if !has_path {
                bg_color = bg_color.gamma_multiply(0.6);
            }
        }
        
        painter.rect_filled(
            node_rect,
            egui::Rounding::same(8.0),
            bg_color
        );
        
        let border_stroke = if node.node_type == automation_flow::NodeType::InputFile {
            let has_file = node.parameters.get("file_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            if has_file {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)) 
            } else {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 100, 100)) 
            }
        } else if node.node_type == automation_flow::NodeType::OutputFile {
            let has_path = node.parameters.get("output_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            if has_path {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 100)) 
            } else {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 100, 100)) 
            }
        } else {
            egui::Stroke::new(2.0, egui::Color32::from_gray(200))
        };
        
        painter.rect_stroke(
            node_rect,
            egui::Rounding::same(8.0),
            border_stroke
        );
        
        painter.text(
            egui::pos2(node_rect.center().x, node_rect.min.y + 15.0),
            egui::Align2::CENTER_TOP,
            node.node_type.display_name(),
            egui::FontId::default(),
            egui::Color32::WHITE
        );
        
        if node.node_type == automation_flow::NodeType::InputFile {
            let has_file = node.parameters.get("file_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            let status_text = if has_file {
                if let Some(file_path) = node.parameters.get("file_path") {
                    let filename = std::path::Path::new(&file_path.value)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    format!("{}", filename)
                } else {
                    "File loaded".to_string()
                }
            } else {
                "📥 Drop file here".to_string()
            };
            
            painter.text(
                egui::pos2(node_rect.center().x, node_rect.min.y + 35.0),
                egui::Align2::CENTER_TOP,
                status_text,
                egui::FontId::proportional(10.0),
                if has_file { egui::Color32::from_rgb(200, 255, 200) } else { egui::Color32::from_rgb(255, 200, 200) }
            );
        } else if node.node_type == automation_flow::NodeType::OutputFile {
            let has_path = node.parameters.get("output_path")
                .map(|p| !p.value.is_empty())
                .unwrap_or(false);
            
            let status_text = if has_path {
                if let Some(output_path) = node.parameters.get("output_path") {
                    let filename = std::path::Path::new(&output_path.value)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    format!("💾 {}", filename)
                } else {
                    "💾 Path set".to_string()
                }
            } else {
                "📥 Drop file here".to_string()
            };
            
            painter.text(
                egui::pos2(node_rect.center().x, node_rect.min.y + 35.0),
                egui::Align2::CENTER_TOP,
                status_text,
                egui::FontId::proportional(10.0),
                if has_path { egui::Color32::from_rgb(200, 255, 200) } else { egui::Color32::from_rgb(255, 200, 200) }
            );
        }
        
        let node_response = ui.interact(node_rect, egui::Id::new(format!("node_{}", node.id)), egui::Sense::click_and_drag());
        if node_response.clicked() {
            self.selected_node = Some(node.id.clone());
            log_debug!("Node {} selected", node.id);
        }
        
        node_response.context_menu(|ui| {
            ui.label(format!("Node: {}", node.node_type.display_name()));
            ui.separator();
            
            if node.node_type == automation_flow::NodeType::InputFile {
                if ui.button("📁 Browse File...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Media Files", &["mp4", "avi", "mov", "mkv", "mp3", "wav", "flac", "aac"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        if let Some(ref mut workflow) = self.current_workflow {
                            if let Some(node_mut) = workflow.nodes.get_mut(&node.id) {
                                if let Some(param) = node_mut.parameters.get_mut("file_path") {
                                    param.value = path.display().to_string();
                                    log_debug!("Set input file path: {}", param.value);
                                }
                            }
                        }
                    }
                    ui.close_menu();
                }
                ui.separator();
            }
            
            if node.node_type == automation_flow::NodeType::OutputFile {
                if ui.button("💾 Save As...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video Files", &["mp4", "mkv", "avi", "mov", "webm"])
                        .add_filter("Audio Files", &["mp3", "wav", "flac", "aac", "ogg"])
                        .add_filter("All Files", &["*"])
                        .save_file()
                    {
                        if let Some(ref mut workflow) = self.current_workflow {
                            if let Some(node_mut) = workflow.nodes.get_mut(&node.id) {
                                if let Some(param) = node_mut.parameters.get_mut("output_path") {
                                    param.value = path.display().to_string();
                                    log_debug!("Set output file path: {}", param.value);
                                }
                            }
                        }
                    }
                    ui.close_menu();
                }
                ui.separator();
            }
            
            if ui.button("🚮 Delete Node").clicked() {
                self.selected_node = Some(node.id.clone());
                self.delete_selected_node();
                ui.close_menu();
            }
            
            ui.separator();
            ui.label("Parameters:");
            
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    // Ensure we have the latest hardware encoders BEFORE borrowing workflow
                    let current_hw_encoders = self.get_cached_hardware_encoders();
                    
                    // Use comprehensive parameter UI for context menu
                    if let Some(ref mut workflow) = self.current_workflow {
                        if let Some(node_mut) = workflow.nodes.get_mut(&node.id) {
                            node_mut.show_comprehensive_parameters_ui(
                                ui, 
                                &self.translations, 
                                &current_hw_encoders
                            );
                            
                            // Auto-fill output filenames when input files change
                            if node_mut.node_type == automation_flow::NodeType::InputFile {
                                // Trigger auto-fill for all connected output nodes
                                let workflow_clone = workflow.clone();
                                let mut changed = false;
                                for (_, output_node) in workflow.nodes.iter_mut() {
                                    if output_node.node_type == automation_flow::NodeType::OutputFile {
                                        if output_node.auto_fill_output_from_input(&workflow_clone) {
                                            changed = true;
                                        }
                                    }
                                }
                                if changed {
                                    self.save_workflow_state();
                                }
                            }
                        }
                    }
                });
        });
        
        if node_response.drag_started() {
            self.save_workflow_state();
        }
        
        if node_response.dragged() {
            if let Some(ref mut workflow) = self.current_workflow {
                if let Some(ref mut node_mut) = workflow.nodes.get_mut(&node.id) {
                    node_mut.position += node_response.drag_delta();
                }
            }
        }
        
        let mut port_interactions = Vec::new();
        
        for (i, port) in node.input_ports.iter().enumerate() {
            let port_pos = egui::pos2(
                node_screen_pos.x - 5.0,
                node_screen_pos.y + 35.0 + (i as f32 * 25.0)
            );
            
            let is_potential_target = self.creating_connection && 
                self.connection_source_node.as_ref() != Some(&node.id);

            let port_color = if is_potential_target {
                port.data_type.get_color().gamma_multiply(1.5)
            } else {
                port.data_type.get_color()
            };
            
            painter.circle_filled(
                port_pos,
                6.0,
                port_color
            );
            

            painter.circle_stroke(
                port_pos,
                6.0,
                egui::Stroke::new(1.0, egui::Color32::WHITE)
            );
            
            painter.text(
                egui::pos2(port_pos.x + 15.0, port_pos.y),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::monospace(10.0),
                egui::Color32::WHITE
            );
            
            let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(20.0, 20.0));
            let response = ui.interact(port_rect, egui::Id::new(format!("input_port_{}_{}", node.id, i)), egui::Sense::click());
            
            if response.clicked() && self.creating_connection {
                port_interactions.push((node.id.clone(), i, port.data_type.clone()));
                log_debug!("Clicked input port {} on node {}", i, node.id);
            }
        }
        
        for (target_node_id, target_port_idx, target_data_type) in port_interactions {
            if let (Some(ref source_node_id), Some(source_port_idx)) = 
                (&self.connection_source_node, self.connection_source_port) {
                
                log_debug!("Attempting to connect {} port {} -> {} port {}", 
                    source_node_id, source_port_idx, target_node_id, target_port_idx);
                
                if source_node_id != &target_node_id {
                    if let Some(ref mut workflow) = self.current_workflow {
                        if let Some(source_node) = workflow.nodes.get(source_node_id) {
                            if source_port_idx < source_node.output_ports.len() {
                                let source_data_type = &source_node.output_ports[source_port_idx].data_type;
                                
                                log_debug!("Data types: {:?} -> {:?}", source_data_type, target_data_type);
                                
                                if automation_flow::AutomationWorkflow::are_types_compatible(source_data_type, &target_data_type) {
                                    let connection = automation_flow::NodeConnection {
                                        id: format!("conn_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                                        from_node: source_node_id.clone(),
                                        from_port: source_port_idx,
                                        to_node: target_node_id,
                                        to_port: target_port_idx,
                                        data_type: source_data_type.clone(),
                                    };
                                    
                                    match workflow.add_connection(connection) {
                                        Ok(_) => {
                                            log_debug!("Connection created successfully");
                                            
                                            // Trigger auto-fill for all output nodes when connections are made
                                            let workflow_clone = workflow.clone();
                                            for (_, output_node) in workflow.nodes.iter_mut() {
                                                if output_node.node_type == automation_flow::NodeType::OutputFile {
                                                    output_node.auto_fill_output_from_input(&workflow_clone);
                                                }
                                            }
                                            
                                            self.save_workflow_state();
                                        },
                                        Err(e) => {
                                            log_error!("❌ Connection failed: {}", e);
                                        }
                                    }
                                } else {
                                    log_error!("❌ Data type mismatch: {:?} != {:?}", source_data_type, target_data_type);
                                }
                            }
                        }
                    }
                } else {
                    log_error!("❌ Cannot connect node to itself");
                }
            }
            
            self.creating_connection = false;
            self.connection_source_node = None;
            self.connection_source_port = None;
            log_debug!("Connection state reset");
        }
        
        for (i, port) in node.output_ports.iter().enumerate() {
            let port_pos = egui::pos2(
                node_screen_pos.x + node.size.x + 5.0,
                node_screen_pos.y + 35.0 + (i as f32 * 25.0)
            );
            
            painter.circle_filled(
                port_pos,
                6.0,
                port.data_type.get_color()
            );
            
            painter.circle_stroke(
                port_pos,
                6.0,
                egui::Stroke::new(1.0, egui::Color32::WHITE)
            );
            
            painter.text(
                egui::pos2(port_pos.x - 15.0, port_pos.y),
                egui::Align2::RIGHT_CENTER,
                &port.name,
                egui::FontId::monospace(10.0),
                egui::Color32::WHITE
            );
            
            let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(20.0, 20.0));
            let response = ui.interact(port_rect, egui::Id::new(format!("output_port_{}_{}", node.id, i)), egui::Sense::click());
            
            if response.clicked() && !self.creating_connection {
                self.creating_connection = true;
                self.connection_source_node = Some(node.id.clone());
                self.connection_source_port = Some(i);
                self.connection_target_pos = port_pos;
                log_debug!("Started connection from node {} port {}", node.id, i);
            } else if response.clicked() && self.creating_connection {
                log_warn!("Cannot start new connection while one is in progress");
            }
        }
    }
    
    fn draw_single_node(&mut self, ui: &mut egui::Ui, node: &automation_flow::AutomationNode) {
        let painter = ui.painter();
        let node_rect = egui::Rect::from_min_size(node.position, node.size);
        
        let is_selected = self.selected_node.as_ref() == Some(&node.id);
        let bg_color = if is_selected {
            node.node_type.get_color().gamma_multiply(1.3)
        } else {
            node.node_type.get_color()
        };
        
        painter.rect_filled(
            node_rect,
            egui::Rounding::same(8.0),
            bg_color
        );
        
        painter.rect_stroke(
            node_rect,
            egui::Rounding::same(8.0),
            egui::Stroke::new(2.0, egui::Color32::from_gray(200))
        );
        
        painter.text(
            egui::pos2(node_rect.center().x, node_rect.min.y + 15.0),
            egui::Align2::CENTER_TOP,
            node.node_type.display_name(),
            egui::FontId::default(),
            egui::Color32::WHITE
        );
        
        let node_response = ui.interact(node_rect, egui::Id::new(format!("node_{}", node.id)), egui::Sense::click_and_drag());
        if node_response.clicked() {
            self.selected_node = Some(node.id.clone());
            log_debug!("Node {} selected", node.id);
        }
        
        if node_response.drag_started() {
            self.save_workflow_state();
        }
        
        if node_response.dragged() {
            if let Some(ref mut workflow) = self.current_workflow {
                if let Some(ref mut node_mut) = workflow.nodes.get_mut(&node.id) {
                    node_mut.position += node_response.drag_delta();
                }
            }
        }
        
        let mut port_interactions = Vec::new();
        
        for (i, port) in node.input_ports.iter().enumerate() {
            if let Some(port_pos) = node.get_input_port_position(i) {
                let is_potential_target = self.creating_connection && 
                    self.connection_source_node.as_ref() != Some(&node.id);
                
                let port_color = if is_potential_target {
                    port.data_type.get_color().gamma_multiply(1.5)
                } else {
                    port.data_type.get_color()
                };
                
                painter.circle_filled(
                    port_pos,
                    6.0,
                    port_color
                );
                
                painter.circle_stroke(
                    port_pos,
                    6.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE)
                );
                
                painter.text(
                    egui::pos2(port_pos.x + 15.0, port_pos.y),
                    egui::Align2::LEFT_CENTER,
                    &port.name,
                    egui::FontId::monospace(10.0),
                    egui::Color32::WHITE
                );
                
                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(20.0, 20.0));
                let response = ui.interact(port_rect, egui::Id::new(format!("input_port_{}_{}", node.id, i)), egui::Sense::click());
                
                if response.clicked() && self.creating_connection {
                    port_interactions.push((node.id.clone(), i, port.data_type.clone()));
                    log_debug!("Clicked input port {} on node {}", i, node.id);
                }
            }
        }
        
        for (target_node_id, target_port_idx, target_data_type) in port_interactions {
            if let (Some(ref source_node_id), Some(source_port_idx)) = 
                (&self.connection_source_node, self.connection_source_port) {
                
                log_debug!("Attempting to connect {} port {} -> {} port {}", 
                    source_node_id, source_port_idx, target_node_id, target_port_idx);
                
                if source_node_id != &target_node_id {
                    if let Some(ref mut workflow) = self.current_workflow {
                        if let Some(source_node) = workflow.nodes.get(source_node_id) {
                            if source_port_idx < source_node.output_ports.len() {
                                let source_data_type = &source_node.output_ports[source_port_idx].data_type;
                                
                                log_debug!("Data types: {:?} -> {:?}", source_data_type, target_data_type);
                                
                                if source_data_type == &target_data_type {
                                    let connection = automation_flow::NodeConnection {
                                        id: format!("conn_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                                        from_node: source_node_id.clone(),
                                        from_port: source_port_idx,
                                        to_node: target_node_id,
                                        to_port: target_port_idx,
                                        data_type: source_data_type.clone(),
                                    };
                                    
                                    match workflow.add_connection(connection) {
                                        Ok(_) => {
                                            log_debug!("Connection created successfully");
                                            
                                            // Trigger auto-fill for all output nodes when connections are made
                                            let workflow_clone = workflow.clone();
                                            for (_, output_node) in workflow.nodes.iter_mut() {
                                                if output_node.node_type == automation_flow::NodeType::OutputFile {
                                                    output_node.auto_fill_output_from_input(&workflow_clone);
                                                }
                                            }
                                            
                                            self.save_workflow_state();
                                        },
                                        Err(e) => {
                                            log_error!("❌ Connection failed: {}", e);
                                        }
                                    }
                                } else {
                                    log_error!("❌ Data type mismatch: {:?} != {:?}", source_data_type, target_data_type);
                                }
                            }
                        }
                    }
                } else {
                    log_error!("❌ Cannot connect node to itself");
                }
            }
            
            self.creating_connection = false;
            self.connection_source_node = None;
            self.connection_source_port = None;
            log_debug!("Connection state reset");
        }
        

        for (i, port) in node.output_ports.iter().enumerate() {
            if let Some(port_pos) = node.get_output_port_position(i) {
                painter.circle_filled(
                    port_pos,
                    6.0,
                    port.data_type.get_color()
                );
                
                painter.circle_stroke(
                    port_pos,
                    6.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE)
                );
                
                painter.text(
                    egui::pos2(port_pos.x - 15.0, port_pos.y),
                    egui::Align2::RIGHT_CENTER,
                    &port.name,
                    egui::FontId::monospace(10.0),
                    egui::Color32::WHITE
                );
                
                let port_rect = egui::Rect::from_center_size(port_pos, egui::vec2(20.0, 20.0));
                let response = ui.interact(port_rect, egui::Id::new(format!("output_port_{}_{}", node.id, i)), egui::Sense::click());
                
                if response.clicked() && !self.creating_connection {
                    self.creating_connection = true;
                    self.connection_source_node = Some(node.id.clone());
                    self.connection_source_port = Some(i);
                    self.connection_target_pos = port_pos;
                    log_debug!("Started connection from node {} port {}", node.id, i);
                } else if response.clicked() && self.creating_connection {
                    log_warn!("Cannot start new connection while one is in progress");
                }
            }
        }
        
    }
    
    fn handle_canvas_interaction(&mut self, ui: &mut egui::Ui, _canvas_rect: egui::Rect, response: &egui::Response) {
        if self.creating_connection {
            if let Some(pointer_pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
                self.connection_target_pos = pointer_pos;
            }
        }
        
        if response.clicked() {
            if self.creating_connection {
                self.creating_connection = false;
                self.connection_source_node = None;
                self.connection_source_port = None;
                log_debug!("Connection cancelled by clicking empty area");
            } else {
                self.selected_node = None;
            }
        }
        
        if response.secondary_clicked() {
            if self.creating_connection {
                self.creating_connection = false;
                self.connection_source_node = None;
                self.connection_source_port = None;
                log_debug!("Connection cancelled by right click");
            }
        }
        
        ui.ctx().input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                if self.creating_connection {
                    self.creating_connection = false;
                    self.connection_source_node = None;
                    self.connection_source_port = None;
                    log_debug!("Connection cancelled by ESC key");
                }
            }
            
            if i.key_pressed(egui::Key::Delete) {
                self.delete_selected_node();
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                self.undo_workflow();
            }
            
            if (i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) || 
               (i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
                self.redo_workflow();
            }
            
            if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
                self.select_all_nodes();
            }
        });
    }
    
    fn is_point_near_connection_line(&self, line_start: egui::Pos2, line_end: egui::Pos2, point: egui::Pos2) -> bool {
        let line_length_sq = (line_end.x - line_start.x).powi(2) + (line_end.y - line_start.y).powi(2);
        
        if line_length_sq == 0.0 {
            let dist_sq = (point.x - line_start.x).powi(2) + (point.y - line_start.y).powi(2);
            return dist_sq <= 100.0; 
        }
        
        let t = ((point.x - line_start.x) * (line_end.x - line_start.x) + 
                (point.y - line_start.y) * (line_end.y - line_start.y)) / line_length_sq;
        
        let t = t.max(0.0).min(1.0);
        
        let closest_point = egui::pos2(
            line_start.x + t * (line_end.x - line_start.x),
            line_start.y + t * (line_end.y - line_start.y)
        );
        
        let dist_sq = (point.x - closest_point.x).powi(2) + (point.y - closest_point.y).powi(2);
        dist_sq <= 100.0 
    }
    
    fn save_workflow(&self) {
        if let Some(ref workflow) = self.current_workflow {
            match rfd::FileDialog::new()
                .add_filter("Workflow Files", &["json"])
                .set_file_name(&workflow.name)
                .save_file()
            {
                Some(path) => {
                    match serde_json::to_string_pretty(workflow) {
                        Ok(json) => {
                            if let Err(e) = std::fs::write(&path, json) {
                                log_error!("Failed to save workflow: {}", e);
                            } else {
                                log_info!("Workflow saved successfully");
                            }
                        }
                        Err(e) => {
                            log_error!("Failed to serialize workflow: {}", e);
                        }
                    }
                }
                None => {}
            }
        }
    }
    
    fn load_workflow(&mut self) {
        match rfd::FileDialog::new()
            .add_filter("Workflow Files", &["json"])
            .pick_file()
        {
            Some(path) => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<automation_flow::AutomationWorkflow>(&content) {
                            Ok(workflow) => {
                                self.current_workflow = Some(workflow);
                                log_info!("Workflow loaded successfully");
                            }
                            Err(e) => {
                                log_error!("Failed to parse workflow: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log_error!("Failed to read workflow file: {}", e);
                    }
                }
            }
            None => {}
        }
    }
    
    fn execute_workflow(&mut self) {
        log_debug!("Execute button clicked");
        
        // Clone workflow early to avoid borrowing issues
        let workflow_clone = self.current_workflow.clone();
        
        if let Some(workflow) = workflow_clone {
            log_info!("Workflow exists with {} nodes", workflow.nodes.len());
            
            if workflow.nodes.is_empty() {
                self.status_message = "No nodes in workflow".to_string();
                log_warn!("No nodes in workflow");
                return;
            }
            
            let has_input = workflow.nodes.values().any(|n| n.node_type == automation_flow::NodeType::InputFile);
            let has_output = workflow.nodes.values().any(|n| n.node_type == automation_flow::NodeType::OutputFile);
            
            if !has_input {
                self.status_message = "No input file node found".to_string();
                log_error!("No input file node found");
                return;
            }
            
            if !has_output {
                self.status_message = "No output file node found".to_string();
                log_error!("No output file node found");
                return;
            }
            
            // Ensure workflow executor has latest hardware encoders
            let hardware_encoders = self.get_cached_hardware_encoders();
            if !hardware_encoders.is_empty() {
                self.workflow_executor.update_hardware_cache(hardware_encoders);
                log_info!("🚀 Workflow execution with hardware acceleration enabled");
            } else {
                log_info!("⚙ Workflow execution with software-only encoding");
            }
            
            log_info!("Starting workflow execution...");
            match self.workflow_executor.execute_workflow(workflow) {
                Ok(_) => {
                    self.status_message = "Workflow execution started successfully".to_string();
                    log_debug!("Workflow execution started successfully");
                }
                Err(e) => {
                    self.status_message = format!("Workflow execution failed: {}", e);
                    log_error!("❌ Workflow execution failed: {}", e);
                }
            }
        } else {
            self.status_message = "No workflow loaded".to_string();
            log_warn!("No workflow loaded");
        }
    }
    
    fn save_workflow_state(&mut self) {
        if let Some(ref workflow) = self.current_workflow {
            if self.history_index >= 0 && (self.history_index as usize) < self.workflow_history.len() - 1 {
                self.workflow_history.truncate((self.history_index + 1) as usize);
            }
            
            self.workflow_history.push(workflow.clone());
            self.history_index = self.workflow_history.len() as isize - 1;
            
            if self.workflow_history.len() > self.max_history_size {
                self.workflow_history.remove(0);
                self.history_index -= 1;
            }
            
            log_debug!("Saved workflow state. History size: {}, Index: {}", 
                self.workflow_history.len(), self.history_index);
        }
    }
    
    fn undo_workflow(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(workflow) = self.workflow_history.get(self.history_index as usize) {
                self.current_workflow = Some(workflow.clone());
                log_debug!("Undo: History index now {}", self.history_index);
            }
        } else {
            log_debug!("No more undo steps available");
        }
    }
    
    fn redo_workflow(&mut self) {
        if self.history_index < (self.workflow_history.len() as isize - 1) {
            self.history_index += 1;
            if let Some(workflow) = self.workflow_history.get(self.history_index as usize) {
                self.current_workflow = Some(workflow.clone());
                log_debug!("Redo: History index now {}", self.history_index);
            }
        } else {
            log_debug!("No more redo steps available");
        }
    }
    
    fn delete_selected_node(&mut self) {
        if let Some(ref selected_id) = self.selected_node.clone() {
            self.save_workflow_state();
            
            if let Some(ref mut workflow) = self.current_workflow {
                workflow.remove_node(selected_id);
                self.selected_node = None;
                log_debug!("Deleted node: {}", selected_id);
            }
        }
    }
    
    fn copy_selected_node(&mut self) -> Option<automation_flow::AutomationNode> {
        if let Some(ref selected_id) = self.selected_node {
            if let Some(ref workflow) = self.current_workflow {
                return workflow.nodes.get(selected_id).cloned();
            }
        }
        None
    }
    
    fn paste_node(&mut self, node: &automation_flow::AutomationNode, offset: egui::Vec2) {
        // Save state first
        self.save_workflow_state();
        
        if let Some(ref mut workflow) = self.current_workflow {
            let mut new_node = node.clone();
            new_node.id = format!("node_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
            let new_position = new_node.position + offset;
            new_node.position = new_position;
            
            log_debug!("Pasted node at position {:?}", new_position);
            workflow.add_node(new_node);
        }
    }
    
    fn select_all_nodes(&mut self) {
        if let Some(ref workflow) = self.current_workflow {
            if let Some(first_node_id) = workflow.nodes.keys().next().cloned() {
                self.selected_node = Some(first_node_id);
            }
        }
    }
    
    fn handle_drag_and_drop(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        let file_path = path.to_string_lossy().to_string();
                        log_debug!("File dropped: {}", file_path);
                        
                        if self.is_media_file(&file_path) {
                            if self.show_automation_editor {
                                self.assign_file_to_automation_node(&file_path);
                            } else {
                                // Check if current operation supports multiple files
                                let supports_multiple = matches!(self.current_operation, 
                                    Some(OperationType::AudioMerge) | 
                                    Some(OperationType::BatchConvert) | 
                                    Some(OperationType::VideoAudioMerge)
                                );
                                
                                if supports_multiple {
                                    // For multi-file operations, let FileSelector handle the drag-drop
                                    // to avoid duplicate processing
                                    log_debug!("Multi-file operation detected, FileSelector will handle this file");
                                } else {
                                    // For single-file operations, handle directly
                                    if !self.input_files.is_empty() {
                                        self.input_files[0] = file_path.clone();
                                    } else {
                                        self.input_files.push(file_path.clone());
                                    }
                                    log_debug!("Set file for single-file operation: {}", file_path);
                                }
                            }
                        } else {
                            self.status_message = format!("Unsupported file type: {}", file_path);
                            log_warn!("⚠ Unsupported file type: {}", file_path);
                        }
                    }
                }
            }
        });
    }
    
    fn is_media_file(&self, file_path: &str) -> bool {
        let path = std::path::Path::new(file_path);
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), 
                "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "3gp" | "ogv" |
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "opus" | "m4a" |
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" |
                "srt" | "ass" | "ssa" | "vtt" | "sub" // Subtitle files
            )
        } else {
            false
        }
    }
    
    fn assign_file_to_automation_node(&mut self, file_path: &str) {
        if let Some(ref mut workflow) = self.current_workflow {
            // Find both input and output file nodes
            let mut input_nodes: Vec<String> = workflow.nodes.iter()
                .filter(|(_, node)| node.node_type == automation_flow::NodeType::InputFile)
                .map(|(id, _)| id.clone())
                .collect();
            
            let mut output_nodes: Vec<String> = workflow.nodes.iter()
                .filter(|(_, node)| node.node_type == automation_flow::NodeType::OutputFile)
                .map(|(id, _)| id.clone())
                .collect();
            
            // Only show warning if workflow has other nodes but no file nodes
            // (Empty workflow should not trigger this warning)
            if input_nodes.is_empty() && output_nodes.is_empty() && !workflow.nodes.is_empty() {
                self.status_message = "No Input or Output File nodes found. Add a file node first.".to_string();
                log_debug!("Cannot assign file: no file nodes in workflow with {} nodes", workflow.nodes.len());
                return;
            }
            
            // If workflow is empty, silently ignore (user hasn't started building workflow yet)
            if workflow.nodes.is_empty() {
                return;
            }
            
            // Determine target node - prioritize selected node if it's a file node
            let (target_node_id, is_output_node) = if let Some(ref selected_id) = self.selected_node {
                if input_nodes.contains(selected_id) {
                    (selected_id.clone(), false)
                } else if output_nodes.contains(selected_id) {
                    (selected_id.clone(), true)
                } else if !input_nodes.is_empty() {
                    (input_nodes.remove(0), false)
                } else {
                    (output_nodes.remove(0), true)
                }
            } else if !input_nodes.is_empty() {
                (input_nodes.remove(0), false)
            } else {
                (output_nodes.remove(0), true)
            };
            
            if let Some(node) = workflow.nodes.get_mut(&target_node_id) {
                let param_name = if is_output_node { "output_path" } else { "file_path" };
                let node_type_name = if is_output_node { "Output" } else { "Input" };
                
                if let Some(param) = node.parameters.get_mut(param_name) {
                    param.value = file_path.to_string();
                    self.status_message = format!("File assigned to {} node: {}", 
                        node_type_name,
                        std::path::Path::new(file_path).file_name()
                            .unwrap_or_default().to_string_lossy());
                    log_debug!("Assigned file {} to {} node {}", file_path, node_type_name, target_node_id);
                }
            }
        } else {
            self.status_message = "No workflow loaded. Switch to automation editor first.".to_string();
        }
    }
}

fn cubic_bezier(p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2, t: f32) -> egui::Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    egui::pos2(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

fn main() -> Result<()> {
    std::env::set_var("AV_LOG_FORCE_LEVEL", "quiet");
    std::env::set_var("AV_LOG_FORCE_NOCOLOR", "1");
    std::env::set_var("FFMPEG_HIDE_BANNER", "1");
    std::env::set_var("AV_LOG_SKIP_REPEATED", "1");
    
    // Initialize bundled FFmpeg (will be initialized on first use)
    let _ = bundled_ffmpeg::get_bundled_ffmpeg()?;
    
    // Load the  
    let icon_data = load_custom_icon();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("FF GUI - Audio/Video Processing Tool")
            .with_icon(icon_data),
        ..Default::default()
    };
    
    eframe::run_native(
        "FF GUI",
        options,
        Box::new(|cc| {
            // Enable Chinese font support
            setup_fallback_fonts(&cc.egui_ctx); 
            Ok(Box::new(FFmpegGui::default()))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}

fn setup_fallback_fonts(ctx: &egui::Context) {
    log_debug!("Setting up font support...");
    
    let mut fonts = egui::FontDefinitions::default();
    
    // Load Chinese font support - use system-appropriate paths
    #[cfg(target_os = "windows")]
    let chinese_fonts = [
        ("chinese", "C:/Windows/Fonts/msyh.ttc"),       // Microsoft YaHei
        ("chinese2", "C:/Windows/Fonts/simsun.ttc"),    // SimSun fallback
    ];
    
    #[cfg(not(target_os = "windows"))]
    let chinese_fonts: [(& str, & str); 0] = [];  // No default fonts on non-Windows
    
    // Load symbol font support - use system-appropriate paths
    #[cfg(target_os = "windows")]
    let symbol_fonts = [
        ("emoji", "C:/Windows/Fonts/seguiemj.ttf"),     // Segoe UI Emoji
        ("symbol", "C:/Windows/Fonts/seguisym.ttf"),    // Segoe UI Symbol
    ];
    
    #[cfg(not(target_os = "windows"))]
    let symbol_fonts: [(& str, & str); 0] = [];  // No default fonts on non-Windows
    
    let mut fonts_loaded = false;
    
    // Load Chinese fonts
    for (name, font_path) in chinese_fonts.iter() {
        if let Ok(font_bytes) = std::fs::read(font_path) {
            log_debug!("Loading Chinese font: {}", font_path);
            
            fonts.font_data.insert(
                name.to_string(),
                egui::FontData::from_owned(font_bytes),
            );

            // Add Chinese font to the end of font family as fallback
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(name.to_string());
                
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(name.to_string());

            fonts_loaded = true;
            break; // Only need to load one successful Chinese font
        }
    }
    
    // Load symbol fonts
    for (name, font_path) in symbol_fonts.iter() {
        if let Ok(font_bytes) = std::fs::read(font_path) {
            log_debug!("Loading symbol font: {}", font_path);
            
            fonts.font_data.insert(
                name.to_string(),
                egui::FontData::from_owned(font_bytes),
            );

            // Add symbol font to the end of font family
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(name.to_string());

            fonts_loaded = true;
        }
    }

    if fonts_loaded {
        ctx.set_fonts(fonts);
        log_debug!("Font support enabled");
    } else {
        log_debug!("Failed to load fonts, using system defaults");
    }
}


impl Drop for FFmpegGui {
    fn drop(&mut self) {
        if let Some(executor) = &self.task_executor {
            executor.terminate_all_ffmpeg_processes();
        }
    }
}

fn load_custom_icon() -> egui::IconData {
    // Try to load the ICO version
    if let Some(icon_data) = load_image_from_bytes(include_bytes!("../icon.ico")) {
        return icon_data;
    }
    
    // Fallback to a programmatic icon if ICO loading fails
    create_fallback_icon()
}

fn load_image_from_bytes(bytes: &[u8]) -> Option<egui::IconData> {
    use image::ImageFormat;
    use std::io::Cursor;
    
    // Try to load the image using the image crate
    let cursor = Cursor::new(bytes);
    
    // Try ICO first (most common for Windows icons)
    if let Ok(img) = image::load(cursor, ImageFormat::Ico) {
        return convert_image_to_icon_data(img);
    }
    
    // Try other formats as fallback
    let cursor = Cursor::new(bytes);
    if let Ok(img) = image::io::Reader::new(cursor).with_guessed_format() {
        if let Ok(img) = img.decode() {
            return convert_image_to_icon_data(img);
        }
    }
    
    None
}

fn convert_image_to_icon_data(img: image::DynamicImage) -> Option<egui::IconData> {
    // Convert to RGBA8 and resize to appropriate size for window icon
    let img = img.resize(64, 64, image::imageops::FilterType::Lanczos3);
    let rgba_img = img.to_rgba8();
    
    let (width, height) = rgba_img.dimensions();
    let pixels = rgba_img.into_raw();
    
    Some(egui::IconData {
        rgba: pixels,
        width: width,
        height: height,
    })
}

fn create_yellow_diamond(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let size = (width.min(height) as f32 / 2.0 * 0.8) as f32;
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            
            // Create diamond shape
            let dist = (dx.abs() + dy.abs()) / size;
            
            let pixel_offset = ((y * width + x) * 4) as usize;
            
            if dist <= 1.0 {
                // Main yellow color (matching your icon)
                let alpha = (1.0 - dist * 0.1).max(0.8);
                pixels[pixel_offset] = 255;     // R
                pixels[pixel_offset + 1] = 215; // G  
                pixels[pixel_offset + 2] = 0;   // B
                pixels[pixel_offset + 3] = (255.0 * alpha) as u8; // A
                
                // Add dark border
                if dist > 0.85 {
                    pixels[pixel_offset] = 139;     // Dark yellow border
                    pixels[pixel_offset + 1] = 69;
                    pixels[pixel_offset + 2] = 19;
                }
            } else {
                // Transparent
                pixels[pixel_offset] = 0;
                pixels[pixel_offset + 1] = 0;
                pixels[pixel_offset + 2] = 0;
                pixels[pixel_offset + 3] = 0;
            }
        }
    }
    
    pixels
}

fn create_fallback_icon() -> egui::IconData {
    // Create a 32x32 fallback icon
    let pixels = create_yellow_diamond(32, 32);
    egui::IconData {
        rgba: pixels,
        width: 32,
        height: 32,
    }
}