use chrono::prelude::*;
use sqlite::{self};
use std::{fmt::Formatter, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
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
            _ => Err(ParseInOutError),
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
        #[from]
        source: chrono::format::ParseError,
    },
}

type StampResult<'a> = std::result::Result<&'a Stamp, DbError>;

fn do_simple_query(conn: &sqlite::Connection, query: String) -> Result<(), DbError> {
    conn.execute(query)?;
    Ok(())
}

impl Stamp {
    pub fn new(id: i64, date: DateTime<Utc>, in_out: InOut) -> Self {
        Self { id, date, in_out }
    }

    pub fn check_in() -> Self {
        Self {
            id: 0,
            date: Utc::now(),
            in_out: InOut::In,
        }
    }

    pub fn check_out() -> Self {
        Self {
            id: 0,
            date: Utc::now(),
            in_out: InOut::Out,
        }
    }

    pub fn insert(self: &mut Stamp, conn: &sqlite::Connection) -> StampResult {
        let insert_query = format!(
            "INSERT INTO Stamp ( datetime, in_out) VALUES( \"{}\", \"{}\") ",
            self.date.to_rfc3339(),
            self.in_out
        );

        conn.execute(insert_query)?;

        let mut statement = conn.prepare("SELECT last_insert_rowid()")?;

        match statement.next()? {
            sqlite::State::Row => {
                self.id = statement.read::<i64, _>(0)?;
            }
            sqlite::State::Done => {
                unreachable!("SELECT last_insert_rowid() should not fail");
            }
        }

        Ok(self)
    }

    pub fn update(self: &Stamp, conn: &sqlite::Connection) -> StampResult {
        let query = format!(
            "UPDATE Stamp SET datetime, in_out VALUES ( \"{}\", \"{}\") WHERE id = {}",
            self.date.to_rfc3339(),
            self.in_out,
            self.id
        );
        do_simple_query(conn, query)?;
        Ok(self)
    }

    pub fn previous(self: &Stamp, conn: &sqlite::Connection) -> Option<Stamp> {
        if let Ok(s) = Stamp::get(conn, self.id - 1) {
            Some(s)
        } else {
            None
        }
    }

    pub fn first(conn: &sqlite::Connection) -> Option<Stamp> {
        if let Ok(s) = Stamp::get(conn, 1) {
            Some(s)
        } else {
            None
        }
    }

    pub fn last(conn: &sqlite::Connection) -> Option<Stamp> {
        // Find the last id from the table
        let mut statement = conn.prepare("SELECT max(id) FROM Stamp;").ok()?;
        match statement.next().ok()? {
            sqlite::State::Row => {
                // Once we have it, get the Stamp entry
                let last_id = statement.read::<i64, _>(0).ok()?;

                // TODO: this get cause a dead-lock! by calling
                // twice CONN.lock()
                if let Ok(s) = Self::get(conn, last_id) {
                    Some(s)
                } else {
                    None
                }
            }
            sqlite::State::Done => None,
        }
    }

    pub fn get(conn: &sqlite::Connection, id: i64) -> Result<Stamp, DbError> {
        let mut statement = conn.prepare(format!(
            "SELECT datetime, in_out FROM Stamp WHERE id = {};",
            id
        ))?;

        match statement.next()? {
            sqlite::State::Row => Ok(Self {
                id: id,
                date: DateTime::parse_from_rfc3339(&statement.read::<String, _>("datetime")?)?
                    .into(),
                in_out: InOut::from_str(&statement.read::<String, _>("in_out")?).unwrap(),
            }),
            sqlite::State::Done => Err(DbError::NoSuchEntry),
        }
    }

    pub fn delete(self: &Stamp, conn: &sqlite::Connection) -> Result<(), DbError> {
        do_simple_query(conn, format!("DELETE FROM Stamp WHERE ID = {};", self.id))
    }

    pub fn create(conn: &sqlite::Connection) -> Result<(), DbError> {
        let query = "CREATE TABLE IF NOT EXISTS Stamp (
                id INTEGER NOT NULL PRIMARY KEY ASC,
                datetime TEXT,
                in_out TEXT
            );";

        do_simple_query(conn, query.into())
    }

    pub fn iter<'a>(self: &Stamp, conn: &'a sqlite::Connection) -> StampIterator<'a> {
        StampIterator::new(conn, self.id)
    }

    pub fn drop(conn: &sqlite::Connection) -> Result<(), DbError> {
        let query = "DROP TABLE Stamp";

        do_simple_query(conn, query.into())
    }
}

pub struct StampIterator<'a> {
    current_index: i64,
    db_conn: &'a sqlite::Connection,
}

impl<'a> StampIterator<'a> {
    fn new(conn: &'a sqlite::Connection, start_index: i64) -> Self {
        Self {
            db_conn: conn,
            current_index: start_index,
        }
    }
}

impl<'a> Iterator for StampIterator<'a> {
    type Item = Stamp;

    fn next(&mut self) -> Option<Stamp> {
        if let Ok(s) = Stamp::get(self.db_conn, self.current_index) {
            self.current_index += 1;
            Some(s)
        } else {
            None
        }
    }
}
