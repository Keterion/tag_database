use crate::methods::utils::{
    self,
    macros::{handle_unique, insert_ret_id},
};
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
    let tokens = query.split(' ');
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
        if let Some(id) = utils::get_id("tags", &format!("name='{}'", token), conn) {
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
