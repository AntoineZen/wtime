use crate::db::InOut::{In, Out};
use crate::db::{InOut, Stamp};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use now::DateTimeNow;

/// Datacontainer for application live variables
pub struct App {
    /// Database connection
    conn: sqlite::Connection,
}

impl App {
    pub fn new(db_name: String) -> Result<Self> {
        let db_file = std::path::Path::new(&db_name);
        let must_init = !db_file.exists();
        let conn = sqlite::open(db_file)?;

        if must_init {
            Stamp::create(&conn).context("Crate Stamp table")?;
        }
        Ok(Self { conn })
    }

    /// Get total worked time since given date `from`.
    fn get_total_from(&self, from: &DateTime<Utc>) -> Duration {
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

    fn print_resume(&self) {
        // Print worked time
        let now = Utc::now();

        let begin_of_day = now.beginning_of_day();
        let day_total = self.get_total_from(&begin_of_day);
        println!(
            "You worked {} hours, {} minutes and {} seconds today (since {})",
            day_total.num_hours(),
            day_total.num_minutes() % 60,
            day_total.num_seconds() % 60,
            begin_of_day
        );

        // Don't show week total on mondays
        let begin_of_week = now.beginning_of_week();
            let week_total = self.get_total_from(&begin_of_week);
        if week_total != day_total {
            println!(
                "You worked {} hours, {} minutes and {} seconds this week (since {})",
                week_total.num_hours(),
                week_total.num_minutes() % 60,
                week_total.num_seconds() % 60,
                begin_of_week
            );
        }
    }

    pub fn do_checkin(&self) -> Result<()> {
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

    pub fn do_checkout(&self) -> Result<()> {
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
                work_time.num_minutes() % 60,
                work_time.num_seconds() & 60
            );
        }

        Ok(())
    }

    pub fn do_list(&self) -> Result<()> {
        self.print_resume();
        Ok(())
    }
}
