use shah::{db::snake::SnakeHead, error::SystemError};
use shah::models::Binary;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub struct Database {
    pub heads: Vec<SnakeHead>,
    pub snake: Vec<u8>,
    // pub origins: Vec<Origin>,
    // pub ponds: Vec<Pond>,
    // pub notes: Vec<Note>,
}

impl Database {
    pub fn init() -> Database {
        Self {
            heads: read_items("/home/i007c/projects/00-team/shah/data/detail.index.bin")
                .expect("heads read"),
            snake: read_snake("/home/i007c/projects/00-team/shah/data/detail.snake.bin")
                .expect("snake read"),
            // origins: read_items("/home/i007c/projects/00-team/shah/data/note.pond.origin.bin")
            //     .expect("origins read"),
            // ponds: read_items("/home/i007c/projects/00-team/shah/data/note.pond.index.bin")
            //     .expect("ponds read"),
            // notes: read_items("/home/i007c/projects/00-team/shah/data/note.pond.items.bin")
            // .expect("notes read"),
        }
    }
}

fn read_items<T: Binary + Default>(path: &str) -> Result<Vec<T>, SystemError> {
    let mut file = File::options().read(true).open(path)?;

    let db_size = file.seek(SeekFrom::End(0))? as usize;
    file.seek(SeekFrom::Start(0))?;
    let mut items = Vec::<T>::with_capacity(db_size / T::S);
    loop {
        let mut item = T::default();
        let rs = file.read(item.as_binary_mut())?;
        if rs != T::S {
            break;
        }
        items.push(item);
    }
    Ok(items)
}

fn read_snake(path: &str) -> Result<Vec<u8>, SystemError> {
    let mut file = File::options().read(true).open(path)?;

    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf)?;

    Ok(buf)
}
