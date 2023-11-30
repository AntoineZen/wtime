//! Database abstraction layer
//!
//! This module contains object that enable easier manipulation of the
//! objects stored in database.

use chrono::{prelude::*, Duration};
use sqlite::{self};
use std::{fmt::Formatter, str::FromStr};
use thiserror::Error;

/// Stamping direction (checked -in or -out) used for Stamp data structure.
#[derive(Debug, Eq, PartialEq)]
pub enum InOut {
    /// Stamp is a Check-in
    In,
    /// Stamp is a Check-out
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

/// Unit-struct for parsing error on InOut enum
#[derive(Debug, PartialEq, Eq)]
pub struct ParseInOutError;

impl FromStr for InOut {
    type Err = ParseInOutError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "in" => Ok(Self::In),
            "out" => Ok(Self::Out),
            _ => Err(ParseInOutError),
        }
    }
}

/// Represent an "stamp", i.e. a check-out or a check-in time
#[derive(Debug)]
pub struct Stamp {
    /// Stamp unique ID (primary-key in database)
    pub id: i64,
    /// Stamp timestamp, Utc
    pub date: DateTime<Utc>,
    /// Marker if Stamp was check-in or check-out
    ///
    /// See [InOut] enum.
    pub in_out: InOut,
}

/// Type for database related error
#[derive(Error, Debug)]
pub enum DbError {
    /// Error Popageated from sqlite libary
    #[error(transparent)]
    SqLiteError {
        #[from]
        source: sqlite::Error,
    },

    /// Database was not open prior to operation returning this error
    #[error("Database not opened!")]
    DbNotOpenError,

    /// Requested entry does not exist in the database
    #[error("No such entry")]
    NoSuchEntry,

    /// ISO8601 string in database was not parsed correctly.
    #[error(transparent)]
    ParseError {
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
    /// Construct a new struct with exact value
    pub fn new(id: i64, date: DateTime<Utc>, in_out: InOut) -> Self {
        Self { id, date, in_out }
    }

    /// Create a new stamp item, bearing current timestamp and check-IN direction
    pub fn check_in() -> Self {
        Self {
            id: 0,
            date: Utc::now(),
            in_out: InOut::In,
        }
    }

    /// Create a new stamp item, bearing current timestamp and check-OUT direction
    pub fn check_out() -> Self {
        Self {
            id: 0,
            date: Utc::now(),
            in_out: InOut::Out,
        }
    }

    /// Insert (create) stamp into given database
    ///
    /// # Arguments
    /// * `conn` - reference to a open SQLITE database connection
    ///
    /// # Return
    /// Return self if no error.
    pub fn insert(&mut self, conn: &sqlite::Connection) -> StampResult {
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

    /// Update the database with in memory data for given Stamping.
    ///
    /// # Arguments
    /// * `conn` - reference to a open SQLITE database connection
    ///
    /// # Return
    /// Return self if no error.
    pub fn update(self: &Stamp, conn: &sqlite::Connection) -> StampResult {
        let query = format!(
            "UPDATE Stamp SET datetime = \"{}\", in_out = \"{}\" WHERE id = {};",
            self.date.to_rfc3339(),
            self.in_out,
            self.id
        );
        do_simple_query(conn, query)?;
        Ok(self)
    }

    /// Get the stamp previous to this one.
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    ///
    /// # Return
    ///
    /// Return previous one into Some. Return None, if this the first stamp.
    pub fn previous(self: &Stamp, conn: &sqlite::Connection) -> Option<Stamp> {
        Self::get(conn, self.id - 1).ok()
    }

    /// Get the very first stamp
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    ///
    /// # Return
    ///
    /// Return the very fist stamp into Some. Return None, if this there is no stamp at all.
    pub fn first(conn: &sqlite::Connection) -> Option<Stamp> {
        Self::get(conn, 1).ok()
    }

    /// Get the very last stamp
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    ///
    /// # Return
    ///
    /// Return the very last stamp into Some. Return None, if this there is no stamp at all.
    pub fn last(conn: &sqlite::Connection) -> Option<Stamp> {
        // Find the last id from the table
        let mut statement = conn.prepare("SELECT max(id) FROM Stamp;").ok()?;
        match statement.next().ok()? {
            sqlite::State::Row => {
                // Once we have it, get the Stamp entry
                let last_id = statement.read::<i64, _>(0).ok()?;

                Self::get(conn, last_id).ok()
            }
            sqlite::State::Done => None,
        }
    }

    /// Get the stamp with given ID.
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    /// * `id` - Stamp ID to get
    ///
    /// # Returns
    ///
    /// Stamp object with the given id, or [DbError::NoSuchEntry] error
    pub fn get(conn: &sqlite::Connection, id: i64) -> Result<Stamp, DbError> {
        let mut statement = conn.prepare(format!(
            "SELECT datetime, in_out FROM Stamp WHERE id = {};",
            id
        ))?;

        match statement.next()? {
            sqlite::State::Row => Ok(Self {
                id,
                date: DateTime::parse_from_rfc3339(&statement.read::<String, _>("datetime")?)?
                    .into(),
                in_out: InOut::from_str(&statement.read::<String, _>("in_out")?).unwrap(),
            }),
            sqlite::State::Done => Err(DbError::NoSuchEntry),
        }
    }

    /// Get the stamp the very first Stamp after the given timestamp
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    /// * `initial_date` - Date/Time to look after
    ///
    /// # Returns
    ///
    /// First stamp after given date or [DbError::NoSuchEntry] error
    pub fn get_after(
        conn: &sqlite::Connection,
        initial_date: &DateTime<Utc>,
    ) -> Result<Self, DbError> {
        let mut statement = conn.prepare(format!(
            "SELECT id, datetime, in_out FROM Stamp WHERE datetime >= '{}';",
            initial_date.to_rfc3339()
        ))?;

        match statement.next()? {
            sqlite::State::Row => Ok(Self {
                id: statement.read::<i64, _>("id")?,
                date: DateTime::parse_from_rfc3339(&statement.read::<String, _>("datetime")?)?
                    .into(),
                in_out: InOut::from_str(&statement.read::<String, _>("in_out")?).unwrap(),
            }),
            sqlite::State::Done => Err(DbError::NoSuchEntry),
        }
    }

    /// Delete current stamp from database
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    pub fn delete(self: &Stamp, conn: &sqlite::Connection) -> Result<(), DbError> {
        do_simple_query(conn, format!("DELETE FROM Stamp WHERE ID = {};", self.id))
    }

    /// Create database table (Static method)
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    pub fn create(conn: &sqlite::Connection) -> Result<(), DbError> {
        let query = "CREATE TABLE IF NOT EXISTS Stamp (
                id INTEGER NOT NULL PRIMARY KEY ASC,
                datetime TEXT,
                in_out TEXT
            );";

        do_simple_query(conn, query.into())
    }

    /// Get an iterator, staring from current stamp
    ///
    /// # Arguments
    ///
    /// * `conn` - reference to a open SQLITE database connection
    pub fn iter<'a>(&self, conn: &'a sqlite::Connection) -> StampIterator<'a> {
        StampIterator::new(conn, self.id)
    }

    /// Delete table as a whole
    ///
    /// This is mainly used for testing purpose
    pub fn drop(conn: &sqlite::Connection) -> Result<(), DbError> {
        let query = "DROP TABLE Stamp";

        do_simple_query(conn, query.into())
    }

    /// Compute the time delta from this stamp to another.
    ///
    /// * `other` - Stamp to compute time delta from
    pub fn delta(self: &Stamp, other: &Stamp) -> Duration {
        if other.date > self.date {
            other.date - self.date
        } else {
            self.date - other.date
        }
    }
}

/// Iterator over stamps objects
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

#[cfg(test)]
mod test {
    use super::{DbError, InOut, ParseInOutError, Stamp};
    use chrono::{DateTime, Duration, Utc};
    use sqlite;
    use std::{fs, path::Path, str::FromStr};

    const DB_FILE: &str = "unit-test.sqlite";

    fn open_db(file_name: &str) -> sqlite::Connection {
        sqlite::open(Path::new(file_name)).unwrap()
    }

    struct TestFixture {
        c: sqlite::Connection,
    }

    impl TestFixture {
        fn init() -> Self {
            let conn = open_db(DB_FILE);
            Stamp::create(&conn).unwrap();
            TestFixture { c: conn }
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            fs::remove_file(DB_FILE).unwrap();
        }
    }

    #[test]
    fn inout() {
        assert_eq!(InOut::from_str("In"), Ok(InOut::In));
        assert_eq!(InOut::from_str("in"), Ok(InOut::In));

        assert_eq!(InOut::from_str("   In "), Ok(InOut::In));
        assert_eq!(InOut::from_str(" in   "), Ok(InOut::In));

        assert_eq!(InOut::from_str("OUT"), Ok(InOut::Out));
        assert_eq!(InOut::from_str("out"), Ok(InOut::Out));

        assert_eq!(InOut::from_str(" oUt "), Ok(InOut::Out));
        assert_eq!(InOut::from_str(" out  "), Ok(InOut::Out));

        assert_eq!(InOut::from_str(" ou  "), Err(ParseInOutError));
        assert_eq!(InOut::from_str(""), Err(ParseInOutError));
        assert_eq!(InOut::from_str("i"), Err(ParseInOutError));
    }

    #[test]
    fn insert() {
        let f = TestFixture::init();

        Stamp::create(&f.c).unwrap();

        let mut s_in = Stamp::check_in();

        s_in.insert(&f.c).unwrap();

        assert_ne!(s_in.id, 0);
        assert!(matches!(s_in.in_out, InOut::In));

        let mut s_out = Stamp::check_out();
        s_out.insert(&f.c).unwrap();

        assert_eq!(s_out.id, s_in.id + 1);
        assert!(matches!(s_out.in_out, InOut::Out));
    }

    #[test]
    fn get() {
        let f = TestFixture::init();

        // Get a non-existent stamp
        assert!(matches!(Stamp::get(&f.c, 1), Err(DbError::NoSuchEntry)));

        // Create a stamp
        Stamp::check_in().insert(&f.c).unwrap();

        // Check we can get it now
        assert!(matches!(Stamp::get(&f.c, 1), Ok(x) if x.id == 1));
    }

    #[test]
    fn update() {
        let f = TestFixture::init();

        // Create a stamp
        let mut s = Stamp::check_in();
        s.insert(&f.c).unwrap();

        s.in_out = InOut::Out;

        let result = s.update(&f.c);

        println!("{:?}", &result);

        assert!(result.is_ok());
    }

    #[test]
    fn first_getter() {
        let f = TestFixture::init();

        // Get a non-existent stamp
        assert!(matches!(Stamp::first(&f.c), None));

        // Create a stamp
        let mut first = Stamp::check_in();
        first.insert(&f.c).unwrap();

        // Create a second stamp
        Stamp::check_in().insert(&f.c).unwrap();

        assert!(matches!(Stamp::first(&f.c), Some( x) if x.id == first.id));
    }

    #[test]
    fn iterator() {
        let f = TestFixture::init();

        // Create a stamp
        let mut last_inserted = None;
        for _ in 0..10 {
            Stamp::check_in().insert(&f.c).unwrap();
            let mut s = Stamp::check_out();
            s.insert(&f.c).unwrap();
            last_inserted = Some(s);
        }

        let first_stamp = Stamp::first(&f.c).unwrap();

        let mut last_iterated: Option<Stamp> = None;
        let mut count = 0;
        for s in first_stamp.iter(&f.c) {
            last_iterated = Some(s);
            count += 1;
        }

        assert_eq!(last_iterated.unwrap().id, last_inserted.unwrap().id);
        assert_eq!(20, count);
    }

    #[test]
    fn last_getter() {
        let f = TestFixture::init();

        // Check that last() return None on an empty DB
        let res = Stamp::last(&f.c);
        assert!(matches!(res, None));

        // Create some stamp
        let mut last_inserted = None;
        for _ in 0..10 {
            Stamp::check_in().insert(&f.c).unwrap();
            let mut s = Stamp::check_out();
            s.insert(&f.c).unwrap();
            last_inserted = Some(s);
        }

        assert!(matches!( Stamp::last(&f.c), Some(x) if x.id == last_inserted.unwrap().id));
    }

    #[test]
    fn delta() {
        let t1 = Stamp::new(
            0,
            DateTime::<Utc>::from_str("2020-01-01T08:00:00Z").unwrap(),
            InOut::In,
        );
        let t2 = Stamp::new(
            0,
            DateTime::<Utc>::from_str("2020-01-01T10:15:20Z").unwrap(),
            InOut::In,
        );

        let exp_delta = Duration::hours(2) + Duration::minutes(15) + Duration::seconds(20);
        assert_eq!(t2.delta(&t1), exp_delta);
        assert_eq!(t1.delta(&t2), exp_delta);
    }

    #[test]
    fn get_after() {
        let f = TestFixture::init();

        let mut t1 = Stamp::new(
            0,
            DateTime::<Utc>::from_str("2020-01-01T08:00:00Z").unwrap(),
            InOut::In,
        );

        t1.insert(&f.c).unwrap();

        let start_date = DateTime::<Utc>::from_str("2020-01-01T00:00:00Z").unwrap();
        let t1_retrived = Stamp::get_after(&f.c, &start_date).unwrap();

        assert_eq!(t1.id, t1_retrived.id);

        let does_not_exists = Stamp::get_after(
            &f.c,
            &DateTime::<Utc>::from_str("2020-01-01T10:00:00Z").unwrap(),
        );
        assert!(matches!(does_not_exists, Err(DbError::NoSuchEntry)));
    }
}
