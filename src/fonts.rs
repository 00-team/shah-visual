use std::sync::Arc;

fn add_font(
    fonts: &mut egui::FontDefinitions, name: &'static str, data: &'static [u8],
) {
    fonts
        .font_data
        .insert(name.to_string(), Arc::new(egui::FontData::from_static(data)));

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, name.to_string());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(name.to_string());
}

pub fn fonts_update(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    macro_rules! add_all {
        ($([$name:literal; $path:literal],)*) => {
            $(add_font(&mut fonts, $name, include_bytes!($path));)*
        };
    }

    add_all!(
        ["segmdl2"; "/usr/share/fonts/TTF/segmdl2.ttf"],
        ["segoeprb"; "/usr/share/fonts/TTF/segoeprb.ttf"],
        ["segoepr"; "/usr/share/fonts/TTF/segoepr.ttf"],
        ["segoescb"; "/usr/share/fonts/TTF/segoescb.ttf"],
        ["segoesc"; "/usr/share/fonts/TTF/segoesc.ttf"],
        ["segoeuib"; "/usr/share/fonts/TTF/segoeuib.ttf"],
        ["segoeuii"; "/usr/share/fonts/TTF/segoeuii.ttf"],
        ["segoeuil"; "/usr/share/fonts/TTF/segoeuil.ttf"],
        ["segoeuisl"; "/usr/share/fonts/TTF/segoeuisl.ttf"],
        ["segoeui"; "/usr/share/fonts/TTF/segoeui.ttf"],
        ["segoeuiz"; "/usr/share/fonts/TTF/segoeuiz.ttf"],
        ["seguibli"; "/usr/share/fonts/TTF/seguibli.ttf"],
        ["seguibl"; "/usr/share/fonts/TTF/seguibl.ttf"],
        ["seguiemj"; "/usr/share/fonts/TTF/seguiemj.ttf"],
        ["seguihis"; "/usr/share/fonts/TTF/seguihis.ttf"],
        ["seguili"; "/usr/share/fonts/TTF/seguili.ttf"],
        ["seguisbi"; "/usr/share/fonts/TTF/seguisbi.ttf"],
        ["seguisb"; "/usr/share/fonts/TTF/seguisb.ttf"],
        ["seguisli"; "/usr/share/fonts/TTF/seguisli.ttf"],
        ["seguisym"; "/usr/share/fonts/TTF/seguisym.ttf"],
        ["jbmnnfr"; "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-Regular.ttf"],
    );

    ctx.set_fonts(fonts);
}
