use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CreateRecord {
    pub job_name: String,
    pub clock_in: NaiveDateTime,
}

#[derive(FromRow, Default, Debug, PartialEq, Eq, Clone)]
pub struct Record {
    pub id: i64,
    pub job_name: String,
    pub clock_in: NaiveDateTime,
    pub clock_out: Option<NaiveDateTime>,
    pub message: Option<String>,
}
