use crate::models::{CreateRecord, Record};
use sqlx::SqlitePool;

pub async fn create(record: &CreateRecord, pool: &SqlitePool) -> Result<Record, sqlx::Error> {
    let out = sqlx::query(
        r#"
        INSERT INTO work_hours
            (job_name, clock_in, clock_out, message)
        VALUES
            (?, ?, ?, ?)"#,
    )
    .bind(&record.job_name)
    .bind(record.clock_in)
    .bind(record.clock_out)
    .bind(&record.message)
    .execute(pool)
    .await?;

    Ok(Record {
        id: out.last_insert_rowid(),
        job_name: record.job_name.clone(),
        clock_in: record.clock_in,
        clock_out: record.clock_out,
        message: record.message.clone(),
    })
}

pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Record>, sqlx::Error> {
    let result: Vec<Record> = sqlx::query_as(
        r#"
        SELECT * from work_hours
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(result)
}

pub async fn find_by_id(id: i64, pool: &SqlitePool) -> Result<Record, sqlx::Error> {
    let result = sqlx::query_as(
        r#"
        SELECT * from work_hours
        WHERE
            id = ?
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(result)
}

pub async fn update(record: &Record, pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE 
            work_hours
        SET
            job_name = ?,
            clock_in = ?,
            clock_out = ?,
            message = ?
        WHERE
            id = ?
        "#,
    )
    .bind(&record.job_name)
    .bind(record.clock_in)
    .bind(record.clock_out)
    .bind(&record.message)
    .bind(record.id)
    .execute(pool)
    .await?;

    Ok(())
}
