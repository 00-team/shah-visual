mod app;
mod db;
mod config;
mod shortcuts;
mod tiles;
mod utils;
mod error;

pub use error::Result;

fn main() {
    unsafe { std::env::set_var("RUST_LOG", "info") };
    pretty_env_logger::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.persistence_path = Some("./pref.json".into());
    eframe::run_native(
        "00-team-test-app",
        native_options,
        Box::new(|cc| Ok(Box::new(app::ShahApp::new(cc).unwrap()))),
    )
    .unwrap();
}
