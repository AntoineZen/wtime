use chrono::prelude::*;
use sqlite;
use std::{path::Path, sync::Mutex};
use thiserror::Error;

pub enum InOut {
    In,
    Out,
}

impl InOut {
    pub fn to_string(&self) -> String {
        match self {
            InOut::In => "In".into(),
            InOut::Out => "Out".into(),
        }
    }
}

pub struct Stamp {
    id: u64,
    date: DateTime<Utc>,
    in_out: InOut,
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    SqLiteError {
        #[from]
        source: sqlite::Error,
    },

    #[error("Database not opened!")]
    DbNotOpenError,
}

type StampResult<'a> = std::result::Result<&'a Stamp, DbError>;

static CONN: Mutex<Option<sqlite::Connection>> = Mutex::new(None);

pub fn init(db_path: &Path) -> Result<(), DbError> {
    let c = sqlite::open(db_path)?;
    // It's okay to unwrap, because lock() can only fail if another thread panicked,
    // so we are domed anyway.
    let mut inner = CONN.lock().unwrap();

    *inner = Some(c);

    Ok(())
}

fn do_simple_query(query: String) -> Result<(), DbError> {
    let local_conn = CONN.lock().unwrap();

    if let Some(c) = local_conn.as_ref() {
        c.execute(query)?;
    } else {
        return Err(DbError::DbNotOpenError);
    }

    Ok(())
}

impl Stamp {
    pub fn new(id: u64, date: DateTime<Utc>, in_out: InOut) -> Stamp {
        Stamp {
            id: id,
            date: date,
            in_out: in_out,
        }
    }

    pub fn insert(self: &Stamp) -> StampResult {
        let query = format!(
            "INSERT INTO Stamp ( datetime, in_out) VALUES( {}, {}",
            self.date.to_rfc3339(),
            self.in_out.to_string()
        );
        do_simple_query(query)?;
        Ok(self)
    }

    pub fn update(self: &Stamp) -> StampResult {
        let query = format!(
            "INSERT INTO Stamp ( datetime, in_out) VALUES( {}, {}",
            self.date.to_rfc3339(),
            self.in_out.to_string()
        );
        do_simple_query(query)?;
        Ok(self)
    }

    pub fn get(_id: u64) -> Stamp {
        Stamp::new(0, Utc::now(), InOut::In)
    }

    pub fn delete(self: &Stamp) -> Result<(), DbError> {
        do_simple_query(format!("DELETE FROM Stamp WHERE ID = {};", self.id))
    }

    pub fn create() -> Result<(), DbError> {
        let query = "CREATE TABLE IF NOT EXISTS Stamp (
                id INTEGER NOT NULL PRIMARY KEY ASC,
                datetime TEXT,
                in_out TEXT
            );";

        do_simple_query(query.into())
    }
}
