use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
    let matches = command!()
        .subcommand(Command::new("checkin").about("Start counting working time"))
        .subcommand(Command::new("checkout").about("Stop counting work time and display count"))
        .get_matches();

    match matches.subcommand() {
        Some(("checkin", _)) => println!("Doing checking"),
        Some(("checkout", _)) => println!("Doing checkout"),
        None => println!("Doing default"),
        _ => unreachable!("Should never match none"),
    }
}
