mod field;
mod item;

use field::Field;

use super::{Database, Value};
use crate::utils::db_name;
use crate::Result;
use shah::db::entity::{EntityHead, EntityKochProg, ENTITY_META};
use shah::models::{Binary, Schema, SchemaModel};
use shah::DbError;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::ops::DerefMut;
use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct EntityPersist {
    field_visibility: Vec<(String, bool)>,
}

pub struct EntityDb {
    pub file: File,
    //path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub scope: String,
    pub prefix: String,
    pub revision: u16,
    pub schema: SchemaModel,
    pub koch_prog: EntityKochProg,
    pub item_size: u64,
    pub item_total: u64,
    pub item_skip: Value<u64>,
    pub item_show: Value<u64>,
    pub item_data: Vec<Vec<u8>>,
    pub sort_by: Option<Field>,
    active_item: Option<usize>,
    pub fields: Vec<Field>,
    read_from_mem: bool,
    id: egui::Id,
}

impl Database for EntityDb {
    fn show(&mut self, ui: &mut egui::Ui) {
        if self.read_from_mem {
            ui.ctx().memory_mut(|mem| {
                let Some(ep) = mem.data.get_persisted::<EntityPersist>(self.id)
                else {
                    return;
                };

                for (i, vis) in ep.field_visibility.iter() {
                    for f in self.fields.iter_mut() {
                        if &f.name == i {
                            f.visible = *vis;
                            break;
                        }
                    }
                }
            });
        }

        self.read_from_mem = false;
        ui.ctx().memory_mut(|mem| {
            let value = EntityPersist {
                field_visibility: self
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.visible))
                    .collect::<Vec<_>>(),
            };
            mem.data.insert_persisted(self.id, value);
        });
        self.show(ui);
    }
    fn title(&self) -> String {
        self.title()
    }
    fn init(path: PathBuf) -> Result<Self> {
        Self::init(path)
    }
}

impl EntityDb {
    fn title(&self) -> String {
        if self.prefix.is_empty() {
            format!("{}.{}", self.name, self.revision)
        } else {
            format!("{}/{}.{}", self.prefix, self.name, self.revision)
        }
    }

    pub fn init(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut head = EntityHead::default();
        file.read_exact_at(head.as_binary_mut(), 0)?;

        let schema = match Schema::decode(&head.schema)? {
            Schema::Model(m) => m,
            _ => return Err(DbError::InvalidDbSchema)?,
        };

        let mut fields = Vec::<Field>::with_capacity(schema.fields.len());
        let mut i = 0usize;
        for (fdx, (fi, fs)) in schema.fields.iter().enumerate() {
            let s = fs.size();
            let (show, show_array) = Field::get_show(fs);
            let range = i..i + s;
            fields.push(Field {
                idx: fdx,
                name: format!("{fi}: {}", Field::get_ty(fs)),
                number_stats: Field::get_number_stats(fs),
                number_sort: Field::get_number_sort(fs),
                range,
                show,
                show_array,
                visible: true,
            });
            i += s;
        }

        let (scope, prefix, _name) = db_name(&path);
        let head_name = head.db_head.name().to_string();
        let prefix = if head_name == prefix { "" } else { prefix };

        let mut db = Self {
            file,
            // path,
            name: head_name,
            prefix: prefix.to_string(),
            scope: scope.to_string(),
            revision: head.db_head.revision,
            item_size: head.item_size,
            item_skip: Value::new(0),
            item_show: Value::new(10),
            item_data: Vec::with_capacity(10),
            item_total: 0,
            active_item: None,
            koch_prog: Default::default(),
            sort_by: None,
            fields,
            schema,
            read_from_mem: true,
            id: egui::Id::new(("entity", path)),
        };

        db.update();

        Ok(db)
    }

    pub fn update(&mut self) {
        self.update_total();
        self.update_koch_prog();
        self.update_items()
    }

    pub fn update_total(&mut self) {
        let pos = self.file.seek(SeekFrom::End(0)).expect("no seek");
        self.item_total = (pos - ENTITY_META) / self.item_size;
    }

    pub fn update_koch_prog(&mut self) {
        self.file
            .read_exact_at(self.koch_prog.as_binary_mut(), EntityHead::N)
            .expect("could not read");
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.item_skip.changed() || self.item_show.changed() {
            self.update_items();
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("db: {}.{}", self.name, self.revision));
            ui.label(format!(
                "model: <{} {}/>",
                self.schema.name, self.item_size
            ));
            ui.label(format!(
                "koch prog: {}/{}",
                self.koch_prog.prog, self.koch_prog.total
            ));
            ui.label(format!("total items: {}", self.item_total));
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(
                    self.item_skip.deref_mut(),
                    0..=self.item_total,
                )
                .text("skip"),
            );
            ui.separator();
            ui.add(
                egui::Slider::new(
                    self.item_show.deref_mut(),
                    0..=self.item_total,
                )
                .text("show"),
            );
            if let Some(fsb) = &self.sort_by {
                ui.separator();
                if ui.button(format!("sort by: {}", fsb.name)).clicked() {
                    self.sort_by = None;
                }
            }
        });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for f in self.fields.iter_mut() {
                ui.checkbox(&mut f.visible, &f.name);
            }
        });
        ui.separator();
        ui.vertical(|ui| {
            for f in self.fields.iter() {
                if !f.visible {
                    continue;
                }
                let Some(ns) = f.number_stats else { continue };
                ui.label(format!(
                    "{}: {}",
                    f.name,
                    ns(&self.item_data, f.range.clone())
                ));
            }
        });
        let colps = egui::CollapsingHeader::new("schema").show_background(true);
        colps.show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.monospace(format!("{:#?}", self.schema));
            });
        });

        egui::Frame::new()
            .stroke(ui.style().visuals.window_stroke)
            .show(ui, |ui| self.show_items(ui));

        self.show_active_item(ui);
    }
}
