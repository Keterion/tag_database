use rusqlite::{Connection, Result};
//use db::*;

pub struct Database {
    db: Connection,
    pub tags: Tags,
    pub images: Images,
    pub groups: Groups,
    pub namespaces: Namespaces,
    pub subtags: Subtags,
}

struct Tags {}
struct Images {}
struct Groups {}
struct Namespaces {}
struct Subtags {}

impl Default for Database {
    fn default() -> Self {
        Database {
            db: Connection::open_in_memory().unwrap(),
            tags: Tags {},
            images: Images {},
            groups: Groups {},
            namespaces: Namespaces {},
            subtags: Subtags {},
        }
    }
}

impl Database {
    pub fn open(path: &str) -> Database {
        Database {
            db: Connection::open(path).unwrap(),
            ..Default::default()
        }
    }
}
