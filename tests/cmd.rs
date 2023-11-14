use assert_cmd::*;

#[test]
fn test_default() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("--db")
        .arg("test.sqlite")
        .assert()
        .success();
}
