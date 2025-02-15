// use crate::db::Database;

use eframe::{App, CreationContext};
use egui::Context;
use egui::ViewportCommand;
use shah::error::SystemError;

use crate::shortcuts as sc;

// #[derive(Default)]
pub struct ShahApp {
    settings: bool,
    fullscreen: bool,
    side_panel: bool,
    // tree: egui_tiles::Tree<MyPane>,
    // behavior: MyBehavior,
    frame: f32,
    cpu_usage: f32,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
    fdg: egui_file_dialog::FileDialog,
}

impl ShahApp {
    pub fn new(cc: &CreationContext<'_>) -> Result<Self, SystemError> {
        cc.egui_ctx.style_mut(|style| {
            let w = 8.0;
            style.spacing.scroll.bar_width = w;
            style.spacing.scroll.floating_allocated_width = w - 2.0;
            style.spacing.scroll.handle_min_length = 24.0;

            style.wrap_mode = Some(egui::TextWrapMode::Extend);
            for (_, font_id) in style.text_styles.iter_mut() {
                font_id.size += 5.0;
            }
        });

        // let db = Database::init();

        // let mut tiles = egui_tiles::Tiles::default();
        // let tabs = vec![
        // tiles.insert_pane(MyPane::Index(db.heads)),
        // tiles.insert_pane(MyPane::Snake(db.snake)),
        // tiles.insert_pane(MyPane::Origin(db.origins)),
        // tiles.insert_pane(MyPane::Pond(db.ponds)),
        // tiles.insert_pane(MyPane::Note(db.notes)),
        // ];
        // let root = tiles.insert_horizontal_tile(tabs);
        // let tree = egui_tiles::Tree::new("main_tree", root, tiles);

        Ok(Self {
            settings: false,
            fullscreen: false,
            side_panel: false,
            // tree: egui_tiles::Tree<MyPane>,
            // behavior: MyBehavior,
            frame: 0.0,
            cpu_usage: 0.0,
            dropped_files: vec![],
            picked_path: None,
            fdg: egui_file_dialog::FileDialog::new(),
            // ..Default::default()
        })
    }
}

impl App for ShahApp {
    fn persist_egui_memory(&self) -> bool {
        false
    }
    fn update(&mut self, ctx: &Context, f: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_shortcut(&sc::QUIT)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&sc::FULLSCREEN)) {
            self.fullscreen = !self.fullscreen;
        }

        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.fullscreen));
        egui::Window::new("Settings")
            .open(&mut self.settings)
            .scroll([true, true])
            .constrain(true)
            .show(ctx, |ui| ctx.settings_ui(ui));

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::default().inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("settings").clicked() {
                        self.settings = !self.settings;
                    }
                    ui.checkbox(&mut self.fullscreen, "fullscreen");
                    ui.checkbox(&mut self.side_panel, "side panel");
                    self.frame += 1.0;
                    if self.frame % 10.0 == 0.0 {
                        let cpu = f.info().cpu_usage.unwrap_or_default();
                        self.cpu_usage = cpu * 1e3;
                    }
                    ui.label(format!("cpu usage: {}ms", self.cpu_usage));
                })
            });

        egui::SidePanel::right("side-panel")
            .resizable(false)
            .show_animated(ctx, self.side_panel, |_ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Drag-and-drop files onto the window!");

            if ui.button("open efd").clicked() {
                self.fdg.pick_file();
            }

            self.fdg.update(ctx);
            if let Some(path) = self.fdg.take_picked() {
                self.picked_path = path.to_str().map(|v| v.to_string());
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });

        // egui::CentralPanel::default().show(ctx, |ui| self.tree.ui(&mut self.behavior, ui));
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
    use std::fmt::Write as _;

    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }
    let text = ctx.input(|i| {
        let mut text = "Dropping files:\n".to_owned();
        for file in &i.raw.hovered_files {
            if let Some(path) = &file.path {
                write!(text, "\n{}", path.display()).ok();
            } else if !file.mime.is_empty() {
                write!(text, "\n{}", file.mime).ok();
            } else {
                text += "\n???";
            }
        }
        text
    });

    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

    let screen_rect = ctx.screen_rect();
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
    painter.text(
        screen_rect.center(),
        Align2::CENTER_CENTER,
        text,
        TextStyle::Heading.resolve(&ctx.style()),
        Color32::WHITE,
    );
}
