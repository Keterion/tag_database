use crate::methods::utils::macros::handle_unique;
use rusqlite::{Connection, Result};
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
