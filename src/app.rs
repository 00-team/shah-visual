// use crate::db::Database;

use std::path::PathBuf;

use eframe::{App, CreationContext};
use egui::Context;
use egui::ViewportCommand;
use shah::error::SystemError;

use crate::config::config;
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
    db_paths: Vec<PathBuf>,
    file_dialog: egui_file_dialog::FileDialog,
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

        let config = crate::config::config();
        let file_dialog = egui_file_dialog::FileDialog::new().add_quick_access("Hi", |qa| {
            for (d, p) in config.quick_access.iter() {
                qa.add_path(&d, p);
            }
        });

        Ok(Self {
            settings: false,
            fullscreen: false,
            side_panel: false,
            // tree: egui_tiles::Tree<MyPane>,
            // behavior: MyBehavior,
            frame: 0.0,
            cpu_usage: 0.0,
            db_paths: vec![],
            file_dialog,
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

            if ui.button("add databases").clicked() {
                self.file_dialog.pick_multiple();
            }

            self.file_dialog.update(ctx);
            if let Some(paths) = self.file_dialog.take_picked_multiple() {
                self.db_paths = paths;
            }

            ui.group(|ui| {
                ui.label("db paths:");
                for p in self.db_paths.iter() {
                    ui.monospace(format!("{p:?}"));
                }
            });
        });

        // egui::CentralPanel::default().show(ctx, |ui| self.tree.ui(&mut self.behavior, ui));
    }
}
