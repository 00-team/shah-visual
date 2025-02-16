use crate::database::{Database, DatabaseKind};
use egui_tiles as et;

#[derive(Default)]
pub struct Behavior {}

impl et::Behavior<Database> for Behavior {
    fn tab_title_for_pane(&mut self, db: &Database) -> egui::WidgetText {
        egui::WidgetText::from(db.title())
    }

    fn pane_ui(
        &mut self, ui: &mut egui::Ui, _: et::TileId, db: &mut Database,
    ) -> et::UiResponse {
        match &mut db.kind {
            DatabaseKind::Entity(edb) => {
                edb.show(ui);
            }
        }
        // ui.label(&pane.title);

        // match pane {
        //     Pane::Index(heads) => {
        //         const H: f32 = 249.0;
        //         let mut s = egui::ScrollArea::vertical();
        //         if let Some(i) = self.index_index {
        //             let n = i as f32;
        //             s = s.vertical_scroll_offset(n * (H + 3.0));
        //             self.index_index = None;
        //         }
        //         s.show_rows(ui, H, heads.len(), |ui, range| {
        //             let min = range.start;
        //             for (i, head) in heads[range].iter().enumerate() {
        //                 let i = i + min;
        //                 egui::Frame::default()
        //                     .fill(ui.style().visuals.window_fill)
        //                     .stroke(utils::stroke(
        //                         head.is_alive(),
        //                         head.is_free(),
        //                         ui.visuals(),
        //                     ))
        //                     .inner_margin(8.0)
        //                     .outer_margin(8.0)
        //                     .rounding(5.0)
        //                     .show(ui, |ui| {
        //                         ui.label(format!("index: {i}"));
        //                         ui.separator();
        //                         utils::gene(&head.gene, "Gene", ui);
        //                         if ui
        //                             .button(format!(
        //                                 "position: {}",
        //                                 head.position
        //                             ))
        //                             .clicked()
        //                         {
        //                             self.snake_head = Some(head.clone());
        //                         }
        //
        //                         ui.label(format!(
        //                             "capacity: {}",
        //                             head.capacity
        //                         ));
        //                         ui.label(format!("length: {}", head.length));
        //                         let next = head.position + head.capacity;
        //                         if ui.button(format!("next: {next}")).clicked()
        //                         {
        //                             if let Some((i, _)) = heads
        //                                 .iter()
        //                                 .enumerate()
        //                                 .find(|(_, h)| h.position == next)
        //                             {
        //                                 self.index_index = Some(i);
        //                             }
        //                         }
        //                         if ui.button(format!("past")).clicked() {
        //                             if let Some((i, _)) = heads
        //                                 .iter()
        //                                 .enumerate()
        //                                 .find(|(_, h)| {
        //                                     h.position + h.capacity
        //                                         == head.position
        //                                 })
        //                             {
        //                                 self.index_index = Some(i);
        //                             }
        //                         }
        //
        //                         ui.add(utils::ColoredBool::new(
        //                             "is free",
        //                             head.is_free(),
        //                         ));
        //                         ui.add(utils::ColoredBool::new(
        //                             "is alive",
        //                             head.is_alive(),
        //                         ));
        //                         ui.add(utils::ColoredBool::new(
        //                             "is edited",
        //                             head.is_edited(),
        //                         ));
        //                         ui.add(utils::ColoredBool::new(
        //                             "is private",
        //                             head.is_private(),
        //                         ));
        //                     })
        //                     .response
        //                     .rect;
        //
        //                 // let h = x.max.y - x.min.y;
        //                 // log::info!("heads height: {h}");
        //             }
        //         });
        //     }
        //
        //     MyPane::Snake(data) => 'a: {
        //         if self.snake_head.is_none() {
        //             ui.label("select a head to show");
        //             break 'a;
        //         }
        //
        //         let head = self.snake_head.unwrap();
        //         let s = head.position as usize;
        //         let l = head.length.min(head.capacity) as usize;
        //         let l = s + l;
        //         let e = s + head.capacity as usize;
        //         if s >= data.len() {
        //             ui.label(format!(
        //                 "head: Gene({}, {}, {:?}, {}) is out of bound\ndata length: {}",
        //                 head.gene.id,
        //                 head.gene.iter,
        //                 head.gene.pepper,
        //                 head.gene.server,
        //                 data.len()
        //             ));
        //             break 'a;
        //         }
        //
        //         if e > data.len() {
        //             ui.label(format!(
        //                 "capacity out of bound: {e} >= {}",
        //                 data.len()
        //             ));
        //             ui.label(format!("available capacity: {}", data.len() - s));
        //             break 'a;
        //             // e = data.len();
        //         }
        //
        //         let sss = &data[s..l - 8];
        //         let sss = sss.as_utf8_str();
        //         // let sss = String::from_utf8(sss.to_vec())
        //         //     .unwrap_or("error reading content of head".to_string());
        //         let len =
        //             u64::from_le_bytes(data[l - 8..l].try_into().unwrap());
        //         let cap = &data[l..e];
        //         let s = egui::ScrollArea::vertical();
        //         s.show(ui, |ui| {
        //             egui::Frame::default()
        //                 .fill(ui.style().visuals.window_fill)
        //                 .stroke(utils::stroke(
        //                     head.is_alive(),
        //                     head.is_free(),
        //                     ui.visuals(),
        //                 ))
        //                 .inner_margin(8.0)
        //                 .outer_margin(8.0)
        //                 .rounding(5.0)
        //                 .show(ui, |ui| {
        //                     ui.style_mut().wrap_mode =
        //                         Some(egui::TextWrapMode::Wrap);
        //                     ui.label(format!("length: {l} | {len}"));
        //                     ui.separator();
        //                     ui.label(sss);
        //                     ui.separator();
        //                     ui.label(format!("{cap:?}"))
        //                 })
        //                 .response
        //                 .rect;
        //         });
        //     }
        // }

        Default::default()
    }

    fn simplification_options(&self) -> et::SimplificationOptions {
        et::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn is_tab_closable(&self, _: &et::Tiles<Database>, _: et::TileId) -> bool {
        true
    }
}
