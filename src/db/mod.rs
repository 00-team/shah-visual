mod entity;
mod trie_const;

use crate::Result;
use entity::EntityDb;
use shah::models::{Binary, DbHead, ShahMagicDb};
use shah::DbError;
use std::{fs::OpenOptions, os::unix::fs::FileExt, path::PathBuf};

use self::trie_const::TrieConstDb;

trait Database: Sized {
    fn init(path: PathBuf) -> Result<Self>;
    fn title(&self) -> String;
    fn show(&mut self, ui: &mut egui::Ui);
}

pub struct DbTile {
    pub kind: DatabaseKind,
    pub path: PathBuf,
}

impl PartialEq for DbTile {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl DbTile {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self { kind: DatabaseKind::new(path.clone())?, path })
    }

    pub fn title(&self) -> String {
        self.kind.title()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.kind.show(ui);
    }
}

pub enum DatabaseKind {
    Entity(EntityDb),
    TrieConst(TrieConstDb),
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
            ShahMagicDb::Entity => {
                Self::Entity(<EntityDb as Database>::init(path)?)
            }
            ShahMagicDb::TrieConst => {
                Self::TrieConst(<TrieConstDb as Database>::init(path)?)
            }
            _ => return Err(DbError::InvalidDbHead)?,
        })
    }

    pub fn title(&self) -> String {
        match self {
            Self::Entity(db) => Database::title(db),
            Self::TrieConst(db) => Database::title(db),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        match self {
            Self::Entity(db) => Database::show(db, ui),
            Self::TrieConst(db) => Database::show(db, ui),
        }
    }
}
