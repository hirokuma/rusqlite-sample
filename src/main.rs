use anyhow::Result;
use rusqlite::types::{FromSql, ValueRef};
use rusqlite::{Connection, named_params};
use serde::Deserialize;
use time::OffsetDateTime;

#[derive(Debug, Deserialize)]
struct Person {
    id: i32,
    #[serde(deserialize_with = "convert_datetime")]
    created_at: OffsetDateTime,
    name: String,
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
            id          INTEGER PRIMARY KEY,
            created_at  TEXT DEFAULT CURRENT_TIMESTAMP,
            name        TEXT NOT NULL
        )",
        (), // empty list of parameters.
    )?;

    let mut stmt = conn.prepare("INSERT INTO person (name) VALUES (:name)")?;
    stmt.execute(named_params! {
        ":name": "yoshio",
    })?;

    let mut stmt = conn.prepare("SELECT * FROM person")?;
    let mut rows = stmt.query([])?;
    let mut people = Vec::new();
    while let Some(row) = rows.next()? {
        let v: Person = serde_rusqlite::from_row(row)?;
        people.push(v);
    }
    for person in people {
        println!("id: {}", person.id);
        println!("  name      : {}", person.name);
        println!("  created_at: {}", person.created_at);
    }
    Ok(())
}

fn convert_datetime<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    <OffsetDateTime as FromSql>::column_result(ValueRef::Text(s.as_bytes()))
        .map_err(serde::de::Error::custom)
}
