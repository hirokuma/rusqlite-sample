//! 自前の型を SQLite の日時文字列として保存する例。
//!
//! フォーマット文字列は一切書かず、`time` 型に対する rusqlite の
//! `ToSql` / `FromSql` 実装へ処理を委譲する。
//! これにより、rusqlite が決めている保存形式
//! （`"YYYY-MM-DD HH:MM:SS.SSS+HH:MM"` など）と常に一致する。
//!
//! 実行: `cargo run --example to_sql`

use anyhow::Result;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};
use rusqlite::{Connection, named_params};
use time::OffsetDateTime;

/// アプリ固有の型（例）。SQLite には rusqlite の日時文字列で保存する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Timestamp(OffsetDateTime);

impl From<Timestamp> for OffsetDateTime {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

impl ToSql for Timestamp {
    /// 書式は自前で用意せず、`OffsetDateTime` の `ToSql` 実装に丸投げする。
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl FromSql for Timestamp {
    /// 読み込みも `OffsetDateTime` の `FromSql` 実装に丸投げする。
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        OffsetDateTime::column_result(value).map(Self)
    }
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
        "CREATE TABLE event (
            id    INTEGER PRIMARY KEY,
            at    TEXT NOT NULL,
            label TEXT NOT NULL
        )",
        (),
    )?;

    let at = Timestamp(OffsetDateTime::from_unix_timestamp_nanos(
        1_793_020_975_123_000_000,
    )?);
    conn.execute(
        "INSERT INTO event (at, label) VALUES (:at, :label)",
        named_params! { ":at": at, ":label": "sample" },
    )?;

    let stored: String = conn.query_row("SELECT at FROM event", [], |row| row.get(0))?;
    println!("保存された文字列        : {stored}");

    let read_back: Timestamp = conn.query_row("SELECT at FROM event", [], |row| row.get(0))?;
    println!("FromSql で読み直し      : {read_back:?}");
    assert_eq!(read_back, at);

    let parsed_by_sqlite: String = conn.query_row(
        "SELECT strftime('%Y-%m-%d %H:%M:%f', at) FROM event",
        [],
        |row| row.get(0),
    )?;
    println!("SQLite の日時関数で解釈 : {parsed_by_sqlite}");

    // 文字列そのものが欲しい場合は ToSqlOutput から取り出せる
    println!("to_sql() が返す文字列   : {}", to_sql_text(at.to_sql()?));

    Ok(())
}

/// `ToSql::to_sql()` が返す `ToSqlOutput` から TEXT を取り出すヘルパー。
fn to_sql_text(out: ToSqlOutput<'_>) -> String {
    match out {
        ToSqlOutput::Borrowed(ValueRef::Text(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
        ToSqlOutput::Owned(Value::Text(text)) => text,
        other => format!("(not text: {other:?})"),
    }
}
