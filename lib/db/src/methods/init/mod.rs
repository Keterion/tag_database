use rusqlite::{Connection, Result};
pub fn recreate_db(path: std::path::PathBuf) -> Connection {
    match std::fs::remove_file(&path) {
        Ok(_) => {},
        Err(err) => eprintln!("Error removing file:\n{}", err),
    }
    if let Ok(conn) = Connection::open(path) {
        init_tables(&conn).unwrap();
        conn
    } else {
        panic!("Failed to open database")
    }
}
/// creates the tables needed
pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
        id      INTEGER PRIMARY KEY,
        name    TEXT NOT NULL UNIQUE)",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS namespaces (
        id      INTEGER PRIMARY KEY,
        name    TEXT NOT NULL UNIQUE)",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS images (
        id      INTEGER PRIMARY KEY,
        path    TEXT NOT NULL UNIQUE)",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS namespace_map (
        id              INTEGER PRIMARY KEY,
        namespace_id    INTEGER,
        tag_id          INTEGER UNIQUE,
        FOREIGN KEY(namespace_id) REFERENCES namespaces(id) ON DELETE CASCADE,
        FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE)",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS subtag_map (
        id          INTEGER PRIMARY KEY,
        parent_id   INTEGER,
        child_id    INTEGER,
        FOREIGN KEY(parent_id) REFERENCES tags(id) ON DELETE CASCADE,
        FOREIGN KEY(child_id) REFERENCES tags(id) ON DELETE CASCADE,
        CONSTRAINT duplicate_subtag UNIQUE(parent_id, child_id))",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tag_map (
        id          INTEGER PRIMARY KEY,
        img_id      INTEGER,
        tag_id      INTEGER,
        FOREIGN KEY (img_id) REFERENCES images(id) ON DELETE CASCADE,
        FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
        CONSTRAINT duplicate_tag UNIQUE(img_id, tag_id))",
        (),
    )?;
    Ok(())
}
