use std::path::Path;
use wtime::db::{InOut, init, Stamp, DbError};

#[test]
fn test_create() {
    init(Path::new("test_database.sqlite")).unwrap();
    Stamp::create().unwrap();

    Stamp::drop().unwrap();
}

#[test]
fn test_insert() {
    init(Path::new("test_database.sqlite")).unwrap();
    Stamp::create().unwrap();

    let mut s_in = Stamp::check_in();

    s_in.insert().unwrap();

    assert!(s_in.id != 0);
    assert!(matches!(s_in.in_out, InOut::In));

    let mut s_out = Stamp::check_out();
    s_out.insert().unwrap();

    assert!(s_out.id == s_in.id + 1);
    assert!(matches!(s_out.in_out, InOut::Out));

    Stamp::drop().unwrap();
}

#[test]
fn test_get() {
    init(Path::new("test_database.sqlite")).unwrap();
    Stamp::create().unwrap();

    // Get a non-existent stamp
    let res = Stamp::get(1);
    assert!(matches!(res, Err(DbError::NoSuchEntry)));

    // Create a stamp
    Stamp::check_in().insert().unwrap();

    // Check we can get it now
    let s = Stamp::get(1).unwrap();
    assert!(s.id == 1);
    
    Stamp::drop().unwrap();
#[test]
fn test_first() {
    init(Path::new("test_database.sqlite")).unwrap();
    Stamp::create().unwrap();

    // Get a non-existent stamp
    let res = Stamp::first();
    assert!(matches!(res, None));

    // Create a stamp
    let mut s = Stamp::check_in();
    s.insert().unwrap();

    let res = Stamp::first();
    assert!(matches!(&res, Some(ref s)));
    let fisrt_s = res.unwrap();

    assert!(s.id == fisrt_s.id);

    
    Stamp::drop().unwrap();
}
}