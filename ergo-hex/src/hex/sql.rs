use rusqlite;
use rusqlite::ToSql;

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