#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod tests;

use rusqlite::{Connection, Result};

macro_rules! handle_unique {
    ($command:expr) => {
        match $command {
            Ok(_) => Ok(1),
            Err(err) => match err.sqlite_error_code() {
                Some(rusqlite::ErrorCode::ConstraintViolation) => {
                    log::warn!(
                        "ConstraintViolation\n{}\nfor\n{}\n",
                        err,
                        stringify!($command)
                    );
                    Ok(2)
                }
                _ => Err(err),
            },
        }
    };
}
/// returns Ok when successful or the insert failed with a constraintViolation
macro_rules! insert_ret_id {
    ($command:expr, $conn:expr) => {
        match handle_unique!($command) {
            Ok(1) => Some($conn.last_insert_rowid()),
            Ok(_) => None,
            Err(_) => None,
        }
    };
}
//pub mod wrapper {
//    use super::{Connection, tags, namespaces, images, utils, subtags};
//    use log::error;
//
//    macro_rules! match_err {
//        ($command:expr) => {
//            match $command {
//                Ok(thing) => {},
//                Err(err) => {
//                    error!("{}", err);
//                }
//            }
//        };
//    }
//    pub struct Database {
//        pub connection: Connection,
//    }
//    impl Database {
//        pub fn new(conn: Connection) -> Database {
//            env_logger::init();
//            Database {
//                connection: conn,
//            }
//        }
//        pub fn add_tags(&mut self, tags: Vec<&str>) -> Vec<Option<i64>> {
//            tags::add_tags(tags, &mut self.connection)
//        }
//        pub fn remove_tags(&self, tag_ids: Vec<i64>) -> Option<()> {
//            match tags::remove_tags(tag_ids, &self.connection) {
//                Ok(_) => Some(()),
//                Err(err) => {
//                    error!("{}", err);
//                    None
//                },
//            }
//        }
//        pub fn rename_tag(&self, tag_id: i64, name: &str) {
//            match_err!(tags::rename_tag(tag_id, name, &self.connection));
//        }
//        pub fn get_tag_name(&self, tag_id: i64) -> Option<String> {
//            tags::get_name(tag_id, &self.connection)
//        }
//        pub fn remove_orphans(&self) {
//            let tag_orphans = tags::get_orphans(&self.connection);
//            let namespace_orphans = namespaces::get_orphans(&self.connection);
//            let image_orphans = images::get_orphans(&self.connection);
//
//            for orphan in tag_orphans {
//                match_err!(utils::remove_id(orphan.0, "tags", &self.connection));
//            }
//            for orphan in namespace_orphans {
//                match_err!(utils::remove_id(orphan.0, "namespaces", &self.connection));
//            }
//            for orphan in image_orphans {
//                match_err!(utils::remove_id(orphan.0, "images", &self.connection));
//            }
//        }
//        pub fn set_tag_parent(&self, parent_id: i64, child_id: i64) {
//            subtags::parent_tag(parent_id, child_id, &self.connection);
//        }
//        pub fn get_tag_parents(&self, tag_id: i64) -> Vec<i64> {
//            subtags::get_parents(tag_id, &self.connection).unwrap()
//        }
//        pub fn get_tag_children(&self, tag_id: i64) -> Vec<i64> {
//            subtags::get_children(tag_id, &self.connection).unwrap()
//        }
//
//    }
//}
pub mod utils {
    use super::{Connection, Result};

    /// returns either id or none of table with matching query
    pub fn get_id(table: &str, query: &str, conn: &Connection) -> Option<i64> {
        let mut stmt = conn
            .prepare(&format!("SELECT id FROM {} WHERE {}", table, query))
            .unwrap();
        match stmt.query_row((), |row| row.get(0)) {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }
    /// removes the row with matching id in the specified table
    pub fn remove_id(id: i64, table: &str, conn: &Connection) -> Result<()> {
        conn.execute(&format!("DELETE FROM {} WHERE id=?1", table), [id])?;
        Ok(())
    }
}
pub mod tags {
    use super::{subtags, tags, utils, Connection, Result};
    /// adds tag to image, if create is true nonexistent tags will get created
    pub fn add_tag_to_img(tag: &str, img: i64, create: bool, conn: &Connection) -> Result<()> {
        let mut insert = conn.prepare("INSERT INTO tag_map(img_id, tag_id) VALUES (?1, ?2)")?;
        // primary tag
        let primary_id = match utils::get_id("tags", &format!("name='{}'", tag), conn) {
            Some(id) => id,
            None => {
                if create {
                    tags::add_tag(tag, conn).unwrap()
                } else {
                    panic!(
                        "Primary tag '{}' doesn't exist and shouldn't be created",
                        tag
                    );
                }
            }
        }; // if there was no id found, panic
        handle_unique!(insert.execute([img, primary_id]))?;

        // parent tags
        let parents = subtags::get_parents(primary_id, conn)?;
        for parent in parents {
            insert.execute([img, parent])?;
        }
        Ok(())
    }
    /// removes given tag from given image
    pub fn remove_tag_from_img(tag_id: i64, img_id: i64, conn: &Connection) -> Result<()> {
        let mut rem = conn.prepare("DELETE FROM tag_map WHERE tag_id=?1 AND img_id=?2")?;
        rem.execute([tag_id, img_id])?;
        Ok(())
    }
    /// Adds tag into the tags table and returns id if successful
    pub fn add_tag(tag: &str, conn: &Connection) -> Option<i64> {
        insert_ret_id!(
            conn.execute("INSERT INTO tags(name) VALUES (?1)", [tag]),
            &conn
        )
    }
    /// Adds multiple tags and return a vector of None and Some(id) values
    pub fn add_tags(tags: Vec<&str>, conn: &mut Connection) -> Vec<Option<i64>> {
        let mut ids: Vec<Option<i64>> = Vec::new();
        let mut stmt = conn.prepare("INSERT INTO tags(name) VALUES (?1)").unwrap();
        for tag in tags {
            ids.push(insert_ret_id!(stmt.execute([tag]), conn)); // works if the insert worked or the tag already exists, else it panics
        }
        ids
    }
    /// Deletes given tag from the tags table, all rows which use the tag also get deleted
    pub fn remove_tag(tag: i64, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM tags WHERE id=?1", [tag])?;
        // doesn't need any other stuff because of the ON DELETE CASCADE action
        Ok(())
    }
    /// Deletes all tags, delete cascades to all connected rows
    pub fn remove_tags(tags: Vec<i64>, conn: &Connection) -> Result<()> {
        for tag in tags {
            remove_tag(tag, conn)?;
        }
        Ok(())
    }
    /// Gets name of the given id or None
    pub fn get_name(tag_id: i64, conn: &Connection) -> Option<String> {
        let mut stmt = conn.prepare("SELECT name FROM tags WHERE id=?1").unwrap();
        match stmt.query_row([tag_id], |row| row.get(0)) {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }
    /// Renames the tag
    pub fn rename_tag(tag_id: i64, new_name: &str, conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE tags SET name=?1 WHERE id=?2",
            [new_name, &format!("{}", tag_id)],
        )?;
        Ok(())
    }
    /// Gets all tags without connection to an image
    pub fn get_orphans(conn: &Connection) -> Vec<(i64, String)> {
        let mut stmt = conn.prepare("SELECT id, name FROM tags WHERE NOT EXISTS (SELECT id FROM tag_map WHERE tag_map.tag_id=tags.id)").unwrap();
        let mut res: Vec<(i64, String)> = Vec::new();
        for orphan in stmt
            .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .unwrap()
        {
            res.push(orphan.unwrap());
        }
        res
    }
    pub fn get_tags_with(search_term: &str, conn: &Connection) -> Vec<(i64, String)> {
        if search_term.len() < 1 {
            return vec![];
        }
        let mut stmt = conn
            .prepare("SELECT id, name FROM tags WHERE name LIKE ?1")
            .unwrap();
        let mut res: Vec<(i64, String)> = Vec::new();
        for matching in stmt
            .query_map([&format!("%{}%", search_term)], |row| {
                Ok((row.get(0).unwrap(), row.get(1).unwrap()))
            })
            .unwrap()
        {
            res.push(matching.unwrap());
        }
        res
    }
}
pub mod images {
    use rusqlite::{Connection, Result};
    /// Adds an image into the images table, returns id if successful
    pub fn add_image(path: &str, conn: &Connection) -> Option<i64> {
        insert_ret_id!(
            conn.execute("INSERT INTO images(path) VALUES (?1)", [path]),
            &conn
        )
    }
    /// Removes an image from the images table via path, delete cascades
    pub fn remove_image_path(path: &str, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM images WHERE path=?1", [path])?;
        Ok(())
    }
    /// Gets all images which have a given tag
    pub fn get_images_with_tag(tag_id: i64, conn: &Connection) -> Result<Vec<(i64, String)>> {
        let mut stmt = conn.prepare(
            "SELECT (images.id, images.path)
            FROM images
            JOIN tag_map
            WHERE tag_map.tag_id=?1
            AND tag_map.img_id=images.id",
        )?;
        let mut res: Vec<(i64, String)> = vec![];
        let q = stmt.query_map([tag_id], |row| {
            Ok((row.get::<usize, i64>(0)?, row.get::<usize, String>(1)?))
        })?;
        for img in q {
            res.push(img.unwrap());
        }
        Ok(res)
    }
    /// Returns a vector of tag ids and names which are linked with the given image
    pub fn get_tags_of_img(img_id: i64, conn: &Connection) -> Vec<(i64, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT tag_map.tag_id, tags.name
            FROM tag_map
            INNER JOIN tags
            WHERE tag_map.img_id=?1
            AND tags.id=tag_map.tag_id",
            )
            .unwrap();
        let q = stmt
            .query_map([img_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();
        let mut res: Vec<(i64, String)> = vec![];
        for id in q {
            res.push(id.unwrap());
        }
        res
    }
    /// Gets the path of the given image id or None
    pub fn get_path(img_id: i64, conn: &Connection) -> Option<String> {
        let mut stmt = conn.prepare("SELECT path FROM images WHERE id=?1").unwrap();
        match stmt.query_row([img_id], |row| row.get(0)) {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }
    /// Updates the path of a given image id with a new one
    pub fn update_path(id: i64, new_path: &str, conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE images SET path=?1 WHERE id=?2",
            [new_path, &format!("{}", id)],
        )?;
        Ok(())
    }
    /// Returns a vector of image ids and paths which aren't connected to any tags
    pub fn get_orphans(conn: &Connection) -> Vec<(i64, String)> {
        let mut stmt = conn
            .prepare(
                "
            SELECT id, path
            FROM images
            WHERE NOT EXISTS (
                SELECT id
                FROM tag_map
                WHERE tag_map.img_id=images.id
            )
        ",
            )
            .unwrap();
        let mut res: Vec<(i64, String)> = Vec::new();
        for orphan in stmt
            .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .unwrap()
        {
            res.push(orphan.unwrap());
        }
        res
    }
    pub fn query_sql(query: &str, conn: &Connection) -> Vec<(i64, String)> {
        let tokens = query.split(" ");
        let mut constructed = String::new();
        let mut res: Vec<(i64, String)> = vec![];
        constructed.push_str("SELECT img.id, img.path FROM images img ");
        let mut tag_id: i64;

        //let mut exclude: Vec<&str> = vec![];
        let mut joins = 0;
        for token in tokens {
            //if token.starts_with('!') {
            //    exclude.push(&token[1..]); // exclude the exclamation mark
            //}
            if let Some(id) = crate::utils::get_id("tags", &format!("name='{}'", token), conn) {
                constructed.push_str(&format!(
                    "JOIN tag_map t{} ON img.id = t{}.img_id AND t{}.tag_id = '{}'",
                    joins, joins, joins, id
                ));
                joins += 1;
            }
        }

        let mut stmt = conn.prepare(&constructed).unwrap();
        for image in stmt
            .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .unwrap()
        {
            match image {
                Ok(val) => res.push(val),
                Err(err) => eprint!("{}", err),
            }
        }

        res
    }
}
pub mod subtags {
    use super::{utils, Connection, Result};
    /// Gets all results of a query with a given variable as well as the results of the query with
    /// the results
    macro_rules! recurse {
        ($query:expr, $primary_id:expr) => {{
            let prim_ids = $query.query_map([$primary_id], |row| row.get(0)).unwrap();

            let mut curr: Vec<i64> = Vec::new();
            let mut total: Vec<i64> = Vec::new();
            let mut curr_id: i64;

            for primary in prim_ids {
                curr.push(primary.unwrap());
            }
            while !curr.is_empty() {
                curr_id = curr.pop().unwrap();
                total.push(curr_id); // insert last of vec (popped) into total
                for new in $query.query_map([curr_id], |row| row.get(0))? {
                    curr.insert(0, new.unwrap());
                } // add new ones in front since we take from the back
            }
            Ok(total)
        }};
    }
    /// Gets all parents of a tag
    pub fn get_parents(child_id: i64, conn: &Connection) -> Result<Vec<i64>> {
        let mut stmt = conn.prepare("SELECT parent_id FROM subtag_map WHERE child_id=?1")?;
        recurse!(stmt, child_id)
    }
    /// Gets all children of a tag
    pub fn get_children(parent_id: i64, conn: &Connection) -> Result<Vec<i64>> {
        let mut stmt = conn.prepare("SELECT child_id FROM subtag_map WHERE parent_id=?1")?;
        recurse!(stmt, parent_id)
    }
    /// Sets a tag as parent of another tag, returns None if it fails
    pub fn parent_tag(parent_id: i64, child_id: i64, conn: &Connection) -> Option<()> {
        if Option::is_none(&utils::get_id(
            "subtag_map",
            &format!("parent_id={} AND child_id={}", child_id, parent_id),
            conn,
        )) {
            match handle_unique!(conn.execute(
                "INSERT INTO subtag_map(parent_id, child_id) VALUES (?1, ?2)",
                [parent_id, child_id],
            )) {
                Ok(_) => {}
                Err(err) => return None,
            }
        } else {
            return None;
        } // return None if there would be a loop created or the insert failed
        Some(())
    }
    /// Deletes the connection between two tags
    pub fn remove_connection(tag1: &str, tag2: &str, conn: &Connection) -> Option<()> {
        let t1_id = match utils::get_id("tags", &format!("name='{}'", tag1), conn) {
            Some(id) => id,
            None => return None,
        };
        let t2_id = utils::get_id("tags", &format!("name='{}'", tag2), conn).unwrap();

        match conn.execute(
            "
                     DELETE FROM subtag_map
                     WHERE (parent_id=?1 AND child_id=?2)
                     OR (parent_id=?2 AND child_id=?1)
                     ",
            [t1_id, t2_id],
        ) {
            Ok(_) => Some(()),
            Err(err) => {
                println!("{}", err);
                None
            }
        }
    }
}
pub mod namespaces {
    use super::{Connection, Result};
    /// Adds a namespace into the namespaces table, returns id if successful
    pub fn add_namespace(name: &str, conn: &Connection) -> Option<i64> {
        match handle_unique!(conn.execute("INSERT INTO namespaces(name) VALUES (?1)", [name])) {
            Ok(_) => Some(conn.last_insert_rowid()),
            Err(_) => None,
        }
    }
    /// Changes namespace name of given namespace id
    pub fn rename_namespace(id: i64, new_name: &str, conn: &Connection) -> Result<()> {
        conn.execute(
            "UPDATE namespaces SET name=?1 WHERE id=?2",
            [new_name, &format!("{}", id)],
        )?;
        Ok(())
    }
    /// Adds a namespace to a tag, only one namespace per tag is allowed
    pub fn add_namespace_to_tag(namespace: i64, tag: i64, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO namespace_map(namespace_id, tag_id) VALUES (?1, ?2)",
            [namespace, tag],
        )?;
        Ok(())
    }
    /// Removes namespace from tag, as there can only be one you don't need to specify it
    pub fn remove_namespace_from_tag(tag: i64, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM namespace_map WHERE tag_id=?1", [tag])?;
        Ok(())
    }
    /// Removes the namespace from the namespaces table, removing all associations
    pub fn remove_namespace(namespace_id: i64, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM namespaces WHERE id=?1", [namespace_id])?;
        Ok(())
    }
    /// Gets the namespace from a namespace id
    pub fn get_namespace_name(namespace_id: i64, conn: &Connection) -> Option<String> {
        let mut stmt = conn
            .prepare("SELECT (name) FROM namespaces WHERE id=?1")
            .unwrap();
        match stmt.query_row([namespace_id], |row| row.get(0)) {
            Ok(name) => Some(name),
            Err(_) => None,
        }
    }
    /// Gets the namespace id associated with a tag
    pub fn get_namespace_of_tag(tag_id: i64, conn: &Connection) -> Option<i64> {
        let mut stmt = conn
            .prepare("SELECT namespace_id FROM namespace_map WHERE tag_id=?1")
            .unwrap();
        match stmt.query_row([tag_id], |row| row.get::<usize, i64>(0)) {
            Ok(id) => Some(id),
            Err(err) => None,
        }
    }
    /// Gets all of the tags connected with a namespace
    pub fn get_tags_with_namespace(namespace_id: i64, conn: &Connection) -> Vec<(i64, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT tags.id, tags.name
            FROM tags
            INNER JOIN namespace_map
            WHERE namespace_map.namespace_id=?1
            AND namespace_map.tag_id=tags.id",
            )
            .unwrap();
        let tmp = stmt
            .query_map([namespace_id], |row| {
                Ok((row.get(0).unwrap(), row.get(1).unwrap()))
            })
            .unwrap();
        let mut res: Vec<(i64, String)> = Vec::new();
        for val in tmp {
            res.push(val.unwrap());
        }
        res
    }
    /// Gets all namespaces which aren't linked with any tag
    pub fn get_orphans(conn: &Connection) -> Vec<(i64, String)> {
        let mut stmt = conn
            .prepare(
                "
            SELECT id, name
            FROM namespaces
            WHERE NOT EXISTS (
                SELECT id
                FROM namespace_map
                WHERE namespace_map.namespace_id=namespaces.id
            )
        ",
            )
            .unwrap();
        let mut res: Vec<(i64, String)> = Vec::new();
        for orphan in stmt
            .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .unwrap()
        {
            res.push(orphan.unwrap());
        }
        res
    }
}
pub mod init {
    use super::{Connection, Result};
    pub fn recreate_db(path: std::path::PathBuf) -> Connection {
        std::fs::remove_file(&path).unwrap();
        let conn = Connection::open(path).unwrap();
        init_tables(&conn).unwrap();
        conn
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
}
