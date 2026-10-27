use rusqlite::{
    Connection,
    types::{FromSql, ValueRef},
};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Deserialize, Serialize)]
struct Person {
    id: i32,
    #[serde(deserialize_with = "convert_datetime")]
    #[serde(serialize_with = "convert_from_datetime")]
    created_at: OffsetDateTime,
    name: String,
}

fn main() -> anyhow::Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
            id          INTEGER PRIMARY KEY,
            created_at  TEXT DEFAULT CURRENT_TIMESTAMP,
            name        TEXT NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "INSERT INTO person (name) VALUES (?)",
        serde_rusqlite::to_params("yoshio")?,
    )?;
    let person2 = Person {
        id: 256,
        created_at: OffsetDateTime::now_utc(),
        name: "masao".to_string(),
    };
    let params = serde_rusqlite::to_params_named(&person2)?;
    conn.execute(
        "INSERT INTO person (id, created_at, name) VALUES (:id, :created_at, :name)",
        params.to_slice().as_slice(),
    )?;

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

fn convert_from_datetime<S>(datetime: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let text = datetime
        .format(&Rfc3339)
        .map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&text)
}
