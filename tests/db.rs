use std::path::Path;
use wtime::db::init;
use wtime::db::Stamp;

#[test]
fn test_create() {
    init(Path::new("test_database.sqlite")).unwrap();
    Stamp::create().unwrap();
}

#[test]
fn test_insert() {
    init(Path::new("test_database.sqlite")).unwrap();

    let mut s_in = Stamp::check_in();

    s_in.insert().unwrap();

    assert!(s_in.id != 0);
    assert!(matches!(s_in.in_out, InOut::In));

    let mut s_out = Stamp::check_out();
    s_out.insert().unwrap();

    assert!(s_out.id == s_in.id + 1);
    assert!(matches!(s_out.in_out, InOut::Out));

}
