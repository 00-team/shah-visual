mod app;
mod config;
mod shortcuts;
mod utils;
// mod db;

fn main() {
    unsafe { std::env::set_var("RUST_LOG", "info") };
    pretty_env_logger::init();

    log::info!("config: {:?}", config::config());

    let mut native_options = eframe::NativeOptions::default();
    native_options.persistence_path = Some("./pref.json".into());
    eframe::run_native(
        "00-team-test-app",
        native_options,
        Box::new(|cc| Ok(Box::new(app::ShahApp::new(cc).unwrap()))),
    )
    .unwrap();
}
