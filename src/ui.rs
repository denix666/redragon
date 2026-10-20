use std::{collections::{BTreeMap, BTreeSet}, fs, path::PathBuf, sync::Arc};
use eframe::egui::{self, mutex::Mutex};
use serde::{Deserialize, Serialize};
use crate::{functions::*, rgb::Rgb};

fn apply_profile_to_slot_1(profile: &Profile) -> Result<(), String> {
    let device = open_keyboard()?;
    let mut table = vec![0; crate::RGB_TABLE_LEN];
    for key in KEYBOARD_ROWS.iter().flat_map(|row| row.iter()) {
        if let Some(color) = profile.colors.get(key.name) {
            let offset = key.led * 3;
            if offset + 2 < table.len() {
                table[offset] = color.r;
                table[offset + 1] = color.g;
                table[offset + 2] = color.b;
            }
        }
    }

    write_rgb_slot(&device, 1, &table)?;
    enable_per_key_lighting(&device)?;
    if read_rgb_slot(&device, 1)? != table {
        return Err("Written profile verification mismatch".to_string());
    }
    Ok(())
}

fn apply_selected_keys_to_slot_1(profile: &Profile, key_names: &BTreeSet<String>) -> Result<(), String> {
    if key_names.is_empty() {
        return Err("Select at least one key first".to_string());
    }

    let device = open_keyboard()?;
    let mut table = read_rgb_slot(&device, 1)?;
    for key_name in key_names {
        let key = KEYBOARD_ROWS
            .iter()
            .flat_map(|row| row.iter())
            .find(|key| key.name == key_name)
            .ok_or_else(|| format!("No RGB index for key {key_name}"))?;
        let color = profile.colors.get(key_name).copied().unwrap_or_default();
        if key.led >= crate::RGB_TABLE_LEN / 3 {
            return Err(format!("LED index for {key_name} is not calibrated yet"));
        }
        let offset = key.led * 3;
        table[offset..offset + 3].copy_from_slice(&[color.r, color.g, color.b]);
    }

    write_rgb_slot(&device, 1, &table)?;
    enable_per_key_lighting(&device)?;
    if read_rgb_slot(&device, 1)? != table {
        return Err("Written keys verification mismatch".to_string());
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct KeySpec {
    name: &'static str,
    led: usize,
}

#[derive(Clone, Copy)]
struct KeyPlacement {
    name: &'static str,
    x: f32,
    y: usize,
    width: f32,
    height: f32,
}

macro_rules! key_at {
    ($name:literal, $x:expr, $y:expr) => {
        KeyPlacement {
            name: $name,
            x: $x,
            y: $y,
            width: 1.0,
            height: 1.0,
        }
    };
    ($name:literal, $x:expr, $y:expr, $width:expr) => {
        KeyPlacement {
            name: $name,
            x: $x,
            y: $y,
            width: $width,
            height: 1.0,
        }
    };
    ($name:literal, $x:expr, $y:expr, $width:expr, $height:expr) => {
        KeyPlacement {
            name: $name,
            x: $x,
            y: $y,
            width: $width,
            height: $height,
        }
    };
}

const KEYBOARD_ROWS: &[&[KeySpec]] = &[
    &[
        KeySpec {
            name: "Esc",
            led: 0,
        },
        KeySpec { name: "F1", led: 2 },
        KeySpec { name: "F2", led: 3 },
        KeySpec { name: "F3", led: 4 },
        KeySpec { name: "F4", led: 5 },
        KeySpec { name: "F5", led: 6 },
        KeySpec { name: "F6", led: 7 },
        KeySpec { name: "F7", led: 8 },
        KeySpec { name: "F8", led: 9 },
        KeySpec {
            name: "F9",
            led: 10,
        },
        KeySpec {
            name: "F10",
            led: 11,
        },
        KeySpec {
            name: "F11",
            led: 12,
        },
        KeySpec {
            name: "F12",
            led: 13,
        },
        KeySpec {
            name: "Del",
            led: 14,
        },
        KeySpec {
            name: "Ins",
            led: 15,
        },
        KeySpec {
            name: "PgUp",
            led: 16,
        },
        KeySpec {
            name: "PgDn",
            led: 17,
        },
    ],
    &[
        KeySpec { name: "`", led: 22 },
        KeySpec { name: "1", led: 23 },
        KeySpec { name: "2", led: 24 },
        KeySpec { name: "3", led: 25 },
        KeySpec { name: "4", led: 26 },
        KeySpec { name: "5", led: 27 },
        KeySpec { name: "6", led: 28 },
        KeySpec { name: "7", led: 29 },
        KeySpec { name: "8", led: 30 },
        KeySpec { name: "9", led: 31 },
        KeySpec { name: "0", led: 32 },
        KeySpec { name: "-", led: 33 },
        KeySpec { name: "=", led: 34 },
        KeySpec {
            name: "Backspace",
            led: 36,
        },
        KeySpec {
            name: "Num Lock",
            led: 37,
        },
        KeySpec {
            name: "Num /",
            led: 38,
        },
        KeySpec {
            name: "Num *",
            led: 39,
        },
        KeySpec {
            name: "Num -",
            led: 40,
        },
    ],
    &[
        KeySpec {
            name: "Tab",
            led: 44,
        },
        KeySpec { name: "Q", led: 45 },
        KeySpec { name: "W", led: 46 },
        KeySpec { name: "E", led: 47 },
        KeySpec { name: "R", led: 48 },
        KeySpec { name: "T", led: 49 },
        KeySpec { name: "Y", led: 50 },
        KeySpec { name: "U", led: 51 },
        KeySpec { name: "I", led: 52 },
        KeySpec { name: "O", led: 53 },
        KeySpec { name: "P", led: 54 },
        KeySpec { name: "[", led: 55 },
        KeySpec { name: "]", led: 56 },
        KeySpec {
            name: "\\",
            led: 58,
        },
        KeySpec {
            name: "Num 7",
            led: 59,
        },
        KeySpec {
            name: "Num 8",
            led: 60,
        },
        KeySpec {
            name: "Num 9",
            led: 61,
        },
    ],
    &[
        KeySpec {
            name: "CapsLock",
            led: 66,
        },
        KeySpec { name: "A", led: 68 },
        KeySpec { name: "S", led: 69 },
        KeySpec { name: "D", led: 70 },
        KeySpec { name: "F", led: 71 },
        KeySpec { name: "G", led: 72 },
        KeySpec { name: "H", led: 73 },
        KeySpec { name: "J", led: 74 },
        KeySpec { name: "K", led: 75 },
        KeySpec { name: "L", led: 76 },
        KeySpec { name: ";", led: 77 },
        KeySpec { name: "'", led: 78 },
        KeySpec {
            name: "Enter",
            led: 80,
        },
        KeySpec {
            name: "Num 4",
            led: 81,
        },
        KeySpec {
            name: "Num 5",
            led: 82,
        },
        KeySpec {
            name: "Num 6",
            led: 83,
        },
        KeySpec {
            name: "Num +",
            led: 62,
        },
    ],
    &[
        KeySpec {
            name: "Shift L",
            led: 88,
        },
        KeySpec { name: "Z", led: 90 },
        KeySpec { name: "X", led: 91 },
        KeySpec { name: "C", led: 92 },
        KeySpec { name: "V", led: 93 },
        KeySpec { name: "B", led: 94 },
        KeySpec { name: "N", led: 95 },
        KeySpec { name: "M", led: 96 },
        KeySpec { name: ",", led: 97 },
        KeySpec { name: ".", led: 98 },
        KeySpec { name: "/", led: 99 },
        KeySpec {
            name: "Shift R",
            led: 100,
        },
        KeySpec {
            name: "▲",
            led: 102,
        },
        KeySpec {
            name: "Num 1",
            led: 103,
        },
        KeySpec {
            name: "Num 2",
            led: 104,
        },
        KeySpec {
            name: "Num 3",
            led: 105,
        },
        KeySpec {
            name: "Num Enter",
            led: 106,
        },
    ],
    &[
        KeySpec {
            name: "Ctrl L",
            led: 110,
        },
        KeySpec {
            name: "Win",
            led: 111,
        },
        KeySpec {
            name: "Alt L",
            led: 112,
        },
        KeySpec {
            name: "Space",
            led: 116,
        },
        KeySpec {
            name: "Alt R",
            led: 119,
        },
        KeySpec {
            name: "Fn",
            led: 120,
        },
        KeySpec {
            name: "Ctrl R",
            led: 121,
        },
        KeySpec {
            name: "◀",
            led: 123,
        },
        KeySpec {
            name: "▼",
            led: 124,
        },
        KeySpec {
            name: "▶",
            led: 125,
        },
        KeySpec {
            name: "Num 0",
            led: 126,
        },
        KeySpec {
            name: "Num Del",
            led: 127,
        },
    ],
];
const KEYBOARD_LAYOUT: &[KeyPlacement] = &[
    key_at!("Esc", 0.0, 0),
    key_at!("F1", 1.3, 0),
    key_at!("F2", 2.3, 0),
    key_at!("F3", 3.3, 0),
    key_at!("F4", 4.3, 0),
    key_at!("F5", 5.6, 0),
    key_at!("F6", 6.6, 0),
    key_at!("F7", 7.6, 0),
    key_at!("F8", 8.6, 0),
    key_at!("F9", 9.8, 0),
    key_at!("F10", 10.8, 0),
    key_at!("F11", 11.8, 0),
    key_at!("F12", 12.8, 0),
    key_at!("Del", 14.0, 0),
    key_at!("Ins", 15.4, 0),
    key_at!("PgUp", 16.4, 0),
    key_at!("PgDn", 17.4, 0),
    key_at!("`", 0.0, 1),
    key_at!("1", 1.0, 1),
    key_at!("2", 2.0, 1),
    key_at!("3", 3.0, 1),
    key_at!("4", 4.0, 1),
    key_at!("5", 5.0, 1),
    key_at!("6", 6.0, 1),
    key_at!("7", 7.0, 1),
    key_at!("8", 8.0, 1),
    key_at!("9", 9.0, 1),
    key_at!("0", 10.0, 1),
    key_at!("-", 11.0, 1),
    key_at!("=", 12.0, 1),
    key_at!("Backspace", 13.0, 1, 2.0),
    key_at!("Num Lock", 15.4, 1),
    key_at!("Num /", 16.4, 1),
    key_at!("Num *", 17.4, 1),
    key_at!("Num -", 18.4, 1),
    key_at!("Tab", 0.0, 2, 1.5),
    key_at!("Q", 1.5, 2),
    key_at!("W", 2.5, 2),
    key_at!("E", 3.5, 2),
    key_at!("R", 4.5, 2),
    key_at!("T", 5.5, 2),
    key_at!("Y", 6.5, 2),
    key_at!("U", 7.5, 2),
    key_at!("I", 8.5, 2),
    key_at!("O", 9.5, 2),
    key_at!("P", 10.5, 2),
    key_at!("[", 11.5, 2),
    key_at!("]", 12.5, 2),
    key_at!("\\", 13.5, 2, 1.5),
    key_at!("Num 7", 15.4, 2),
    key_at!("Num 8", 16.4, 2),
    key_at!("Num 9", 17.4, 2),
    key_at!("CapsLock", 0.0, 3, 1.75),
    key_at!("A", 1.75, 3),
    key_at!("S", 2.75, 3),
    key_at!("D", 3.75, 3),
    key_at!("F", 4.75, 3),
    key_at!("G", 5.75, 3),
    key_at!("H", 6.75, 3),
    key_at!("J", 7.75, 3),
    key_at!("K", 8.75, 3),
    key_at!("L", 9.75, 3),
    key_at!(";", 10.75, 3),
    key_at!("'", 11.75, 3),
    key_at!("Enter", 12.75, 3, 2.25),
    key_at!("Num 4", 15.4, 3),
    key_at!("Num 5", 16.4, 3),
    key_at!("Num 6", 17.4, 3),
    key_at!("Num +", 18.4, 2, 1.0, 2.0),
    key_at!("Shift L", 0.0, 4, 2.25),
    key_at!("Z", 2.25, 4),
    key_at!("X", 3.25, 4),
    key_at!("C", 4.25, 4),
    key_at!("V", 5.25, 4),
    key_at!("B", 6.25, 4),
    key_at!("N", 7.25, 4),
    key_at!("M", 8.25, 4),
    key_at!(",", 9.25, 4),
    key_at!(".", 10.25, 4),
    key_at!("/", 11.25, 4),
    key_at!("Shift R", 12.25, 4, 1.75),
    key_at!("▲", 14.3, 4),
    key_at!("Num 1", 15.4, 4),
    key_at!("Num 2", 16.4, 4),
    key_at!("Num 3", 17.4, 4),
    key_at!("Num Enter", 18.4, 4, 1.0, 2.0),
    key_at!("Ctrl L", 0.0, 5),
    key_at!("Win", 1.0, 5),
    key_at!("Alt L", 2.0, 5),
    key_at!("Space", 3.0, 5, 6.25),
    key_at!("Alt R", 9.25, 5),
    key_at!("Fn", 10.25, 5),
    key_at!("Ctrl R", 11.25, 5),
    key_at!("◀", 13.3, 5),
    key_at!("▼", 14.3, 5),
    key_at!("▶", 15.3, 5),
    key_at!("Num 0", 16.4, 5, 1.0),
    key_at!("Num Del", 17.4, 5, 1.0),
];



#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    name: String,
    colors: BTreeMap<String, crate::rgb::Rgb>,
}

impl Default for Profile {
    fn default() -> Self {
        let colors = KEYBOARD_ROWS
            .iter()
            .flat_map(|row| row.iter())
            .map(|key| {
                (
                    key.name.to_string(),
                    Rgb {
                        r: 55,
                        g: 65,
                        b: 81,
                    },
                )
            })
            .collect();
        Self {
            name: "Default".to_string(),
            colors,
        }
    }
}

pub struct KeyboardApp {
    config: Arc<Mutex<crate::config::Config>>,
    profile: Profile,
    profile_path: PathBuf,
    selected_key: String,
    selected_keys: BTreeSet<String>,
    status: String,
}

impl KeyboardApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: Arc<Mutex<crate::config::Config>>) -> Self {
        let profile_path = crate::config::Config::profile_path().expect("Failed to compute profile path");
        let profile = Self::load_profile_from_path(&profile_path);
        Self {
            config,
            profile,
            profile_path,
            selected_key: "Esc".to_string(),
            selected_keys: BTreeSet::from(["Esc".to_string()]),
            status: String::new(),
        }
    }

    fn selected_color(&self) -> egui::Color32 {
        self.profile.colors.get(&self.selected_key).copied().unwrap_or_default().egui()
    }

    fn set_selected_color(&mut self, color: egui::Color32) {
        let rgb = Rgb::from_egui(color);
        for key_name in &self.selected_keys {
            self.profile.colors.insert(key_name.clone(), rgb);
        }
        self.status = format!(
            "Color applied to: {}",
            self.selected_keys
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    fn load_profile_from_path(path: &PathBuf) -> Profile {
        fs::read_to_string(path)
            .ok()
            .and_then(|json| serde_json::from_str::<Profile>(&json).ok())
            .unwrap_or_default()
    }

    fn load_profile(&mut self) {
        self.profile = Self::load_profile_from_path(&self.profile_path);
        self.status = format!("Profile loaded: {}", self.profile_path.display());
    }

    fn save_profile(&mut self) {
        match serde_json::to_string_pretty(&self.profile)
            .map_err(|error| error.to_string())
            .and_then(|json| {
                if let Some(parent) = self.profile_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                fs::write(&self.profile_path, json).map_err(|error| error.to_string())
            }) {
            Ok(()) => self.status = format!("Profile saved: {}", self.profile_path.display()),
            Err(error) => self.status = format!("Error: {error}"),
        }
    }

    fn show_keyboard(&mut self, ui: &mut egui::Ui) {
        const GRID_COLS: f32 = 24.0;
        const GRID_ROWS: f32 = 6.0;
        let gap = 4.0;
        let key_unit: f32 = 45.0;
        let key_height = key_unit * 0.99;
        let row_pitch = key_height + gap;
        let font_size = (key_unit * 0.21).clamp(6.0, 11.0);

        let canvas_width = GRID_COLS * key_unit + (GRID_COLS - 1.0) * gap;
        let canvas_height = GRID_ROWS * key_height + (GRID_ROWS - 1.0) * gap;

        egui::Frame::default().inner_margin(egui::Margin::same(4)).outer_margin(egui::Margin::ZERO).show(ui, |ui| {
            let canvas_size = egui::vec2(canvas_width, canvas_height);
            let (canvas, _) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
            let origin = canvas.min;

            for placement in KEYBOARD_LAYOUT {
                let Some(spec) = KEYBOARD_ROWS
                    .iter()
                    .flat_map(|row| row.iter())
                    .find(|key| key.name == placement.name)
                else {
                    continue;
                };
                let enabled = spec.led < crate::RGB_TABLE_LEN / 3;
                let color = self
                    .profile
                    .colors
                    .get(spec.name)
                    .copied()
                    .unwrap_or_default()
                    .egui();
                let selected = self.selected_keys.contains(spec.name);
                let key_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        origin.x + placement.x * (key_unit + gap),
                        origin.y + placement.y as f32 * row_pitch,
                    ),
                    egui::vec2(
                        placement.width * key_unit + (placement.width - 1.0) * gap,
                        placement.height * key_height + (placement.height - 1.0) * gap,
                    ),
                );
                let response = ui.put(
                    key_rect,
                    egui::Button::new(
                        egui::RichText::new(display_key_label(spec.name))
                            .size(font_size)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(color)
                    .stroke(egui::Stroke::new(
                        if selected { 2.0_f32 } else { 1.0_f32 },
                        if selected {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::from_gray(95)
                        },
                    ))
                    .corner_radius(5.0),
                );
                let response = response.on_hover_text(if enabled {
                    format!("{} · LED {}", spec.name, spec.led)
                } else {
                    format!("{} · LED not calibrated", spec.name)
                });
                if response.clicked() && enabled {
                    let ctrl_held = ui.input(|input| input.modifiers.ctrl);
                    update_key_selection(&mut self.selected_keys, spec.name, ctrl_held);
                    self.selected_key = spec.name.to_string();
                }
            }
        });
    }
}

impl eframe::App for KeyboardApp {
    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        self.config.lock().save().ok();
        self.save_profile();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        ctx.input(|i| {
            if let Some(pos) = i.viewport().outer_rect {
                {
                    let mut config = self.config.lock();
                    config.update_window_state(window_width, window_height, pos.min.x, pos.min.y);
                }
            }
        });

        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(crate::APP_NAME);
                ui.separator();
                ui.label(format!("Profile: {}", self.profile_path.display()));
            });
        });

        egui::Panel::bottom("bottom").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Tip: Click — one key; Ctrl+Click — multichoose").weak());
                ui.separator();
                ui.label(&self.status);
            });
        });

        egui::Panel::left("left").show(ui, |ui| {
            ui.set_max_width(100.0);
            ui.vertical(|ui| {
                ui.add_space(30.0);
                ui.label("Quick colors:");

                ui.separator();

                let col_width = (ui.available_width() - 2.0 * 4.0) / 2.0;
                for chunk in crate::QUICK_COLORS.chunks(2) {
                    ui.horizontal(|ui| {
                        for &color in chunk {
                            let btn = egui::Button::new(egui::RichText::new(" ").size(2.0)).fill(color).corner_radius(4.0);
                            if ui.add_sized([col_width, 20.0], btn).clicked() {
                                self.set_selected_color(color);
                            }
                        }
                    });
                }

                ui.separator();

                if ui.button("Reset colors").clicked() {
                    self.profile = Profile::default();
                    self.status = "Colors reset done".to_string();
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            self.show_keyboard(ui);

            ui.separator();

            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let selected_led = KEYBOARD_ROWS.iter().flat_map(|row| row.iter())
                            .find(|key| key.name == self.selected_key).map(|key| key.led);
                        ui.label(format!("{} · LED {} · selected {}", self.selected_key, selected_led.map_or_else(|| "?".to_string(), |led| led.to_string()), self.selected_keys.len()));
                        ui.label(self.selected_keys.iter().cloned().collect::<Vec<_>>().join(", "));
                        ui.separator();
                        let mut color = self.selected_color();
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            self.set_selected_color(color);
                        }
                        ui.label(format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()));
                    });
                    ui.add_space(6.0);
                    if ui.button(format!("Apply to selected ({})", self.selected_keys.len())).clicked() {
                        match apply_selected_keys_to_slot_1(&self.profile, &self.selected_keys) {
                            Ok(()) => self.status = format!("Color applied for {} selected keys; other saved", self.selected_keys.len()),
                            Err(error) => self.status = error,
                        }
                    }
                });
                ui.separator();
                ui.group(|ui| {
                    if ui.button("Save profile").clicked() {
                        self.save_profile();
                    }
                    if ui.button("Load profile").clicked() {
                        self.load_profile();
                    }
                    if ui.button("Apply all changes").clicked() {
                        match apply_profile_to_slot_1(&self.profile) {
                            Ok(()) => self.status = "Saved to slot 1".to_string(),
                            Err(error) => self.status = error,
                        }
                    }
                });
            });
        });
    }
}
