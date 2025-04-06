use shah::models::{Binary, Gene, Schema};
use std::ops::Range;

fn show_gene(v: &[u8], ui: &mut egui::Ui) {
    if !v.iter().any(|x| *x != 0) {
        ui.label("---");
        return;
    }

    let g = Gene::from_binary(v);

    ui.label(format!(
        "Gene({}, {}, {:?}, {})",
        g.id, g.iter, g.pepper, g.server
    ));
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

pub struct Field {
    pub range: Range<usize>,
    pub show: fn(value: &[u8], ui: &mut egui::Ui),
    pub name: String,
    pub visible: bool,
    pub show_array: bool,
}

impl Field {
    pub fn get_ty(schema: &Schema) -> String {
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
                format!("[{}; {}]", Field::get_ty(kind), *length)
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

    pub fn get_show(schema: &Schema) -> (fn(&[u8], &mut egui::Ui), bool) {
        fn do_nothing(_: &[u8], _: &mut egui::Ui) {}

        macro_rules! schema_num_show {
            ($ty:ty) => {{
                fn show_prim_num(v: &[u8], ui: &mut egui::Ui) {
                    let vp = <$ty>::from_le_bytes(v.try_into().unwrap());
                    ui.label(vp.to_string());
                }
                return (show_prim_num, true);
            }};
        }
        schema_numbers!(schema, schema_num_show);

        match schema {
            Schema::Bool => {
                fn show_prim_bool(v: &[u8], ui: &mut egui::Ui) {
                    let vp = u8::from_le_bytes(v.try_into().unwrap()) != 0;
                    ui.label(vp.to_string());
                }
                return (show_prim_bool, true);
            }
            Schema::Gene => {
                return (show_gene, true);
            }
            Schema::Array { is_str, kind, .. } => {
                if matches!(**kind, Schema::Gene) {
                    fn show_genes(v: &[u8], ui: &mut egui::Ui) {
                        let list = v.chunks(Gene::S).enumerate();
                        let list = list.collect::<Vec<_>>();
                        let scroll = egui::ScrollArea::vertical();
                        let scroll = scroll.auto_shrink([false, false]);
                        scroll.show_rows(ui, 20.0, list.len(), |ui, s| {
                            for (i, c) in &list[s] {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{i}:"));
                                    show_gene(c, ui);
                                });
                            }
                        });
                    }
                    return (show_genes, false);
                }

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
                    return (show_str, true);
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
                        return (show_prim_arr, true);
                    }};
                }
                schema_numbers!(&(**kind), schema_num_align);
            }
            _ => {}
        }

        (do_nothing, true)
    }
}
