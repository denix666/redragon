use eframe::egui;
mod functions;
mod ui;
use ui::KeyboardApp;
mod rgb;
mod config;
use std::sync::Arc;
use eframe::egui::mutex::Mutex;
use crate::config::Config;

const APP_NAME: &str = "K686BG-RGB-M programmer";
const APP_PATH: &str = "redragon-programmer";
const MAIN_CONFIG_FILE_NAME: &str = "K686BG-RGB-M.toml";
const PROFILE_FILE_NAME: &str = "K686BG-RGB-M.json";
const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
const DEVICE_VID: u16 = 0x2E3C;
const DEVICE_PID: u16 = 0xC365;
const VENDOR_INTERFACE: i32 = 2;
const REPORT_ID: u8 = 1;
const REPORT_LEN: usize = 64;
const RGB_TABLE_LEN: usize = 396;
const RGB_PAGE_LEN: usize = 54;

const QUICK_COLORS: [egui::Color32; 16] = [
                    egui::Color32::from_rgb(0, 0, 0),
                    egui::Color32::from_rgb(128, 0, 0),
                    egui::Color32::from_rgb(0, 128, 0),
                    egui::Color32::from_rgb(128, 128, 0),
                    egui::Color32::from_rgb(0, 0, 128),
                    egui::Color32::from_rgb(128, 0, 128),
                    egui::Color32::from_rgb(0, 128, 128),
                    egui::Color32::from_rgb(192, 192, 192),
                    egui::Color32::from_rgb(128, 128, 128),
                    egui::Color32::from_rgb(255, 0, 0),
                    egui::Color32::from_rgb(0, 255, 0),
                    egui::Color32::from_rgb(255, 255, 0),
                    egui::Color32::from_rgb(0, 0, 255),
                    egui::Color32::from_rgb(255, 0, 255),
                    egui::Color32::from_rgb(0, 255, 255),
                    egui::Color32::from_rgb(255, 255, 255)
];

fn main() -> eframe::Result<()> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.iter().any(|argument| argument == "--probe-device") {
        match functions::load_keyboard_slots() {
            Ok(slots) => {
                for (slot, table) in slots.iter().enumerate() {
                    let nonzero = table.iter().filter(|byte| **byte != 0).count();
                    println!(
                        "RGB slot {slot}: {} bytes read, {nonzero} non-zero bytes",
                        table.len()
                    );
                }
            }
            Err(error) => {
                eprintln!("Device probe failed: {error}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    let config = Arc::new(Mutex::new(Config::load_or_create().expect("Failed to load config")));

    let (window_width, window_height, last_pos_x, last_pos_y) = {
        let cfg = config.lock();
        (cfg.global.last_width, cfg.global.last_height, cfg.global.last_pos_x, cfg.global.last_pos_y)
    };

    let title = format!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION"));
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window_width, window_height])
            .with_position([last_pos_x, last_pos_y])
            .with_maximized(true),
        ..Default::default()
    };

    if let Ok(icon) = functions::load_embedded_icon() {
        options.viewport = options.viewport.with_icon(icon);
    }

    eframe::run_native(&title, options,
        Box::new(|cc| {
            // Set dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());

            // Setup style
            let mut style: egui::Style = (*cc.egui_ctx.global_style()).clone();

            // Increase font size for different TextStyle
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::new(20.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
                (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Small, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
            ]
            .into();

            cc.egui_ctx.set_global_style(style);
            Ok(Box::new(KeyboardApp::new(cc, config)))
        }),
    )
}
