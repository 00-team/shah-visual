use super::{Database, Value};
use crate::Result;
use egui_extras as ee;
use shah::db::entity::{EntityHead, EntityKochProg, ENTITY_META};
use shah::models::{Binary, Gene, Schema, SchemaModel};
use shah::DbError;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::ops::{DerefMut, Range};
use std::usize;
use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

macro_rules! schema_numbers {
    ($kind:expr, $macro:ident $($arg:tt)*) => {
        match $kind {
            Schema::U8  => $macro!(u8  $($arg)*),
            Schema::U16 => $macro!(u16 $($arg)*),
            Schema::U32 => $macro!(u32 $($arg)*),
            Schema::U64 => $macro!(u64 $($arg)*),
            Schema::I8  => $macro!(i8  $($arg)*),
            Schema::I16 => $macro!(i16 $($arg)*),
            Schema::I32 => $macro!(i32 $($arg)*),
            Schema::I64 => $macro!(i64 $($arg)*),
            Schema::F32 => $macro!(f32 $($arg)*),
            Schema::F64 => $macro!(f64 $($arg)*),
            _ => {}
        }
    };
}

pub struct Field {
    pub range: Range<usize>,
    pub show: fn(value: &[u8], ui: &mut egui::Ui),
    pub name: String,
    pub visible: bool,
}

impl Field {
    fn get_ty(schema: &Schema) -> String {
        macro_rules! schema_num_show {
            ($ty:ty) => {{
                return String::from(stringify!($ty));
            }};
        }
        schema_numbers!(schema, schema_num_show);
        match schema {
            Schema::Gene => "Gene".to_string(),
            Schema::Bool => "bool".to_string(),
            Schema::Model(m) => format!("<{} />", m.name),
            Schema::Array { is_str, length, kind } => {
                if *is_str {
                    return "str".to_string();
                }
                format!("[{}; {}]", Field::get_ty(&(*kind)), *length)
            }
            Schema::Tuple(items) => {
                let x = items
                    .iter()
                    .map(Field::get_ty)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({x})")
            }
            _ => "unknown".to_string(),
        }
    }

    fn get_show(schema: &Schema) -> fn(&[u8], &mut egui::Ui) {
        fn do_nothing(_: &[u8], _: &mut egui::Ui) {}

        macro_rules! schema_num_show {
            ($ty:ty) => {{
                fn show_prim_num(v: &[u8], ui: &mut egui::Ui) {
                    let vp = <$ty>::from_le_bytes(v.try_into().unwrap());
                    ui.label(vp.to_string());
                }
                return show_prim_num;
            }};
        }
        schema_numbers!(schema, schema_num_show);

        match schema {
            Schema::Bool => {
                fn show_prim_bool(v: &[u8], ui: &mut egui::Ui) {
                    let vp = u8::from_le_bytes(v.try_into().unwrap()) != 0;
                    ui.label(vp.to_string());
                }
                return show_prim_bool;
            }
            Schema::Gene => {
                fn show_gene(v: &[u8], ui: &mut egui::Ui) {
                    if !v.iter().any(|x| *x != 0) {
                        ui.label("Gene(---)");
                        return;
                    }

                    let g = Gene::from_binary(v);

                    ui.label(format!(
                        "Gene({}, {}, {:?}, {})",
                        g.id, g.iter, g.pepper, g.server
                    ));
                }
                return show_gene;
            }
            Schema::Array { is_str, kind, .. } => {
                if matches!(**kind, Schema::U8) && *is_str {
                    fn show_str(v: &[u8], ui: &mut egui::Ui) {
                        let sv = v.splitn(2, |x| *x == 0).next().unwrap();
                        let s = match core::str::from_utf8(sv) {
                            Ok(v) => v,
                            Err(e) => {
                                core::str::from_utf8(&v[..e.valid_up_to()])
                                    .unwrap()
                            }
                        };
                        if s.is_empty() {
                            ui.label(
                                egui::RichText::new("<empty>")
                                    .color(egui::Color32::PURPLE),
                            );
                        } else {
                            ui.label(s);
                        }
                    }
                    return show_str;
                }

                macro_rules! schema_num_align {
                    ($ty:ty) => {{
                        fn show_prim_arr(v: &[u8], ui: &mut egui::Ui) {
                            let (head, data, tail) =
                                unsafe { v.align_to::<$ty>() };
                            assert!(head.is_empty());
                            assert!(tail.is_empty());
                            ui.label(format!("{data:?}"));
                        }
                        return show_prim_arr;
                    }};
                }
                schema_numbers!(&(**kind), schema_num_align);
            }
            _ => {}
        }

        do_nothing
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct EntityPersist {
    field_visibility: Vec<(String, bool)>,
}

pub struct EntityDb {
    pub file: File,
    //path: PathBuf,
    pub name: String,
    pub revision: u16,
    pub schema: SchemaModel,
    pub koch_prog: EntityKochProg,
    pub item_size: u64,
    pub item_total: u64,
    pub item_skip: Value<u64>,
    pub item_show: Value<u64>,
    pub item_data: Vec<Vec<u8>>,
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
        format!("{}.{}", self.name, self.revision)
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
        for (fi, fs) in schema.fields.iter() {
            let s = fs.size();
            fields.push(Field {
                range: i..i + s,
                name: format!("{fi}: {}", Field::get_ty(fs)),
                show: Field::get_show(fs),
                visible: true,
            });
            i += s;
        }

        let mut db = Self {
            file,
            // path,
            name: head.db_head.name().to_string(),
            revision: head.db_head.revision,
            item_size: head.item_size,
            item_skip: Value::new(0),
            item_show: Value::new(10),
            item_data: Vec::with_capacity(10),
            item_total: 0,
            koch_prog: Default::default(),
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

    pub fn update_items(&mut self) {
        let skip = self.item_skip.main();
        let show = self.item_show.main();
        let max = (skip + show).min(self.item_total);
        self.item_data.clear();
        for id in skip..max {
            let mut buf = Vec::with_capacity(self.item_size as usize);
            unsafe { buf.set_len(self.item_size as usize) }
            let pos = ENTITY_META + id * self.item_size;
            self.file.read_exact_at(buf.as_mut_slice(), pos).expect("read");
            self.item_data.push(buf)
        }
    }

    // pub fn show_item(&self, item: &[u8], ui: &mut egui::Ui) {
    //     let mut it = item.iter();
    //     ui.label(&self.schema.name);
    //     self.schema.size;
    //     self.schema.fields;
    // }

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
        });
        ui.horizontal_wrapped(|ui| {
            for f in self.fields.iter_mut() {
                ui.checkbox(&mut f.visible, &f.name);
            }
        });
        let colps = egui::CollapsingHeader::new("schema").show_background(true);
        colps.show(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.monospace(format!("{:#?}", self.schema));
            });
        });
        // egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::new()
            .stroke(ui.style().visuals.window_stroke)
            .show(ui, |ui| self.show_items(ui));
        // if self.item.len() == self.item_size as usize {
        //     show_schema_value(
        //         &self.schema,
        //         self.item.as_slice(),
        //         None,
        //         0,
        //         0,
        //         ui,
        //     );
        // } else {
        //     ui.label("reading...");
        // }
        // for item in self.items.iter() {
        // ui.label(format!("{item:?}"));
        // }
        // });
    }

    fn show_items(&mut self, ui: &mut egui::Ui) {
        let fields = self.fields.iter().filter(|f| f.visible);
        ee::TableBuilder::new(ui)
            .column(ee::Column::exact(20.0))
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
            .header(23.0, |mut header| {
                header.col(|ui| {
                    ui.heading("id");
                });
                for f in fields.clone() {
                    header.col(|ui| {
                        ui.heading(&f.name);
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
                            (f.show)(&item[f.range.clone()], ui);
                        });
                    }
                });
                // let max = (self.id + self.n_items as u64)
                //     .min(self.total)
                //     - self.id;
                // body.rows(18.0, self.items.len(), |mut row| {
                //     let item = self.items[row.index()];
                //     let i = self.id + row.index() as u64;
                //     row.col(|ui| {
                //         ui.label(i.to_string());
                //     });
                //     for _ in 0..self.schema.fields.len() {
                //         row.col(|ui| {
                //             ui.separator();
                //         });
                //     }
                // });
                // for id in self.id..max {
                //     body.rows
                // }
            });
    }
}
