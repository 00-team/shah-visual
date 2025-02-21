use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::ops::Range;
use std::usize;
use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

use egui_extras as ee;
use shah::db::entity::{EntityHead, EntityKochProg, ENTITY_META};
use shah::models::{Binary, Gene, Schema, SchemaModel};
use shah::DbError;

use crate::Result;

use super::Database;

macro_rules! _from_iter {
    (str) => {{
        let before = it.as_slice();
        let pos = it.position(|a| *a == 0)?;
        let res = String::from_utf8(before[..pos].to_vec()).ok()?;
        res
    }};
    ($ty:ty) => {{
        let mut size = [0u8; core::mem::size_of::<$ty>()];
        for s in size.iter_mut() {
            *s = *it.next()?;
        }
        <$ty>::from_le_bytes(size)
    }};
}

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

macro_rules! eb {
    ($enm:ident, $($v:ident),*) => {
        $(
            $enm::$v
        )|*
    };
}

macro_rules! prim_branch {
    () => {
        eb!(Schema, U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, Bool, Gene)
    };
}

fn string_schema_value(schema: &Schema, v: &[u8]) -> String {
    macro_rules! prim {
        ($ty:ty) => {{
            let vp = <$ty>::from_le_bytes(v.try_into().unwrap());
            vp.to_string()
        }};
    }
    match schema {
        Schema::U8 => prim!(u8),
        Schema::U16 => prim!(u16),
        Schema::U32 => prim!(u32),
        Schema::U64 => prim!(u64),
        Schema::I8 => prim!(i8),
        Schema::I16 => prim!(i16),
        Schema::I32 => prim!(i32),
        Schema::I64 => prim!(i64),
        Schema::F32 => prim!(f32),
        Schema::F64 => prim!(f64),
        Schema::Bool => {
            let vp = u8::from_le_bytes(v.try_into().unwrap()) != 0;
            vp.to_string()
        }
        Schema::Gene => {
            if !v.iter().any(|x| *x != 0) {
                return "Gene(---)".to_string();
            }

            let g = Gene::from_binary(v);
            format!("Gene({}, {}, {:?}, {})", g.id, g.iter, g.pepper, g.server)
        }
        _ => unreachable!(),
    }
}

fn show_schema_value(
    schema: &Schema, v: &[u8], label: Option<&str>, idx: usize, depth: usize,
    ui: &mut egui::Ui,
) {
    let mut i = 0usize;
    // let mut it = value.iter();
    match schema {
        Schema::Model(m) => {
            if let Some(label) = label {
                ui.label(label);
            }
            // ui.style_mut().wrap_mode = None;
            // ui.label(format!("<{} {} />", m.name, m.size));
            // if m.fields.is_empty() {
            //     return;
            // }

            let col = egui::CollapsingHeader::new(format!("<{} />", m.name))
                .id_salt((&m.name, idx, depth))
                .default_open(depth == 0)
                .show_background(true);

            col.show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                for (name, kind) in m.fields.iter() {
                    let s = kind.size();
                    let vv = &v[i..i + s];
                    i += s;

                    // ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    match kind {
                        prim_branch!() => {
                            // ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            ui.label(format!(
                                "{name}: {}",
                                string_schema_value(kind, vv)
                            ));
                        }
                        _ => {
                            // ui.horizontal(|ui| {
                            // ui.style_mut().wrap_mode = None;
                            // ui.horizontal(|ui| {
                            // ui.label(format!("{name}:"));
                            show_schema_value(
                                kind,
                                vv,
                                Some(name),
                                0,
                                depth + 1,
                                ui,
                            );
                            // });
                            // });
                        }
                    }
                }
            });
            // let fields = m.fields.split_at(m.fields.len() / 2);
            // log::debug!("fields: {fields:?}")
            // ui.columns_const::<2>(|cols| {
            //     for ui in cols.iter_mut() {
            //     }
            // })
            // ui.vertical(|ui| { });
        }
        Schema::Array { length, kind, is_str } => {
            if matches!(**kind, Schema::U8) && *is_str {
                let sv = v.splitn(2, |x| *x == 0).next().unwrap();
                let s = match core::str::from_utf8(sv) {
                    Ok(v) => v,
                    Err(e) => {
                        core::str::from_utf8(&v[..e.valid_up_to()]).unwrap()
                    }
                };
                ui.horizontal(|ui| {
                    let width = ui.fonts(|f| {
                        f.glyph_width(
                            &egui::TextStyle::Body.resolve(ui.style()),
                            ' ',
                        )
                    });
                    ui.spacing_mut().item_spacing.x = width;
                    if let Some(label) = label {
                        ui.label(label);
                        ui.label(
                            egui::RichText::new(":").color(egui::Color32::GOLD),
                        );
                    }
                    if s.is_empty() {
                        ui.label(
                            egui::RichText::new("<empty>")
                                .color(egui::Color32::PURPLE),
                        );
                    } else {
                        ui.label(s);
                    }
                });
                return;
            }
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            macro_rules! schema_num_align {
                ($ty:ty) => {{
                    let (head, data, tail) = unsafe { v.align_to::<$ty>() };
                    assert!(head.is_empty());
                    assert!(tail.is_empty());
                    ui.label(format!(
                        "{}: [{}; {}]: {data:?}",
                        label.unwrap_or_default(),
                        stringify!($ty),
                        data.len()
                    ));
                    return;
                }};
            }
            schema_numbers!(&(**kind), schema_num_align);

            let s = kind.size();

            // ui.vertical(|ui| {
            if let Some(label) = label {
                ui.label(format!("{label}: ["));
            } else {
                ui.label("[");
            }
            for idx in 0..*length {
                show_schema_value(
                    kind,
                    &v[i..i + s],
                    None,
                    idx as usize,
                    depth + 1,
                    ui,
                );
                i += s;
            }
            ui.label("]");
            // });
        }
        Schema::Tuple(_items) => {}

        _ => unreachable!(),
    }
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

pub struct EntityDb {
    pub file: File,
    //path: PathBuf,
    pub name: String,
    pub revision: u16,
    pub item_size: u64,
    pub schema: SchemaModel,
    pub prev_id: u64,
    pub id: u64,
    pub total: u64,
    pub koch_prog: EntityKochProg,
    pub items: Vec<Vec<u8>>,
    pub n_items: usize,
    pub prev_n_items: usize,
    pub fields: Vec<Field>,
}

impl Database for EntityDb {
    fn show(&mut self, ui: &mut egui::Ui) {
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

        let n_items = 10usize;
        let mut db = Self {
            file,
            // path,
            name: head.db_head.name().to_string(),
            revision: head.db_head.revision,
            item_size: head.item_size,
            prev_id: 0,
            id: 0,
            total: 0,
            prev_n_items: n_items,
            n_items,
            koch_prog: Default::default(),
            items: Vec::with_capacity(n_items),
            fields,
            schema,
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
        self.total = (pos - ENTITY_META) / self.item_size;
    }

    pub fn update_koch_prog(&mut self) {
        self.file
            .read_exact_at(self.koch_prog.as_binary_mut(), EntityHead::N)
            .expect("could not read");
    }

    pub fn update_items(&mut self) {
        let max = (self.id + self.n_items as u64).min(self.total);
        self.items.clear();
        for id in self.id..max {
            let mut buf = Vec::with_capacity(self.item_size as usize);
            unsafe { buf.set_len(self.item_size as usize) }
            let pos = ENTITY_META + id * self.item_size;
            self.file.read_exact_at(buf.as_mut_slice(), pos).expect("read");
            self.items.push(buf)
        }
    }

    // pub fn show_item(&self, item: &[u8], ui: &mut egui::Ui) {
    //     let mut it = item.iter();
    //     ui.label(&self.schema.name);
    //     self.schema.size;
    //     self.schema.fields;
    // }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.prev_id != self.id {
            self.prev_id = self.id;
            self.update_items();
        }
        if self.prev_n_items != self.n_items {
            self.prev_n_items = self.n_items;
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
            ui.label(format!("total items: {}", self.total));
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.id, 0..=self.total).text("id"));
            ui.separator();
            let nr = 1..=self.total as usize;
            ui.add(egui::Slider::new(&mut self.n_items, nr).text("n items"));
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
        let fields = self.fields.iter().filter(|f| f.visible);
        // egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Frame::new().stroke(ui.style().visuals.window_stroke).show(
            ui,
            |ui| {
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
                        body.rows(18.0, self.items.len(), |mut row| {
                            let idx = row.index();
                            let item = &self.items[idx];
                            let id = self.id + idx as u64;
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
            },
        );
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
}
