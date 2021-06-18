use rusqlite;
use rusqlite::{ToSql, Connection};
use std::path::Path;


pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let db = Connection::open(path)?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS transaction_list (\
                id       INTEGER PRIMARY KEY AUTOINCREMENT, \
                tx_h     INTEGER NOT NULL, \
                tx_n     INTEGER NOT NULL, \
                tx_id    BLOB    NOT NULL UNIQUE, \
                tx_bytes BLOB    NOT NULL)",[])?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS outputs_list (\
                id      INTEGER PRIMARY KEY, \
                creator INTEGER NOT NULL,   \
                out_n   INTEGER NOT NULL,   \
                consts  BLOB    NOT NULL,   \
                n_consts INTEGER NOT NULL,  \
                script  BLOB    NOT NULL,   \
                script_hash BLOB NOT NULL,  \
                is_36b      BOOLEAN NOT NULL, \
                UNIQUE(creator,out_n))",[])?;
    Ok(db)
}


/// rusqlite uses somewhat weird API. Parameters are passed as references
/// which makes it impossible to bundle them as prt of structure. To circumvent
/// this contraption is created.
pub struct Query<'a> {
    stmt: rusqlite::Statement<'a>,
    param: Vec<Box<dyn rusqlite::ToSql>>,
}

impl<'a> Query<'a> {
    /// Query without parameters
    pub fn new_(conn: &'a rusqlite::Connection, sql: &str) -> rusqlite::Result<Query<'a>> {
        let stmt = conn.prepare(&sql)?;
        Ok(Query {
            stmt,
            param: Vec::new(),
        })
    }

    /// Query with parameters
    pub fn new(
        conn: &'a rusqlite::Connection,
        sql: &str,
        param: Vec<Box<dyn rusqlite::ToSql>>,
    ) -> rusqlite::Result<Query<'a>> {
        let stmt = conn.prepare(&sql)?;
        Ok(Query { stmt, param })
    }

    /// Run query
    pub fn run(&mut self) -> rusqlite::Result<rusqlite::Rows> {
        let params = self.param.iter().map(|x| &**x).collect::<Vec<_>>();
        let params: &[&dyn ToSql] = params.as_ref();
        self.stmt.query(params)
    }
}