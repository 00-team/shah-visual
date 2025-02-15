use core::panic;
use std::{fs::read_to_string, path::PathBuf, sync::OnceLock};

#[derive(serde::Deserialize, serde::Serialize)]
struct ConfigToml {
    quick_access: toml::Table,
}

#[derive(Debug, Default)]
pub struct Config {
    pub quick_access: Vec<(String, PathBuf)>,
}

pub fn config() -> &'static Config {
    static STATE: OnceLock<Config> = OnceLock::new();

    let data = read_to_string("./config.toml").expect("no config.toml file");
    let conf = toml::from_str::<ConfigToml>(&data).expect("bad config.toml");
    let mut config = Config::default();

    for (key, val) in conf.quick_access.iter() {
        match val {
            toml::Value::String(s) => {
                let p = PathBuf::from(s);
                if !p.is_dir() {
                    panic!("the path: {p:?} is not a directory");
                }
                config.quick_access.push((key.to_string(), p));
            }
            _ => panic!("invalid quick_access. only strings are valid."),
        }
    }

    STATE.get_or_init(|| config)
}
