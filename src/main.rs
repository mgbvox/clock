use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::{SqlitePool, Sqlite};


#[cfg(debug_assertions)]
const DB_URL: &str = "sqlite:./clock_DEBUG.db";

#[cfg(not(debug_assertions))]
const DB_URL: &str = "sqlite:./clock.db";


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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_db().await;
    let pool = SqlitePool::connect(DB_URL).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(())
}