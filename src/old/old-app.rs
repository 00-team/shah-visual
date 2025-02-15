use crate::db::Database;
use crate::utils::{self};
use eframe::{App, CreationContext};
use egui::Context;
use egui::ViewportCommand;
use shah::db::entity::Entity;
use shah::db::snake::SnakeHead;
use shah::error::SystemError;
use shah::AsUtf8Str;

enum MyPane {
    Index(Vec<SnakeHead>),
    Snake(Vec<u8>),
}

impl MyPane {
    fn title(&self) -> &'static str {
        match self {
            Self::Index(_) => "Index",
            Self::Snake(_) => "Snake",
        }
    }
}

#[derive(Default)]
struct MyBehavior {
    index_index: Option<usize>,
    snake_head: Option<SnakeHead>,
}

impl egui_tiles::Behavior<MyPane> for MyBehavior {
    fn tab_title_for_pane(&mut self, pane: &MyPane) -> egui::WidgetText {
        pane.title().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _: egui_tiles::TileId,
        pane: &mut MyPane,
    ) -> egui_tiles::UiResponse {
        match pane {
            MyPane::Index(heads) => {
                const H: f32 = 249.0;
                let mut s = egui::ScrollArea::vertical();
                if let Some(i) = self.index_index {
                    let n = i as f32;
                    s = s.vertical_scroll_offset(n * (H + 3.0));
                    self.index_index = None;
                }
                s.show_rows(ui, H, heads.len(), |ui, range| {
                    let min = range.start;
                    for (i, head) in heads[range].iter().enumerate() {
                        let i = i + min;
                        egui::Frame::default()
                            .fill(ui.style().visuals.window_fill)
                            .stroke(utils::stroke(head.is_alive(), head.is_free(), ui.visuals()))
                            .inner_margin(8.0)
                            .outer_margin(8.0)
                            .rounding(5.0)
                            .show(ui, |ui| {
                                ui.label(format!("index: {i}"));
                                ui.separator();
                                utils::gene(&head.gene, "Gene", ui);
                                if ui.button(format!("position: {}", head.position)).clicked() {
                                    self.snake_head = Some(head.clone());
                                }

                                ui.label(format!("capacity: {}", head.capacity));
                                ui.label(format!("length: {}", head.length));
                                let next = head.position + head.capacity;
                                if ui.button(format!("next: {next}")).clicked() {
                                    if let Some((i, _)) =
                                        heads.iter().enumerate().find(|(_, h)| h.position == next)
                                    {
                                        self.index_index = Some(i);
                                    }
                                }
                                if ui.button(format!("past")).clicked() {
                                    if let Some((i, _)) = heads
                                        .iter()
                                        .enumerate()
                                        .find(|(_, h)| h.position + h.capacity == head.position)
                                    {
                                        self.index_index = Some(i);
                                    }
                                }

                                ui.add(utils::ColoredBool::new("is free", head.is_free()));
                                ui.add(utils::ColoredBool::new("is alive", head.is_alive()));
                                ui.add(utils::ColoredBool::new("is edited", head.is_edited()));
                                ui.add(utils::ColoredBool::new("is private", head.is_private()));
                            })
                            .response
                            .rect;

                        // let h = x.max.y - x.min.y;
                        // log::info!("heads height: {h}");
                    }
                });
            }

            MyPane::Snake(data) => 'a: {
                if self.snake_head.is_none() {
                    ui.label("select a head to show");
                    break 'a;
                }

                let head = self.snake_head.unwrap();
                let s = head.position as usize;
                let l = head.length.min(head.capacity) as usize;
                let l = s + l;
                let e = s + head.capacity as usize;
                if s >= data.len() {
                    ui.label(format!(
                        "head: Gene({}, {}, {:?}, {}) is out of bound\ndata length: {}",
                        head.gene.id,
                        head.gene.iter,
                        head.gene.pepper,
                        head.gene.server,
                        data.len()
                    ));
                    break 'a;
                }

                if e > data.len() {
                    ui.label(format!("capacity out of bound: {e} >= {}", data.len()));
                    ui.label(format!("available capacity: {}", data.len() - s));
                    break 'a;
                    // e = data.len();
                }

                let sss = &data[s..l - 8];
                let sss = sss.as_utf8_str();
                // let sss = String::from_utf8(sss.to_vec())
                //     .unwrap_or("error reading content of head".to_string());
                let len = u64::from_le_bytes(data[l - 8..l].try_into().unwrap());
                let cap = &data[l..e];
                let s = egui::ScrollArea::vertical();
                s.show(ui, |ui| {
                    egui::Frame::default()
                        .fill(ui.style().visuals.window_fill)
                        .stroke(utils::stroke(head.is_alive(), head.is_free(), ui.visuals()))
                        .inner_margin(8.0)
                        .outer_margin(8.0)
                        .rounding(5.0)
                        .show(ui, |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                            ui.label(format!("length: {l} | {len}"));
                            ui.separator();
                            ui.label(sss);
                            ui.separator();
                            ui.label(format!("{cap:?}"))
                        })
                        .response
                        .rect;
                });
            }
        }

        Default::default()
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

pub struct InteractiveApp {
    settings: bool,
    fullscreen: bool,
    side_panel: bool,
    tree: egui_tiles::Tree<MyPane>,
    behavior: MyBehavior,
    frame: f32,
    cpu_usage: f32,
}

impl InteractiveApp {
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

        let db = Database::init();

        let mut tiles = egui_tiles::Tiles::default();
        let tabs = vec![
            tiles.insert_pane(MyPane::Index(db.heads)),
            tiles.insert_pane(MyPane::Snake(db.snake)),
            // tiles.insert_pane(MyPane::Origin(db.origins)),
            // tiles.insert_pane(MyPane::Pond(db.ponds)),
            // tiles.insert_pane(MyPane::Note(db.notes)),
        ];
        let root = tiles.insert_horizontal_tile(tabs);
        let tree = egui_tiles::Tree::new("main_tree", root, tiles);

        Ok(Self {
            settings: false,
            fullscreen: false,
            side_panel: false,
            tree,
            behavior: MyBehavior::default(),
            frame: 0.0,
            cpu_usage: 0.0,
        })
    }
}

const QUIT: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q);
const FULLSCREEN: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F);

impl App for InteractiveApp {
    fn persist_egui_memory(&self) -> bool {
        false
    }
    fn update(&mut self, ctx: &Context, f: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_shortcut(&QUIT)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&FULLSCREEN)) {
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

        egui::CentralPanel::default().show(ctx, |ui| self.tree.ui(&mut self.behavior, ui));
    }
}
