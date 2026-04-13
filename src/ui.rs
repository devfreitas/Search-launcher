use eframe::egui;
use crate::indexer::{build_index, AppEntry};
use crate::search::search_apps;
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
        }
    }

    fn execute_selected(&mut self, ctx: &egui::Context) {
        if let Some(app) = self.filtered.get(self.selected_index) {
            let path = HSTRING::from(app.path.to_str().unwrap_or(""));
            unsafe {
                ShellExecuteW(None, w!("open"), &path, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
            }
            self.hide(ctx);
        }
    }

    pub fn hide(&mut self, ctx: &egui::Context) {
        self.is_visible.store(false, Ordering::SeqCst);
        self.search_query.clear();
        self.selected_index = 0;
        self.filtered.clear();
        
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
            if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                let center_pos = egui::pos2(
                    (monitor_size.x - 600.0) / 2.0,
                    (monitor_size.y - 400.0) / 2.0,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(center_pos));
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        if !current_visibility {
            return;
        }

        let frame_style = egui::Frame {
            fill: egui::Color32::from_rgba_premultiplied(20, 20, 22, 250),
            rounding: egui::Rounding::same(16.0),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(60, 60, 65, 255)),
            inner_margin: egui::Margin::same(20.0),
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame_style).show(ctx, |ui| {
            ui.style_mut().visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(80, 100, 200);

            let response = ui.add(egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("🔍 Digite para pesquisar...")
                .font(egui::FontId::proportional(28.0))
                .frame(false)
                .desired_width(f32::INFINITY));

            response.request_focus();

            if response.changed() {
                if self.search_query.trim().is_empty() {
                    self.filtered.clear();
                } else {
                    self.filtered = search_apps(&self.search_query, &self.index);
                }
                self.selected_index = 0;
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.selected_index = (self.selected_index + 1).min(self.filtered.len().saturating_sub(1));
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.filtered.is_empty() {
                self.execute_selected(ctx);
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.hide(ctx);
            }

            if !self.filtered.is_empty() {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
            }

            for (i, app) in self.filtered.iter().enumerate() {
                let is_selected = i == self.selected_index;

                let item_frame = egui::Frame::none()
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .fill(if is_selected {
                        egui::Color32::from_rgba_premultiplied(60, 80, 180, 200)
                    } else {
                        egui::Color32::TRANSPARENT
                    });

                item_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let text_color = if is_selected { egui::Color32::WHITE } else { egui::Color32::LIGHT_GRAY };
                    ui.label(egui::RichText::new(&app.name).color(text_color).size(16.0));
                });
            }
        });
    }
}



