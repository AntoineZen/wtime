use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::{command, Arg, Command};
use now::DateTimeNow;
use wtime::db::InOut::{In, Out};
use wtime::db::{InOut, Stamp};

/// Datacontainer for application live variables
struct App {
    /// Database connection
    conn: sqlite::Connection,
}

impl App {
    fn new(db_name: String) -> Result<Self> {
        let db_file = std::path::Path::new(&db_name);
        let must_init = !db_file.exists();
        let conn = sqlite::open(db_file)?;

        if must_init {
            Stamp::create(&conn).context("Crate Stamp table")?;
        }
        Ok(Self { conn })
    }

    /// Get total worked time since given date `from`.
    fn get_total_from(self: &Self, from: &DateTime<Utc>) -> Duration {
        let mut total = Duration::zero();
        let mut possible_last: Option<Stamp> = None;

        // Get first stamp after given date, it there is none, return Zero duration
        let first = if let Ok(s) = Stamp::get_after(&self.conn, from) {
            s
        } else {
            return Duration::zero();
        };

        // Iterate on all stamps from there and sum the total
        for stamp in first.iter(&self.conn) {
            if let Some(l) = possible_last {
                if l.in_out == In && stamp.in_out == Out {
                    total = total + (stamp.date - l.date);
                }
            }
            possible_last = Some(stamp);
        }

        // Return total duration
        total
    }

    fn print_resume(self: &Self) {
        // Print worked time
        let now = Utc::now();

        let begin_of_day = now.beginning_of_day();
        let day_total = self.get_total_from(&begin_of_day);
        println!(
            "You worked {} hours, {} minutes and {} seconds today (since {})",
            day_total.num_hours(),
            day_total.num_minutes(),
            day_total.num_seconds(),
            begin_of_day
        );

        // Don't show week total on mondays
        let begin_of_week = now.beginning_of_week();
        if begin_of_day != begin_of_week {
            let week_total = self.get_total_from(&begin_of_week);
            println!(
                "You worked {} hours, {} minutes and {} seconds this week (since {})",
                week_total.num_hours(),
                week_total.num_minutes(),
                week_total.num_seconds(),
                begin_of_week
            );
        }
    }

    fn do_checkin(self: &Self) -> Result<()> {
        // check that we are actually out
        if let Some(last_stamp) = Stamp::last(&self.conn) {
            if last_stamp.in_out == InOut::In {
                return Err(anyhow!(
                    "Already checked in ! (Do you meant to check-out ?)"
                ));
            }
        }

        // Creat teh checking stamp
        let mut stamp = Stamp::check_in();
        stamp.insert(&self.conn).context("Inserting new stamp")?;

        println!("Checked in at {}", stamp.date.format("%H:%M"));
        Ok(())
    }

    fn do_checkout(self: &Self) -> Result<()> {
        // Check that last stamp is check-in
        if let Some(last_stamp) = Stamp::last(&self.conn) {
            if last_stamp.in_out == InOut::Out {
                return Err(anyhow!(
                    "Already checked out ! (Do you meant to check-in ?)"
                ));
            }
        }

        // Create the checkout stamps
        let mut stamp = Stamp::check_out();
        stamp.insert(&self.conn).context("Inserting new stamp")?;

        println!("Checked out at {}", stamp.date.format("%H:%M"));

        if let Some(checkin) = stamp.previous(&self.conn) {
            let work_time = checkin.delta(&stamp);
            println!(
                "You worked {} hours, {} minutes and {} seconds",
                work_time.num_hours(),
                work_time.num_minutes(),
                work_time.num_seconds()
            );
        }

        Ok(())
    }

    fn do_list(self: &Self) -> Result<()> {
        self.print_resume();
        Ok(())
    }
}

fn main() -> Result<()> {
    let matches = command!()
        .arg(
            Arg::new("database")
                .long("db")
                .default_value("prod.sqlite")
                .help("Specify database to use"),
        )
        .subcommand(Command::new("checkin").about("Start counting working time"))
        .subcommand(Command::new("checkout").about("Stop counting work time and display count"))
        .get_matches();

    let db_file_name: String = matches
        .get_one::<String>("database")
        .map(|s| s.to_string())
        .context("Get DB file")?;
    let app = App::new(db_file_name).context("Open DB file")?;

    match matches.subcommand() {
        Some(("checkin", _)) => app.do_checkin(),
        Some(("checkout", _)) => app.do_checkout(),
        None => app.do_list(),
        _ => unreachable!("Should never match none"),
    }
}
