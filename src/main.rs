mod cli;
mod crud;
mod display;
mod models;

use models::CreateRecord;
use std::error::Error;

use crate::models::Record;
use chrono::{Local, NaiveDateTime, TimeDelta};
use clap::Parser;
use cli::{Cli, Cmds};
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{Sqlite, SqlitePool};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

#[cfg(debug_assertions)]
const MODE: &str = "debug";

#[cfg(not(debug_assertions))]
const MODE: &str = "release";

async fn ensure_db(url: &str) {
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        println!("Creating database at {}", url);
        match Sqlite::create_database(url).await {
            Ok(_) => println!("Database created successfully."),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        #[cfg(debug_assertions)]
        {
            println!("Using database at {}", url);
        }
    }
}

async fn setup(url: &str) -> Result<SqlitePool, sqlx::Error> {
    // make the db if it doesn't exist
    ensure_db(url).await;
    // run migrations to get db up to date
    let pool = SqlitePool::connect(url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

async fn find_active_session(pool: &SqlitePool) -> Result<Option<Record>, Box<dyn Error>> {
    let q: Vec<Record> = sqlx::query_as(
        r#"
        SELECT * FROM work_hours
        WHERE clock_out IS NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    if q.is_empty() {
        Ok(None)
    } else if q.len() == 1 {
        Ok(Some(q[0].clone()))
    } else {
        Err("Multiple active sessions found".into())
    }
}

async fn clock_in(
    job_name: String,
    time: NaiveDateTime,
    pool: &SqlitePool,
) -> Result<Record, sqlx::Error> {
    let create_record = CreateRecord {
        job_name: job_name.clone(),
        clock_in: time,
        ..Default::default()
    };
    let record = crud::create(&create_record, pool).await?;
    Ok(record)
}

async fn clock_out(record: &Record, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    crud::update(record, pool).await?;
    Ok(())
}

fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

fn format_duration(duration: TimeDelta) -> String {
    let total_secs = duration.num_seconds();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let subseconds = duration.subsec_nanos(); // For milliseconds, or use subsec_nanos for more precision

    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours, minutes, seconds, subseconds
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url: String;

    #[cfg(debug_assertions)]
    {
        dotenvy::dotenv().ok();
        url = std::env::var("DATABASE_URL").unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        let home = homedir::my_home()?.expect("Unable to find home directory!");
        let db_path = home.join(".clockdb");
        let path_str = db_path.to_str().unwrap();
        url = format!("sqlite:{}", path_str);
        println!("Using database at {}", url);
    }

    let pool = setup(url.as_str()).await?;

    let cmd = Cli::parse();
    match cmd.command {
        Cmds::Version => println!("clock v{}-{}, by {}", VERSION, MODE, AUTHORS),
        Cmds::In(args) => match find_active_session(&pool).await? {
            Some(active) => {
                println!("There is currently an active session. Clock out first.");
                println!("Job Name: {} | Job ID: {}", active.job_name, active.id);
                println!("Consider running `clock --out` to end the current session.");
            }
            None => {
                let result = clock_in(args.job, now(), &pool).await?;
                println!(
                    "Clock in to {} at {}, job id {}",
                    result.job_name, result.clock_in, result.id
                );
            }
        },
        Cmds::Out(args) => match find_active_session(&pool).await? {
            Some(active) => {
                let clock_out_record = Record {
                    clock_out: Some(now()),
                    message: Some(args.message),
                    ..active
                };

                clock_out(&clock_out_record, &pool).await?;
                println!("Clock out of {} at {}", clock_out_record.job_name, now());
            }
            None => {
                println!("There is no active session. Clock in first.");
            }
        },
        Cmds::Watch { n } => loop {
            if let Some(session) = find_active_session(&pool).await? {
                let elapsed = now().signed_duration_since(session.clock_in);
                let message = format!(
                    "Session <<{}>> active time: {}",
                    session.job_name.to_uppercase(),
                    format_duration(elapsed)
                );

                display::display_message(message);

                tokio::time::sleep(std::time::Duration::from_secs(n)).await;
            } else {
                display::clear_screen();
                display::display_msg_at("No active session.".into(), 0, 0);
                println!();
                return Ok(());
            }
        },
        Cmds::LS => {
            crud::find_all(&pool).await?.iter().for_each(|r| {
                let out = match r.clock_out {
                    None => "None".to_string(),
                    Some(time) => time.to_string(),
                };

                println!(
                    "Job Name: {} | Job ID: {} | Clock In: {} | Clock Out: {} | Message: {}",
                    r.job_name,
                    r.id,
                    r.clock_in,
                    out,
                    r.message.clone().unwrap_or("None".to_string())
                );
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATABASE_URL: &str = "sqlite::memory:";

    #[tokio::test]
    async fn test_create_entry() {
        let pool = setup(TEST_DATABASE_URL).await.unwrap();
        let e = CreateRecord::default();
        crud::create(&e, &pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_record() {
        let pool = setup(TEST_DATABASE_URL).await.unwrap();
        let e = CreateRecord {
            job_name: "foo".to_string(),
            ..Default::default()
        };
        let created = crud::create(&e, &pool).await.unwrap();

        let retrieved = crud::find_by_id(created.id, &pool).await.unwrap();
        assert_eq!(created, retrieved);
    }

    #[tokio::test]
    async fn test_modify_record() {
        let pool = setup(TEST_DATABASE_URL).await.unwrap();
        let e = CreateRecord {
            job_name: "foo".to_string(),
            ..Default::default()
        };
        let mut created = crud::create(&e, &pool).await.unwrap();

        let retrieved = crud::find_by_id(created.id, &pool).await.unwrap();
        assert!(retrieved.clock_out.is_none());

        let now = Local::now().naive_local();
        created.clock_out = Some(now);

        crud::update(&created, &pool).await.unwrap();

        let retrieved = crud::find_by_id(created.id, &pool).await.unwrap();

        assert_eq!(retrieved.clock_out, created.clock_out);
    }
}
