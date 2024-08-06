use crate::methods::{
    subtags, tags,
    utils::{
        self,
        macros::{handle_unique, insert_ret_id},
    },
};
use rusqlite::{Connection, Result};
/// adds tag to image, if create is true nonexistent tags will get created
pub fn add_tag_to_img(tag: &str, img: i64, create: bool, conn: &Connection) -> Option<()> {
    let mut insert = conn
        .prepare("INSERT INTO tag_map(img_id, tag_id) VALUES (?1, ?2)")
        .unwrap();
    // primary tag
    let primary_id = match utils::get_id("tags", &format!("name='{}'", tag), conn) {
        Some(id) => id,
        None => {
            if create {
                tags::add_tag(tag, conn).unwrap()
            } else {
                eprintln!("Tag {} doesn't exist and shouldn't be created", tag);
                return None;
            }
        }
    }; // if there was no id found, panic
    handle_unique!(insert.execute([img, primary_id])).unwrap();

    // parent tags
    let parents = subtags::get_parents(primary_id, conn).unwrap_or({
        eprintln!("Failed to get parents for tag {}", primary_id);
        vec![]
    });
    for parent in parents {
        insert.execute([img, parent]).unwrap();
    }
    Some(())
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
    if search_term.is_empty() {
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
