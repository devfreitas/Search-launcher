use eframe::egui;
use crate::indexer::{build_index, AppEntry};
use crate::search::search_apps;
use crate::icon_loader::IconManager;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct LauncherApp {
    search_query: String,
    index: Vec<AppEntry>,
    filtered: Vec<AppEntry>,
    selected_index: usize,
    is_visible: Arc<AtomicBool>,
    was_visible_last_frame: bool,
    current_height: f32,
    icon_manager: IconManager, 
}

impl LauncherApp {
    pub fn new(is_visible: Arc<AtomicBool>) -> Self {
        let index = build_index();
        Self {
            search_query: String::new(),
            index,
            filtered: Vec::new(),
            selected_index: 0,
            is_visible,
            was_visible_last_frame: false,
            current_height: 70.0,
            icon_manager: IconManager::new(45),
        }
    }

    fn execute_selected(&mut self, ctx: &egui::Context) {
        let query = self.search_query.trim();

        if query.starts_with("g ") && query.len() > 2 {
            let search_term = &query[2..];
            let url = format!("https://www.google.com/search?q={}", search_term.replace(' ', "+"));
            let path = HSTRING::from(url);
            unsafe {
                ShellExecuteW(None, w!("open"), &path, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
            }
            self.hide(ctx);
            return;
        }

        if let Some(app) = self.filtered.get(self.selected_index) {
            let path_str = app.path.to_str().unwrap_or("");

            if path_str.starts_with("UWP:") {
                let app_id = &path_str[4..];
                let shell_args = format!("shell:appsFolder\\{}", app_id);
                let explorer = HSTRING::from("explorer.exe");
                let args = HSTRING::from(shell_args);
                unsafe {
                    ShellExecuteW(None, w!("open"), &explorer, &args, PCWSTR::null(), SW_SHOWNORMAL);
                }
            } else {
                let path = HSTRING::from(path_str);
                unsafe {
                    ShellExecuteW(None, w!("open"), &path, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
                }
            }
            self.hide(ctx);
        }
    }

    pub fn hide(&mut self, ctx: &egui::Context) {
        self.is_visible.store(false, Ordering::SeqCst);
        self.search_query.clear();
        self.selected_index = 0;
        self.filtered.clear();
        self.current_height = 70.0;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(600.0, self.current_height)));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(-10000.0, -10000.0)));
        ctx.request_repaint(); 
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_visibility = self.is_visible.load(Ordering::SeqCst);
        let just_opened = current_visibility && !self.was_visible_last_frame;
        self.was_visible_last_frame = current_visibility;

        if just_opened {
            self.current_height = 70.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(600.0, self.current_height)));
            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                let center_pos = egui::pos2((monitor_size.x - 600.0) / 2.0, monitor_size.y * 0.25);
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        if !current_visibility { return; }

        let mut target_height = 70.0; 
        if !self.filtered.is_empty() {
            target_height += 12.0; 
            let items_to_show = self.filtered.len().min(8) as f32;
            target_height += items_to_show * 44.0; 
            target_height += 10.0; 
        }

        if (self.current_height - target_height).abs() > 0.5 {
            self.current_height = target_height;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(600.0, self.current_height)));
        }

        // Estilo moderno "Acrylic-like"
        let frame_style = egui::Frame {
            fill: egui::Color32::from_rgba_premultiplied(28, 28, 32, 252),
            rounding: egui::Rounding::same(12.0), 
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(70, 70, 80, 255)),
            inner_margin: egui::Margin::symmetric(20.0, 15.0),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.style_mut().visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
            
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                ui.label(egui::RichText::new("🔍").size(20.0));
                ui.add_space(8.0);
                
                let response = ui.add(egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("O que vamos abrir hoje?")
                    .font(egui::FontId::proportional(22.0))
                    .frame(false)
                    .desired_width(f32::INFINITY));

                response.request_focus();

                if response.changed() {
                    if self.search_query.trim().is_empty() {
                        self.filtered.clear();
                    } else if !self.search_query.trim().starts_with("g ") {
                        self.filtered = search_apps(&self.search_query, &self.index);
                    }
                    self.selected_index = 0;
                }
            });

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.selected_index = (self.selected_index + 1).min(self.filtered.len().saturating_sub(1));
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.execute_selected(ctx);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.hide(ctx);
            }

            if !self.filtered.is_empty() && !self.search_query.trim().starts_with("g ") {
                ui.add_space(10.0);
                ui.painter().hline(ui.cursor().left()..=ui.cursor().right(), ui.cursor().top(), egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(60, 60, 70, 100)));
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for (i, app) in self.filtered.iter().enumerate() {
                        let is_selected = i == self.selected_index;
                        
                        let item_frame = egui::Frame::none()
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .fill(if is_selected {
                                egui::Color32::from_rgba_premultiplied(60, 60, 80, 200)
                            } else {
                                egui::Color32::TRANSPARENT
                            });

                        item_frame.show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let path_str = app.path.to_str().unwrap_or("");
                                
                                if path_str.starts_with("UWP:") {
                                    ui.label(egui::RichText::new("📱").size(18.0));
                                } else if path_str.contains("📁") || app.name.contains("📁") {
                                    ui.label(egui::RichText::new("📁").size(18.0));
                                } else if let Some(icon) = self.icon_manager.get_icon(ctx, &app.path) {
                                    ui.add(egui::Image::new(&icon).fit_to_exact_size(egui::vec2(22.0, 22.0)));
                                } else {
                                    ui.label(egui::RichText::new("🚀").size(18.0));
                                }

                                ui.add_space(12.0);
                                
                                let text_color = if is_selected { 
                                    egui::Color32::WHITE 
                                } else { 
                                    egui::Color32::from_gray(200) 
                                };
                                
                                ui.label(egui::RichText::new(&app.name.replace("📁 ", ""))
                                    .color(text_color)
                                    .size(16.0)
                                    .strong());
                                    
                                if is_selected {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new("Enter para abrir").size(10.0).color(egui::Color32::from_gray(120)));
                                    });
                                }
                            });
                        });
                    }
                });
            } else if self.search_query.trim().starts_with("g ") {
                ui.add_space(15.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(40, 40, 60, 150))
                    .rounding(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🌐").size(18.0));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new(format!("Pesquisar no Google: {}", &self.search_query[2..]))
                                .color(egui::Color32::LIGHT_BLUE)
                                .size(14.0));
                        });
                    });
            }
        });
    }
}