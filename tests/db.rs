use sqlite;
use std::path::Path;
use wtime::db::{DbError, InOut, Stamp};

fn open_db(file_name: &str) -> sqlite::Connection {
    sqlite::open(Path::new(file_name)).unwrap()
}

#[test]
fn test_create() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    Stamp::drop(&c).unwrap();
}

#[test]
fn test_insert() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    let mut s_in = Stamp::check_in();

    s_in.insert(&c).unwrap();

    assert!(s_in.id != 0);
    assert!(matches!(s_in.in_out, InOut::In));

    let mut s_out = Stamp::check_out();
    s_out.insert(&c).unwrap();

    assert!(s_out.id == s_in.id + 1);
    assert!(matches!(s_out.in_out, InOut::Out));

    Stamp::drop(&c).unwrap();
}

#[test]
fn test_get() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    // Get a non-existent stamp
    let res = Stamp::get(&c, 1);
    assert!(matches!(res, Err(DbError::NoSuchEntry)));

    // Create a stamp
    Stamp::check_in().insert(&c).unwrap();

    // Check we can get it now
    let s = Stamp::get(&c, 1).unwrap();
    assert!(s.id == 1);

    Stamp::drop(&c).unwrap();
}

#[test]
fn test_first() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    // Get a non-existent stamp
    let res = Stamp::first(&c);
    assert!(matches!(res, None));

    // Create a stamp
    let mut first = Stamp::check_in();
    first.insert(&c).unwrap();

    // Create a sectond stamp
    let mut second = Stamp::check_in();
    second.insert(&c).unwrap();

    let res = Stamp::first(&c);
    assert!(matches!(&res, Some(ref x) if x.id == first.id));

    Stamp::drop(&c).unwrap();
}

#[test]
fn test_iter() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    // Create a stamp
    let mut last_inserted = None;
    for _ in 0..10 {
        Stamp::check_in().insert(&c).unwrap();
        let mut s = Stamp::check_out();
        s.insert(&c).unwrap();
        last_inserted = Some(s);
    }
    let last_inserted = last_inserted.unwrap();

    let first_stamp = Stamp::first(&c).unwrap();

    let mut last_iterated: Option<Stamp> = None;
    for s in first_stamp.iter(&c) {
        last_iterated = Some(s);
    }

    let last_iterated = last_iterated.unwrap();
    assert!(last_iterated.id == last_inserted.id);

    Stamp::drop(&c).unwrap();
}

#[test]
fn test_last() {
    let c = open_db("test_database.sqlite");
    Stamp::create(&c).unwrap();

    let res = Stamp::last(&c);
    assert!(matches!(res, None));

    // Create some stamp
    let mut last_inserted = None;
    for _ in 0..10 {
        Stamp::check_in().insert(&c).unwrap();
        let mut s = Stamp::check_out();
        s.insert(&c).unwrap();
        last_inserted = Some(s);
    }

    let last_direct = Stamp::last(&c);
    assert!(matches!(&last_direct, Some(x) if x.id == last_inserted.unwrap().id));

    Stamp::drop(&c).unwrap();
}
