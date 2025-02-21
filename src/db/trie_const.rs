use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::ops::{Deref, DerefMut, Range};
use std::usize;
use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

use egui::TextBuffer;
use egui_extras as ee;
use shah::db::entity::{EntityHead, EntityKochProg, ENTITY_META};
use shah::db::trie_const::TrieConstMeta;
use shah::models::{Binary, Gene, Schema, SchemaModel};
use shah::{AsUtf8Str, DbError};

use crate::Result;

use super::Database;

pub struct Value<T> {
    main: T,
    past: T,
}

impl<T: Clone + PartialEq> Value<T> {
    pub fn new(value: T) -> Self {
        Self { main: value.clone(), past: value }
    }

    pub fn changed(&mut self) -> bool {
        if self.main != self.past {
            self.past = self.main.clone();
            return true;
        }

        false
    }

    pub fn main(&self) -> T {
        self.main.clone()
    }
}

impl<T> Deref for Value<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.main
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.main
    }
}

pub struct TrieConstDb {
    file: File,
    name: String,
    index: u64,
    cache: u64,
    abc: Vec<char>,
    cache_len: u64,
    cache_skip: Value<u64>,
    cache_show: Value<u64>,
    cache_data: Vec<u64>,
    cached_cache_ui: String,
}

impl Database for TrieConstDb {
    fn title(&self) -> String {
        self.name.to_string()
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("trie const db");
        ui.label(format!("abc: {} | {:?}", self.abc.len(), self.abc));
        ui.label(format!("cache level: {}", self.cache));
        ui.label(format!("index level: {}", self.index));
        ui.label(format!("key length: {}", self.cache + self.index));

        ui.add(
            egui::Slider::new(
                self.cache_skip.deref_mut(),
                0..=self.cache_len - 1,
            )
            .text("cache skip")
            .drag_value_speed(1000.0),
        );
        ui.add(
            egui::Slider::new(self.cache_show.deref_mut(), 0..=self.cache_len)
                .text("cache show")
                .drag_value_speed(1000.0),
        );

        egui::ScrollArea::vertical().show(ui, |ui| self.show_cache(ui));
    }

    fn init(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut meta = TrieConstMeta::default();
        file.read_exact_at(meta.as_binary_mut(), 0)?;

        let abc_len = meta.abc_len as usize;
        if abc_len >= meta.abc.len() {
            return Err(DbError::InvalidDbMeta)?;
        }

        let abc = meta.abc[..abc_len].as_utf8_str().chars().collect::<Vec<_>>();
        let cache_len = (abc.len() as u64).pow(meta.cache as u32);
        if cache_len * 8 > 100 * 1024 * 1024 {
            panic!("cache is bigger then 100mig");
        }

        let mut db = Self {
            file,
            cache: meta.cache,
            index: meta.index,
            abc,
            name: meta.db.name().to_string(),
            cache_len,
            cache_skip: Value::new(0),
            cache_show: Value::new(10),
            cache_data: Vec::with_capacity(cache_len as usize),
            cached_cache_ui: String::new(),
        };

        db.update_cache_data()?;

        Ok(db)
    }
}

impl TrieConstDb {
    fn update_cache_data(&mut self) -> Result<()> {
        self.cache_skip = Value::new(self.cache_skip.min(self.cache_len - 1));
        self.cache_show = Value::new(self.cache_show.clamp(1, self.cache_len));

        let skip = self.cache_skip.main();

        let m = self.cache_show.min(self.cache_len - skip) as usize;
        let pos = TrieConstMeta::N + skip * 8;

        unsafe { self.cache_data.set_len(m) }
        let buf = &mut self.cache_data[..m];

        let (head, data, tail) = unsafe { buf.align_to_mut::<u8>() };
        assert!(head.is_empty() && tail.is_empty(), "failed align");

        self.file.read_exact_at(data, pos)?;

        self.cached_cache_ui.clear();
        let mut did_wrote_zero = false;
        let len = self.cache_data.len();
        for (i, p) in self.cache_data.iter().enumerate() {
            let is_last = i + 1 == len;
            if did_wrote_zero && *p == 0 && !is_last {
                continue;
            }

            self.cached_cache_ui
                .push_str(&format!("{i:0>w$}: {p}\n", w = self.cache as usize));
            did_wrote_zero = *p == 0;
        }

        Ok(())
    }

    fn show_cache(&mut self, ui: &mut egui::Ui) {
        if self.cache_skip.changed() || self.cache_show.changed() {
            self.update_cache_data().expect("could not update cache data");
        }

        ui.label(&self.cached_cache_ui);
    }
}
