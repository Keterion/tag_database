use crate::methods::utils::{self, macros::handle_unique};
use rusqlite::{Connection, Result};
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
