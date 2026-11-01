fn main() -> anyhow::Result<()> {
    let conn = rusqlite::Connection::open("/tmp/test.db")?;

    conn.execute(
        "CREATE TABLE person (
            id          INTEGER PRIMARY KEY,
            chrono_at   TEXT,
            jiff_at     TEXT,
            time_at     TEXT,
            name        TEXT NOT NULL
        )",
        (),
    )?;

    let chrono_at: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let jiff_at = jiff::Timestamp::now();
    let time_at = time::OffsetDateTime::now_utc();
    conn.execute(
        "INSERT INTO person (chrono_at, jiff_at, time_at, name) VALUES (?, ?, ?, ?)",
        (chrono_at, jiff_at, time_at, "yoshio"),
    )?;

    Ok(())
}
