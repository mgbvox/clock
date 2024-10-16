use chrono::{Local, NaiveDateTime};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::io::{self, Write};
use clap::{Arg, ArgAction, Command};
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;
use crossterm::{execute, terminal::{Clear, ClearType}, cursor};
use std::thread;
use std::time::Instant;
use sqlx::migrate::MigrateDatabase;

#[derive(Debug, PartialEq, Eq)]
struct Record {
    id: i64,
    job_name: String,
    clock_in: NaiveDateTime,
    clock_out: Option<NaiveDateTime>,
    message: Option<String>,
}

#[cfg(debug_assertions)]
const DB_URL: &str = "sqlite://work_hours_DEBUG.db";


#[cfg(not(debug_assertions))]
const DB_URL: &str = "sqlite://work_hours.db";

async fn ensure_db() {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database at {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Database created successfully."),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Using database at {}", DB_URL);
    }
}


async fn ensure_tables(url: &str) -> Result<Pool<Sqlite>, Box<dyn std::error::Error>> {
    let pool = SqlitePoolOptions::new()
        .connect(url)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS work_hours (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            job_name TEXT NOT NULL,
            clock_in DATETIME NOT NULL,
            clock_out DATETIME,
            message TEXT
        )
        "#,
    )
        .execute(&pool)
        .await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("clock")
        .version("1.0")
        .author("Matthew Billman")
        .about("Clock in/out for work")
        .arg(Arg::new("in")
            .long("in")
            .action(ArgAction::Set)
            .value_name("JOBNAME_OR_JOBID")
            .help("Clock in with a job name or job ID"))
        .arg(Arg::new("out")
            .long("out")
            .action(ArgAction::Set)
            .help("Clock out of the current session"))
        .arg(Arg::new("message")
            .short('m')
            .action(ArgAction::Set)
            .value_name("MESSAGE")
            .help("Message to add to the clock-out log"))
        .arg(Arg::new("list")
            .long("ls")
            .action(ArgAction::Set)
            .help("List all clock-in and clock-out records"))
        .get_matches();

    ensure_db().await;
    let pool = ensure_tables(DB_URL).await?;


    //
    // let timer_running = Arc::new(Mutex::new(false));


    //
    // if let Some(job_name) = matches.value_of("in") {
    //     clock_in(&pool, job_name.to_string(), timer_running.clone()).await?;
    // } else if matches.is_present("out") {
    //     let message = matches.value_of("message").map(|s| s.to_string());
    //     clock_out(&pool, message, timer_running.clone()).await?;
    // } else if matches.is_present("list") {
    //     list_records(&pool).await?;
    // } else {
    //     println!("No valid option provided. Use --help for more information.");
    // }

    Ok(())
}
//
// async fn clock_in(pool: &Pool<Sqlite>, job_name: String, timer_running: Arc<Mutex<bool>>) -> Result<(), Box<dyn std::error::Error>> {
//     let now = Local::now().naive_local();
//     sqlx::query("INSERT INTO work_hours (job_name, clock_in) VALUES (?, ?)")
//         .bind(job_name.clone())
//         .bind(now)
//         .execute(pool)
//         .await?;
//
//     println!("Clocked in at {} for job: {}", now, job_name);
//
//     let mut running = timer_running.lock().await;
//     *running = true;
//
//     thread::spawn(move || {
//         let start = Instant::now();
//         loop {
//             let elapsed = start.elapsed();
//             let hours = elapsed.as_secs() / 3600;
//             let minutes = (elapsed.as_secs() % 3600) / 60;
//             let seconds = elapsed.as_secs() % 60;
//
//             if let Ok(mut running) = timer_running.try_lock() {
//                 if !*running {
//                     break;
//                 }
//             }
//
//             execute!(
//                 io::stdout(),
//                 cursor::MoveTo(0, 0),
//                 Clear(ClearType::CurrentLine),
//                 cursor::MoveToColumn(70),
//                 format!("Running Time: {:02}:{:02}:{:02}", hours, minutes, seconds).to_string(),
//             ).unwrap();
//
//             thread::sleep(Duration::from_secs(1));
//         }
//     });
//
//     Ok(())
// }
//
// async fn clock_out(pool: &Pool<Sqlite>, message: Option<String>, timer_running: Arc<Mutex<bool>>) -> Result<(), Box<dyn std::error::Error>> {
//     let now = Local::now().naive_local();
//     let record = sqlx::query_as::<_, Record>(
//         "SELECT * FROM work_hours WHERE clock_out IS NULL ORDER BY clock_in DESC LIMIT 1",
//     )
//         .fetch_optional(pool)
//         .await?;
//
//     if let Some(record) = record {
//         sqlx::query("UPDATE work_hours SET clock_out = ?, message = ? WHERE id = ?")
//             .bind(now)
//             .bind(message.clone())
//             .bind(record.id)
//             .execute(pool)
//             .await?;
//         println!("Clocked out at {} with message: {:?}", now, message);
//     } else {
//         println!("No active clock-in found.");
//     }
//
//     let mut running = timer_running.lock().await;
//     *running = false;
//
//     Ok(())
// }
//
// async fn list_records(pool: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
//     let records = sqlx::query_as::<_, Record>(
//         "SELECT * FROM work_hours ORDER BY clock_in DESC",
//     )
//         .fetch_all(pool)
//         .await?;
//
//     println!("{:<5} {:<20} {:<20} {:<20} {:<30}", "ID", "Job Name", "Clock In", "Clock Out", "Message");
//     for record in records {
//         println!(
//             "{:<5} {:<20} {:<20} {:<20} {:<30}",
//             record.id,
//             record.job_name,
//             record.clock_in,
//             record.clock_out.map_or("-".to_string(), |t| t.to_string()),
//             record.message.clone().unwrap_or_else(|| "-".to_string())
//         );
//     }
//
//     Ok(())
// }
//
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use chrono::NaiveDate;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = ensure_tables("sqlite::memory:")
            .await
            .expect("Failed to create tables in memory.");
        pool
    }

    #[tokio::test]
    async fn test_ensure_tables() {
        let pool = setup_test_db().await;
        let out = sqlx::query_as(
            r#"SELECT name
                FROM sqlite_master
                WHERE type = 'table'
                ORDER BY name;"#
        ).fetch(&pool).await;

        o

    }
}
//
// #[tokio::test]
// async fn test_clock_in() {
//     let pool = setup_test_db().await;
//     clock_in(&pool, "TestJob".to_string(), Arc::new(Mutex::new(false))).await.expect("Clock in failed");
//
//     let record = sqlx::query_as::<_, Record>("SELECT * FROM work_hours WHERE clock_out IS NULL")
//         .fetch_one(&pool)
//         .await
//         .expect("Failed to fetch record");
//
//     assert_eq!(record.clock_out, None);
// }
//
//     #[tokio::test]
//     async fn test_clock_out() {
//         let pool = setup_test_db().await;
//         let timer_running = Arc::new(Mutex::new(false));
//         clock_in(&pool, "TestJob".to_string(), timer_running.clone()).await.expect("Clock in failed");
//         clock_out(&pool, Some("Test message".to_string()), timer_running).await.expect("Clock out failed");
//
//         let record = sqlx::query_as::<_, Record>("SELECT * FROM work_hours ORDER BY clock_in DESC LIMIT 1")
//             .fetch_one(&pool)
//             .await
//             .expect("Failed to fetch record");
//
//         assert!(record.clock_out.is_some());
//         assert_eq!(record.message, Some("Test message".to_string()));
//     }
//
//     #[tokio::test]
//     async fn test_no_active_clock_in() {
//         let pool = setup_test_db().await;
//         let result = clock_out(&pool, None, Arc::new(Mutex::new(false))).await;
//
//         assert!(result.is_ok());
//     }
// }
