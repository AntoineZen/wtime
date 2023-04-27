use chrono::prelude::*;
use sqlite;
use std::{str::FromStr, fmt::Formatter, path::Path, sync::Mutex};
use thiserror::Error;

pub enum InOut {
    In,
    Out,
}

impl std::fmt::Display for InOut {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InOut::In => write!(f, "In"),
            InOut::Out => write!(f, "Out"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseInOutError;

impl std::str::FromStr for InOut {

    type Err = ParseInOutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            _ => Err(ParseInOutError)
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
    #[error("No such entry")]
    NoSuchEntry,

    #[error(transparent)]
    PraseError {
        #[from] source: chrono::format::ParseError,
    },
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

    pub fn get(id: i64) -> Result<Stamp,DbError>  {
        let local_conn = CONN.lock().unwrap();
        if let Some(c) = local_conn.as_ref() {

            let mut statement = c.prepare(
                format!("SELECT datetime, in_out FROM Stamp WHERE id = {};", id)
            )?;

            match statement.next()? {
                sqlite::State::Row => {
                    Ok(Self {
                        id: id,
                        date: DateTime::parse_from_rfc3339(&statement.read::<String, _>("datetime")?)?.into(),
                        in_out: InOut::from_str(&statement.read::<String, _>("in_out")?).unwrap()
                    })
                }
                sqlite::State::Done => {
                    Err(DbError::NoSuchEntry)
                }
            }
        } else {
            Err(DbError::DbNotOpenError)
        }
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