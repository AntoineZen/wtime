use std::path::Path;
use wtime::db::init;
use wtime::db::Stamp;

#[test]
fn test_create() {
    init(Path::new("test_database.sqlite"));

    Stamp::create().unwrap();
}
