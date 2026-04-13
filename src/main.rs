#![windows_subsystem = "windows"]

mod hotkey;
mod indexer;
mod search;
mod ui;

use eframe::egui;
use std::thread;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crossbeam_channel::unbounded;
use ui::LauncherApp;
use tray_icon::{TrayIconBuilder, Icon, menu::{Menu, MenuItem, MenuEvent}};

fn create_tray_icon() -> tray_icon::TrayIcon {
    let (width, height) = (32, 32);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        rgba.extend_from_slice(&[80, 100, 200, 255]); 
    }
    let icon = Icon::from_rgba(rgba, width, height).unwrap();

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Sair do Launcher", true, None);
    tray_menu.append(&quit_i).unwrap();

    let quit_id = quit_i.id().clone();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Native Launcher (Alt+W)")
        .with_icon(icon)
        .build()
        .unwrap();

    thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        while let Ok(event) = menu_channel.recv() {
            if event.id == quit_id {
                std::process::exit(0);
            }
        }
    });

    tray_icon
}

fn main() -> Result<(), eframe::Error> {
    let (tx, rx) = unbounded();

    thread::spawn(move || {
        hotkey::listen_for_hotkey(tx);
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_position([-10000.0, -10000.0])
            .with_decorations(false) 
            .with_always_on_top()    
            .with_transparent(true)
            .with_taskbar(false)
            .with_visible(true),
        ..Default::default()
    };

    let is_visible = Arc::new(AtomicBool::new(false));
    let app_visibility = is_visible.clone();

    eframe::run_native(
        "Native Launcher",
        options,
        Box::new(move |cc| {
            Box::leak(Box::new(create_tray_icon()));

            let ctx = cc.egui_ctx.clone();
            let thread_visibility = is_visible.clone();
            
            thread::spawn(move || {
                while rx.recv().is_ok() {
                    thread_visibility.store(true, Ordering::SeqCst);
                    ctx.request_repaint(); // A interface acorda garantidamente!
                }
            });

            Box::new(LauncherApp::new(app_visibility))
        }),
    )
}