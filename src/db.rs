use chrono::prelude::*;
use sqlite;
use std::{fmt::Display, fmt::Formatter, path::Path, sync::Mutex};
use thiserror::Error;

pub enum InOut {
    In,
    Out,
}

impl Display for InOut {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InOut::In => write!(f, "In"),
            InOut::Out => write!(f, "Out"),
        }
    }
}

pub struct Stamp {
    pub id: i64,
    pub date: DateTime<Utc>,
    pub in_out: InOut,
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
    pub fn new(id: i64, date: DateTime<Utc>, in_out: InOut) -> Stamp {
        Stamp { id, date, in_out }
    }

    pub fn check_in() -> Stamp {
        Stamp {
            id: 0,
            date: Utc::now(),
            in_out: InOut::In,
        }
    }

    pub fn check_out() -> Stamp {
        Stamp {
            id: 0,
            date: Utc::now(),
            in_out: InOut::Out,
        }
    }

    pub fn insert(self: &mut Stamp) -> StampResult {
        let local_conn = CONN.lock().unwrap();
        let insert_query = format!(
            "INSERT INTO Stamp ( datetime, in_out) VALUES( \"{}\", \"{}\") ",
            self.date.to_rfc3339(),
            self.in_out
        );

        if let Some(c) = local_conn.as_ref() {
            c.execute(insert_query)?;

            let mut statement = c.prepare("SELECT last_insert_rowid()")?;

            match statement.next()? {
                sqlite::State::Row => {
                    self.id = statement.read::<i64, _>(0)?;
                }
                sqlite::State::Done => {
                    unreachable!("SELECT last_insert_rowid() should not fail");
                }
            }
        } else {
            return Err(DbError::DbNotOpenError);
        }

        Ok(self)
    }

    pub fn update(self: &Stamp) -> StampResult {
        let query = format!(
            "UPDATE Stamp SET datetime, in_out VALUES ( \"{}\", \"{}\") WHERE id = {}",
            self.date.to_rfc3339(),
            self.in_out,
            self.id
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
