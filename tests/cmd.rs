use assert_cmd::*;
use std::fs;

const TEST_FILE: &str = "test.sqlite";
fn teardown() {
    fs::remove_file(TEST_FILE).unwrap();
}
#[test]
fn test_default() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .assert()
        .success();

    teardown();
}

#[test]
fn test_checkin_checkout() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkin")
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkout")
        .assert()
        .success();

    teardown();
}

#[test]
fn test_double_checkin() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkin")
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkin")
        .assert()
        .failure();

    teardown();
}

#[test]
fn test_first_checkout() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkout")
        .assert()
        .success();

    teardown();
}

#[test]
fn test_double_checkout() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkin")
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkout")
        .assert()
        .success();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("checkout")
        .assert()
        .failure();

    teardown();
}
