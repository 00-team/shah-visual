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
