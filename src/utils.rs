use std::path::{Component, Path};

// use shah::models::Gene;

// pub fn gene(gene: &Gene, name: &'static str, ui: &mut egui::Ui) -> egui::Response {
//     ui.add_enabled(
//         gene.is_some(),
//         egui::Button::new(format!(
//             "{name}({}, {}, {:?}, {})",
//             gene.id, gene.iter, gene.pepper, gene.server
//         )),
//     )
// }
//
// pub fn stroke(alive: bool, free: bool, visuals: &egui::Visuals) -> egui::Stroke {
//     if !alive {
//         return egui::Stroke::new(visuals.window_stroke.width, visuals.error_fg_color);
//     }
//
//     if free {
//         return egui::Stroke::new(visuals.window_stroke.width, visuals.warn_fg_color);
//     }
//
//     visuals.window_stroke
// }

// pub struct ColoredBool {
//     name: &'static str,
//     value: bool,
// }
//
// impl ColoredBool {
//     pub fn new(name: &'static str, value: bool) -> Self {
//         Self { name, value }
//     }
// }
//
// impl egui::Widget for ColoredBool {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         ui.horizontal_wrapped(|ui| {
//             ui.label(format!("{}:", self.name));
//             if self.value {
//                 ui.colored_label(egui::Color32::LIGHT_GREEN, "true");
//             } else {
//                 ui.colored_label(egui::Color32::LIGHT_RED, "false");
//             }
//         })
//         .response
//     }
// }

pub fn db_name(path: &Path) -> (&str, &str, String) {
    let mut after_data = false;
    let mut x = "";
    let mut y = Vec::<&str>::new();
    for p in path.components().rev() {
        let Component::Normal(a) = p else { continue };
        let g = a.to_str().unwrap();
        if after_data {
            return (g, x, y[..y.len().min(2)].join("/"));
        }
        if a == "data" {
            after_data = true;
        } else {
            y.insert(0, x);
            // y = format!("{x}/{y}");
            x = g;
        }
    }
    ("", x, y[..y.len().min(2)].join("/"))
}
