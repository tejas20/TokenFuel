use std::ffi::{CStr, CString, c_char, c_int, c_uchar};
use std::path::Path;
use std::ptr;

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_DONE: c_int = 101;
const SQLITE_BUSY: c_int = 5;
const SQLITE_LOCKED: c_int = 6;
const SQLITE_OPEN_READONLY: c_int = 0x0000_0001;
const SQLITE_OPEN_READWRITE: c_int = 0x0000_0002;
const SQLITE_OPEN_CREATE: c_int = 0x0000_0004;
const SQLITE_OPEN_URI: c_int = 0x0000_0040;

#[repr(C)]
struct Sqlite3 {
    _private: [u8; 0],
}

#[repr(C)]
struct Sqlite3Stmt {
    _private: [u8; 0],
}

#[repr(C)]
struct Sqlite3Backup {
    _private: [u8; 0],
}

#[cfg(windows)]
#[link(name = "winsqlite3", kind = "raw-dylib")]
unsafe extern "C" {
    fn sqlite3_open_v2(
        filename: *const c_char,
        database: *mut *mut Sqlite3,
        flags: c_int,
        vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(database: *mut Sqlite3) -> c_int;
    fn sqlite3_errmsg(database: *mut Sqlite3) -> *const c_char;
    fn sqlite3_busy_timeout(database: *mut Sqlite3, milliseconds: c_int) -> c_int;
    fn sqlite3_prepare_v2(
        database: *mut Sqlite3,
        sql: *const c_char,
        sql_bytes: c_int,
        statement: *mut *mut Sqlite3Stmt,
        tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_bind_text(
        statement: *mut Sqlite3Stmt,
        index: c_int,
        value: *const c_char,
        value_bytes: c_int,
        destructor: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
    ) -> c_int;
    fn sqlite3_step(statement: *mut Sqlite3Stmt) -> c_int;
    fn sqlite3_column_text(statement: *mut Sqlite3Stmt, column: c_int) -> *const c_uchar;
    fn sqlite3_column_bytes(statement: *mut Sqlite3Stmt, column: c_int) -> c_int;
    fn sqlite3_finalize(statement: *mut Sqlite3Stmt) -> c_int;
    fn sqlite3_backup_init(
        dest_db: *mut Sqlite3,
        dest_name: *const c_char,
        source_db: *mut Sqlite3,
        source_name: *const c_char,
    ) -> *mut Sqlite3Backup;
    fn sqlite3_backup_step(backup: *mut Sqlite3Backup, n_page: c_int) -> c_int;
    fn sqlite3_backup_finish(backup: *mut Sqlite3Backup) -> c_int;
}

struct Connection {
    raw: *mut Sqlite3,
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                sqlite3_close(self.raw);
            }
        }
    }
}

struct Statement {
    raw: *mut Sqlite3Stmt,
}

impl Drop for Statement {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                sqlite3_finalize(self.raw);
            }
        }
    }
}

impl Connection {
    #[cfg(windows)]
    fn open_read_only(path: &Path) -> Result<Self, String> {
        let filename = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|_| "SQLite path contains null byte.".to_string())?;
        let mut raw = ptr::null_mut();
        let rc = unsafe {
            sqlite3_open_v2(
                filename.as_ptr(),
                &mut raw,
                SQLITE_OPEN_READONLY | SQLITE_OPEN_URI,
                ptr::null(),
            )
        };
        if rc != SQLITE_OK || raw.is_null() {
            if !raw.is_null() {
                unsafe {
                    sqlite3_close(raw);
                }
            }
            return Err(format!("Could not open SQLite database (code {rc})"));
        }
        unsafe {
            sqlite3_busy_timeout(raw, 1000);
        }
        Ok(Self { raw })
    }

    #[cfg(windows)]
    fn open_in_memory() -> Result<Self, String> {
        let filename = CString::new(":memory:").unwrap();
        let mut raw = ptr::null_mut();
        let rc = unsafe {
            sqlite3_open_v2(
                filename.as_ptr(),
                &mut raw,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
                ptr::null(),
            )
        };
        if rc != SQLITE_OK || raw.is_null() {
            if !raw.is_null() {
                unsafe {
                    sqlite3_close(raw);
                }
            }
            return Err("Could not open in-memory SQLite database.".into());
        }
        Ok(Self { raw })
    }

    #[cfg(windows)]
    fn backup_from(&self, source: &Connection) -> Result<(), String> {
        let main = CString::new("main").unwrap();
        let backup =
            unsafe { sqlite3_backup_init(self.raw, main.as_ptr(), source.raw, main.as_ptr()) };
        if backup.is_null() {
            return Err("SQLite backup_init failed.".into());
        }
        let rc = unsafe { sqlite3_backup_step(backup, -1) };
        let finish_rc = unsafe { sqlite3_backup_finish(backup) };
        if rc != SQLITE_DONE && rc != SQLITE_OK {
            return Err(format!(
                "SQLite backup_step failed (code {rc}, finish {finish_rc})"
            ));
        }
        Ok(())
    }

    #[cfg(windows)]
    fn prepare(&self, sql: &str) -> Result<Statement, String> {
        let c_sql = CString::new(sql).map_err(|_| "SQL contains null byte.")?;
        let mut raw = ptr::null_mut();
        let rc =
            unsafe { sqlite3_prepare_v2(self.raw, c_sql.as_ptr(), -1, &mut raw, ptr::null_mut()) };
        if rc != SQLITE_OK || raw.is_null() {
            let msg = unsafe {
                CStr::from_ptr(sqlite3_errmsg(self.raw))
                    .to_string_lossy()
                    .to_string()
            };
            return Err(format!("SQLite prepare error: {msg}"));
        }
        Ok(Statement { raw })
    }
}

/// Query an optional text value safely from a local SQLite database (such as Cursor's state.vscdb).
/// Handles WAL mode and database concurrency using a busy timeout and in-memory backup fallback.
#[cfg(windows)]
pub fn query_optional_text(path: &Path, sql: &str, param: &str) -> Result<Option<String>, String> {
    if !path.is_file() {
        return Ok(None);
    }

    let conn = Connection::open_read_only(path)?;
    match query_single_param(&conn, sql, param) {
        Ok(result) => Ok(result),
        Err(e) if e.contains("busy") || e.contains("locked") => {
            // Database is locked by active Cursor process. Use SQLite backup API to snapshot safely.
            let mem_conn = Connection::open_in_memory()?;
            mem_conn.backup_from(&conn)?;
            query_single_param(&mem_conn, sql, param)
        }
        Err(e) => Err(e),
    }
}

#[cfg(not(windows))]
pub fn query_optional_text(
    _path: &Path,
    _sql: &str,
    _param: &str,
) -> Result<Option<String>, String> {
    Err("winsqlite is supported only on Windows.".into())
}

#[cfg(windows)]
fn query_single_param(conn: &Connection, sql: &str, param: &str) -> Result<Option<String>, String> {
    let stmt = conn.prepare(sql)?;
    let c_param = CString::new(param).map_err(|_| "Param contains null byte.")?;
    unsafe {
        let rc = sqlite3_bind_text(stmt.raw, 1, c_param.as_ptr(), -1, None);
        if rc != SQLITE_OK {
            return Err("SQLite bind failed.".into());
        }
        let step_rc = sqlite3_step(stmt.raw);
        if step_rc == SQLITE_ROW {
            let text_ptr = sqlite3_column_text(stmt.raw, 0);
            let bytes = sqlite3_column_bytes(stmt.raw, 0);
            if text_ptr.is_null() || bytes <= 0 {
                return Ok(None);
            }
            let slice = std::slice::from_raw_parts(text_ptr, bytes as usize);
            let s = String::from_utf8_lossy(slice).to_string();
            Ok(Some(s))
        } else if step_rc == SQLITE_DONE {
            Ok(None)
        } else if step_rc == SQLITE_BUSY || step_rc == SQLITE_LOCKED {
            Err("SQLite database busy or locked.".into())
        } else {
            Err(format!("SQLite step error: code {step_rc}"))
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn windows_sqlite_reader_binds_keys_and_cannot_write_to_the_source() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("state.vscdb");
        let filename = CString::new(path.to_str().unwrap()).unwrap();
        let mut raw = ptr::null_mut();
        assert_eq!(
            unsafe {
                sqlite3_open_v2(
                    filename.as_ptr(),
                    &mut raw,
                    SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
                    ptr::null(),
                )
            },
            SQLITE_OK
        );
        {
            let connection = Connection { raw };
            for sql in [
                "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT)",
                "INSERT INTO ItemTable VALUES ('cursorAuth/accessToken', 'synthetic-session')",
            ] {
                let statement = connection.prepare(sql).unwrap();
                assert_eq!(unsafe { sqlite3_step(statement.raw) }, SQLITE_DONE);
            }
        }
        let before = std::fs::read(&path).unwrap();
        let sql = "SELECT value FROM ItemTable WHERE key = ?";
        assert_eq!(
            query_optional_text(&path, sql, "cursorAuth/accessToken")
                .unwrap()
                .as_deref(),
            Some("synthetic-session")
        );
        assert_eq!(query_optional_text(&path, sql, "missing").unwrap(), None);
        assert_eq!(
            query_optional_text(&path, sql, "' OR 1=1 --").unwrap(),
            None
        );
        assert!(
            query_optional_text(
                &path,
                "DELETE FROM ItemTable WHERE key = ?",
                "cursorAuth/accessToken"
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn in_memory_backup_keeps_quota_session_lookup_available() {
        let source = Connection::open_in_memory().unwrap();
        let statement = source
            .prepare("CREATE TABLE ItemTable (key TEXT, value TEXT)")
            .unwrap();
        assert_eq!(unsafe { sqlite3_step(statement.raw) }, SQLITE_DONE);
        drop(statement);
        let statement = source
            .prepare("INSERT INTO ItemTable VALUES ('session', 'synthetic-session')")
            .unwrap();
        assert_eq!(unsafe { sqlite3_step(statement.raw) }, SQLITE_DONE);
        drop(statement);
        let snapshot = Connection::open_in_memory().unwrap();
        snapshot.backup_from(&source).unwrap();
        assert_eq!(
            query_single_param(
                &snapshot,
                "SELECT value FROM ItemTable WHERE key = ?",
                "session"
            )
            .unwrap()
            .as_deref(),
            Some("synthetic-session")
        );
    }
}
