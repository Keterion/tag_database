use rusqlite::{Connection, Result};
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
pub mod macros {
    macro_rules! handle_unique {
        ($command:expr) => {
            match $command {
                Ok(_) => Ok(1),
                Err(err) => match err.sqlite_error_code() {
                    Some(rusqlite::ErrorCode::ConstraintViolation) => {
                        eprintln!(
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

    pub(crate) use handle_unique;
    pub(crate) use insert_ret_id;
}
