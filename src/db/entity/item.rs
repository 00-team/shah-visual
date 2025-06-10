use super::EntityDb;
use egui_extras as ee;
use shah::db::entity::ENTITY_META;
use std::os::unix::fs::FileExt;

impl EntityDb {
    pub(super) fn show_items(&mut self, ui: &mut egui::Ui) {
        let fields = self.fields.iter().filter(|f| f.visible);
        ee::TableBuilder::new(ui)
            .column(ee::Column::auto().resizable(true).at_least(50.0))
            .columns(
                ee::Column::remainder()
                    .resizable(true)
                    .clip(true)
                    .at_least(40.0),
                fields.clone().count(),
            )
            .striped(true)
            .resizable(true)
            .sense(egui::Sense::click())
            .header(25.0, |mut header| {
                header.col(|ui| {
                    ui.heading("id");
                });
                for f in fields.clone() {
                    header.col(|ui| {
                        if f.number_sort.is_some() {
                            if ui.button(&f.name).clicked() {
                                self.sort_by = Some(f.clone());
                            }
                        } else {
                            ui.heading(&f.name);
                        }
                    });
                }
            })
            .body(|body| {
                let skip = self.item_skip.main();
                body.rows(18.0, self.item_data.len(), |mut row| {
                    let idx = row.index();
                    let item = &self.item_data[idx];
                    let id = skip + idx as u64;
                    row.col(|ui| {
                        ui.label(id.to_string());
                    });
                    for f in fields.clone() {
                        row.col(|ui| {
                            if !f.show_array {
                                return;
                            };
                            (f.show)(&item[f.range.clone()], ui);
                        });
                    }
                    if row.response().clicked() {
                        if let Some(ax) = self.active_item.as_mut() {
                            if *ax == idx {
                                self.active_item = None;
                            } else {
                                *ax = idx;
                            }
                        } else {
                            self.active_item = Some(idx);
                        }
                    }
                });
            });
    }

    pub(super) fn update_items(&mut self) {
        let skip = self.item_skip.main();
        let show = self.item_show.main();
        let max = (skip + show).min(self.item_total);
        self.item_data.clear();
        for id in skip..max {
            let mut buf = vec![0u8; self.item_size as usize];
            let pos = ENTITY_META + id * self.item_size;
            self.file.read_exact_at(buf.as_mut_slice(), pos).expect("read");
            self.item_data.push(buf)
        }
        if let Some(sb) = &self.sort_by {
            if let Some(ns) = sb.number_sort {
                self.item_data.sort_by_key(|item| ns(&item[sb.range.clone()]));
            }
        }
    }

    pub(super) fn show_active_item(&mut self, ui: &mut egui::Ui) {
        let Some(idx) = self.active_item else { return };
        if idx >= self.item_data.len() {
            return;
        }

        let item = &self.item_data[idx];

        egui::ScrollArea::both().show(ui, |ui| {
            for f in self.fields.iter() {
                ui.label(&f.name);
                (f.show)(&item[f.range.clone()], ui);
            }
        });
    }
}
