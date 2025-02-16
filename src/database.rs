use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

use shah::db::entity::EntityHead;
use shah::models::{Binary, DbHead, Schema, SchemaModel, ShahMagicDb};
use shah::DbError;

use crate::Result;

pub struct Database {
    pub kind: DatabaseKind,
    pub path: PathBuf,
    pub name: String,
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        Ok(Self { kind: DatabaseKind::new(path.clone())?, name, path })
    }

    pub fn title(&self) -> String {
        self.kind.title()
    }
}

pub enum DatabaseKind {
    Entity(EntityDb),
}

impl DatabaseKind {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut db_head = DbHead::default();
        file.read_exact_at(db_head.as_binary_mut(), 0)?;

        if !db_head.magic.is_valid() {
            return Err(DbError::InvalidDbHead)?;
        }
        if db_head.magic.is_custom() {
            return Err(DbError::InvalidDbHead)?;
        }

        Ok(match db_head.magic.db() {
            ShahMagicDb::Entity => match EntityDb::init(path) {
                Ok(v) => Self::Entity(v),
                Err(err) => {
                    log::error!("init error: {err:#?}");
                    return Err(DbError::InvalidDbHead)?;
                }
            },
            _ => return Err(DbError::InvalidDbHead)?,
        })
    }

    pub fn title(&self) -> String {
        match self {
            Self::Entity(edb) => edb.name.clone(),
        }
    }
}

pub struct EntityDb {
    path: PathBuf,
    name: String,
    revision: u16,
    item_size: u64,
    schema: SchemaModel,
}

impl EntityDb {
    pub fn init(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new().read(true).open(&path)?;
        let mut head = EntityHead::default();
        file.read_exact_at(head.as_binary_mut(), 0)?;

        let schema = match Schema::decode(&head.schema)? {
            Schema::Model(m) => m,
            _ => return Err(DbError::InvalidDbSchema)?,
        };

        Ok(Self {
            path,
            name: head.db_head.name().to_string(),
            revision: head.db_head.revision,
            item_size: head.item_size,
            schema,
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("this is an entity db");
        ui.label(format!("{}:{}:{}", self.name, self.revision, self.item_size));
        ui.monospace(format!("{:#?}", self.schema));
    }
}
