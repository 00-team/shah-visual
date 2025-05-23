// use crate::db::Database;

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::{App, CreationContext};
use egui::Context;
use egui::ViewportCommand;
use egui_file_dialog as efd;
use egui_tiles as et;
use shah::error::SystemError;

use crate::db::DbTile;
use crate::fonts;
use crate::shortcuts as sc;
use crate::tiles;
use crate::utils::db_name;

// #[derive(Default)]
pub struct ShahApp {
    settings: bool,
    fullscreen: bool,
    side_panel: bool,
    tree: egui_tiles::Tree<DbTile>,
    behavior: tiles::Behavior,
    frame: f32,
    cpu_usage: f32,
    db_paths: HashMap<String, HashMap<String, Vec<(String, PathBuf)>>>,
    file_dialog: egui_file_dialog::FileDialog,
}

impl ShahApp {
    pub fn new(cc: &CreationContext<'_>) -> Result<Self, SystemError> {
        cc.egui_ctx.style_mut(|style| {
            let w = 8.0;
            style.spacing.scroll.bar_width = w;
            style.spacing.scroll.floating_allocated_width = w - 2.0;
            style.spacing.scroll.handle_min_length = 24.0;
            style.visuals.slider_trailing_fill = true;

            style.wrap_mode = Some(egui::TextWrapMode::Extend);
            for (_, font_id) in style.text_styles.iter_mut() {
                font_id.size += 5.0;
            }
        });

        fonts::fonts_update(&cc.egui_ctx);

        // let db = Database::init();

        // let mut tiles = egui_tiles::Tiles::default();
        // let root = tiles.insert_horizontal_tile(vec![]);
        // let tree = egui_tiles::Tree::empty("main-tree");

        let config = crate::config::config();

        let mut file_dialog = efd::FileDialog::new()
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .title("Select Databases");

        if !config.quick_access.is_empty() {
            file_dialog = file_dialog.add_quick_access("Quick Access", |qa| {
                for (d, p) in config.quick_access.iter() {
                    qa.add_path(d, p);
                }
            });

            let init = config.quick_access[0].1.clone();
            file_dialog = file_dialog.initial_directory(init);
        }

        let app = Self {
            settings: false,
            fullscreen: false,
            side_panel: true,
            tree: egui_tiles::Tree::empty("main-tree"),
            behavior: tiles::Behavior {},
            frame: 0.0,
            cpu_usage: 0.0,
            db_paths: HashMap::new(),
            file_dialog,
        };

        // app.add_db_path("/home/i007c/projects/00-team/shah/data/".into());

        Ok(app)
    }

    fn add_database(&mut self, path: PathBuf) {
        let old = self.tree.tiles.iter().find_map(|(tid, t)| {
            if let et::Tile::Pane(p) = t {
                if p.path == path {
                    return Some(tid);
                }
            }
            None
        });

        if let Some(id) = old {
            self.tree.tiles.remove(*id);
        }

        let db = match DbTile::new(path) {
            Ok(v) => v,
            Err(e) => {
                log::error!("error init new database: {e:#?}");
                return;
            }
        };

        let old_root = self.tree.root;

        let tab = vec![self.tree.tiles.insert_pane(db)];
        let new_root = self.tree.tiles.insert_horizontal_tile(tab);
        self.tree.root = Some(new_root);

        if let Some(rid) = old_root {
            self.tree.move_tile_to_container(rid, new_root, 1, false);
        }
    }

    fn _add_db_path(
        &mut self, path: PathBuf, depth: usize, total: usize,
    ) -> usize {
        if total >= 1000 {
            return 0;
        }
        if depth > 5 {
            return 0;
        }

        if path.is_dir() {
            let mut it = path.read_dir().unwrap();
            let max = 1000 - total;
            let mut n = 0usize;
            while let Some(Ok(p)) = it.next() {
                n += self._add_db_path(p.path(), depth + 1, total + 1);
                if n > max {
                    return n;
                }
            }
            return n;
        }

        if path.is_file() {
            self._add_file_path(path);
        }

        1
    }

    fn _add_file_path(&mut self, path: PathBuf) {
        let (scope, db, kind) = db_name(&path);
        let kind = kind.to_string();
        if let Some(dv) = self.db_paths.get_mut(scope) {
            if let Some(xx) = dv.get_mut(db) {
                xx.push((kind, path));
            } else {
                dv.insert(db.to_string(), vec![(kind, path)]);
            }
        } else {
            // self.db_paths.insert(scope.to_string(), vec![(title, path)]);
            self.db_paths.insert(
                scope.to_string(),
                HashMap::from_iter([(db.to_string(), vec![(kind, path)])]),
            );
        }
    }

    pub fn add_db_path(&mut self, path: PathBuf) {
        self._add_db_path(path, 0, 0);
    }

    pub fn add_db_paths(&mut self, paths: Vec<PathBuf>) {
        for p in paths {
            self.add_db_path(p);
        }
    }
}

impl App for ShahApp {
    fn persist_egui_memory(&self) -> bool {
        true
    }
    fn update(&mut self, ctx: &Context, f: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_shortcut(&sc::QUIT)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&sc::FULLSCREEN)) {
            self.fullscreen = !self.fullscreen;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&sc::OPEN_FILE)) {
            self.file_dialog.pick_multiple();
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
                    if ui.button("☰").clicked() {
                        self.side_panel = !self.side_panel;
                    }
                    if ui.button("Open").clicked() {
                        self.file_dialog.pick_multiple();
                    }
                    ui.menu_button("File", |ui| {
                        if ui.button("settings").clicked() {
                            self.settings = !self.settings;
                        }
                        ui.checkbox(&mut self.fullscreen, "Full Screen");
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                    self.frame += 1.0;
                    if self.frame % 10.0 == 0.0 {
                        let cpu = f.info().cpu_usage.unwrap_or_default();
                        self.cpu_usage = cpu * 1e3;
                    }
                    ui.label(format!("cpu usage: {}ms", self.cpu_usage));
                })
            });

        egui::SidePanel::left("left-side-panel")
            .resizable(false)
            .show_animated(ctx, self.side_panel, |ui| {
                ui.label(format!("db paths: {}", self.db_paths.len()));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (k, pv) in self.db_paths.clone() {
                        let col = egui::CollapsingHeader::new(&k)
                            .id_salt(("scope", &k))
                            .default_open(self.db_paths.len() == 1);
                        // .show_background(true);

                        col.show(ui, |ui| {
                            for (db, pfs) in pv.iter() {
                                if pfs.is_empty() {
                                    continue;
                                }
                                if pfs.len() == 1 {
                                    let (tit, p) = &pfs[0];
                                    let n = format!("{db}/{tit}");
                                    if ui.button(n).clicked() {
                                        self.add_database(p.clone());
                                    }
                                    continue;
                                }

                                let db_col = egui::CollapsingHeader::new(db)
                                    .id_salt(("dbccxx", db))
                                    .default_open(pfs.len() < 5);
                                // .show_background(true);

                                db_col.show(ui, |ui| {
                                    for (title, p) in pfs {
                                        if ui.button(title).clicked() {
                                            self.add_database(p.clone());
                                        }
                                    }
                                });
                            }
                        });
                    }
                });
            });

        self.file_dialog.update(ctx);
        if let Some(paths) = self.file_dialog.take_picked_multiple() {
            self.add_db_paths(paths);
        }

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.group(|ui| {});
        // });

        egui::CentralPanel::default()
            .show(ctx, |ui| self.tree.ui(&mut self.behavior, ui));
    }
}
