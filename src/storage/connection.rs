use crate::errors::AppError;
use crate::pragmas::apply_connection_pragmas;
use rusqlite::Connection;
use sqlite_vec::sqlite3_vec_init;
use std::path::Path;

/// Register sqlite-vec GLOBALLY before any connection is opened.
/// Must be called once at program start.
pub fn register_vec_extension() {
    #[allow(clippy::missing_transmute_annotations)]
    unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite3_vec_init as *const (),
        )));
    }
}

pub fn open_rw(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    apply_connection_pragmas(&conn)?;
    Ok(conn)
}

pub fn open_ro(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}
