use crate::db::Database;
use crate::utils::{self};
use eframe::{App, CreationContext};
use egui::Context;
use egui::ViewportCommand;
use shah::db::entity::Entity;
use shah::db::snake::SnakeHead;
use shah::error::SystemError;

enum MyPane {
    Pond(Vec<Pond>),
    Note(Vec<Note>),
    Origin(Vec<Origin>),
}

impl MyPane {
    fn title(&self) -> &'static str {
        match self {
            Self::Pond(_) => "Ponds",
            Self::Note(_) => "Notes",
            Self::Origin(_) => "Origins",
        }
    }
}

#[derive(Default)]
struct MyBehavior {
    pond_index: Option<usize>,
    note_index: Option<usize>,
    origin_index: Option<usize>,
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
            MyPane::Pond(ponds) => {
                let mut s = egui::ScrollArea::vertical();
                if let Some(i) = self.pond_index {
                    let n = i as f32;
                    s = s.vertical_scroll_offset(n * (301.0 + 3.0));
                    self.pond_index = None;
                }
                s.show_rows(ui, 301.0, ponds.len(), |ui, range| {
                    let min = range.start;
                    for (i, p) in ponds[range].iter().enumerate() {
                        let i = i + min;
                        egui::Frame::default()
                            .fill(ui.style().visuals.window_fill)
                            .stroke(utils::stroke(p.is_alive(), ui.visuals()))
                            .inner_margin(8.0)
                            .outer_margin(8.0)
                            .rounding(5.0)
                            .show(ui, |ui| {
                                ui.label(format!("index: {i}"));
                                ui.separator();
                                utils::gene(&p.gene, "Gene", ui);
                                if utils::gene(&p.origin, "Origin", ui).clicked() {
                                    self.origin_index = Some(p.origin.id as usize);
                                }
                                if utils::gene(&p.next, "Next", ui).clicked() {
                                    self.pond_index = Some(p.next.id as usize);
                                }
                                if utils::gene(&p.past, "Past", ui).clicked() {
                                    self.pond_index = Some(p.past.id as usize);
                                }
                                if ui.button(format!("stack: {}", p.stack)).clicked() {
                                    self.note_index = Some(p.stack as usize);
                                }
                                ui.label(format!("alive: {}", p.alive));
                                ui.label(format!("empty: {}", p.empty));
                                ui.add(utils::ColoredBool::new("is free", p.is_free()));
                                ui.add(utils::ColoredBool::new("is alive", p.is_alive()));
                                ui.add(utils::ColoredBool::new("is edited", p.is_edited()));
                                ui.add(utils::ColoredBool::new("is private", p.is_private()));
                            })
                            .response
                            .rect;

                        // let h = x.max.y - x.min.y;
                        // log::info!("ponds height: {h}");
                    }
                });
            }
            MyPane::Note(notes) => {
                let mut s = egui::ScrollArea::vertical();
                if let Some(i) = self.note_index {
                    let n = i as f32;
                    s = s.vertical_scroll_offset(n * (246.0 + 3.0));
                    self.note_index = None;
                }
                s.show_rows(ui, 246.0, notes.len(), |ui, range| {
                    let min = range.start;
                    for (i, n) in notes[range].iter().enumerate() {
                        let i = i + min;
                        egui::Frame::default()
                            .fill(ui.style().visuals.window_fill)
                            .stroke(utils::stroke(n.is_alive(), ui.visuals()))
                            .inner_margin(8.0)
                            .outer_margin(8.0)
                            .rounding(5.0)
                            .show(ui, |ui| {
                                ui.label(format!("index: {i}"));
                                ui.separator();
                                utils::gene(&n.gene, "Gene", ui);
                                if utils::gene(&n.pond, "Pond", ui).clicked() {
                                    self.pond_index = Some(n.pond.id as usize);
                                }
                                if utils::gene(&n.user, "User", ui).clicked() {}
                                ui.separator();
                                ui.label(n.note());
                                ui.separator();
                                ui.add(utils::ColoredBool::new("is alive", n.is_alive()));
                                ui.add(utils::ColoredBool::new("is edited", n.is_edited()));
                                ui.add(utils::ColoredBool::new("is private", n.is_private()));
                            });
                    }
                });
            }
            MyPane::Origin(origins) => {
                let mut s = egui::ScrollArea::vertical();
                if let Some(i) = self.origin_index {
                    let n = i as f32;
                    s = s.vertical_scroll_offset(n * (276.0 + 3.0));
                    self.origin_index = None;
                }
                s.show_rows(ui, 276.0, origins.len(), |ui, range| {
                    let min = range.start;
                    for (i, o) in origins[range].iter().enumerate() {
                        let i = i + min;
                        egui::Frame::default()
                            .fill(ui.style().visuals.window_fill)
                            .stroke(utils::stroke(o.is_alive(), ui.visuals()))
                            .inner_margin(8.0)
                            .outer_margin(8.0)
                            .rounding(5.0)
                            .show(ui, |ui| {
                                ui.label(format!("index: {i}"));
                                ui.separator();
                                utils::gene(&o.gene, "Gene", ui);
                                if utils::gene(&o.owner, "Owner", ui).clicked() {}
                                if utils::gene(&o.first, "First", ui).clicked() {
                                    self.pond_index = Some(o.first.id as usize);
                                }
                                if utils::gene(&o.last, "Last", ui).clicked() {
                                    self.pond_index = Some(o.last.id as usize);
                                }
                                ui.label(format!("items: {}", o.items));
                                ui.label(format!("ponds: {}", o.ponds));
                                ui.add(utils::ColoredBool::new("is alive", o.is_alive()));
                                ui.add(utils::ColoredBool::new("is edited", o.is_edited()));
                                ui.add(utils::ColoredBool::new("is private", o.is_private()));
                            })
                            .response
                            .rect;
                    }
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
            tiles.insert_pane(MyPane::Origin(db.origins)),
            tiles.insert_pane(MyPane::Pond(db.ponds)),
            tiles.insert_pane(MyPane::Note(db.notes)),
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
