use anyhow::Result;
use bitcoin::{NetworkKind, PrivateKey, key::Secp256k1};
use rusqlite::{Connection, named_params};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

struct DbData {
    hash: [u8; 32],
    pubkey: [u8; 33],
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
}

impl std::fmt::Display for DbData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "hash: {}", hex::encode(self.hash))?;
        writeln!(f, "pubkey: {}", hex::encode(self.pubkey))?;
        writeln!(f, "created_at: {}", self.created_at)?;
        match self.updated_at {
            Some(updated_at) => write!(f, "updated_at: {updated_at}"),
            None => write!(f, "updated_at: None"),
        }
    }
}

fn main() -> Result<()> {
    // let conn = Connection::open_in_memory()?;
    let conn = Connection::open("/tmp/test.db")?;
    conn.execute(
        "CREATE TABLE test (
            hash BLOB NOT NULL,
            pubkey BLOB NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT
        )",
        [],
    )?;

    let hash = Sha256::digest(b"hello world");
    let privkey = PrivateKey::generate(NetworkKind::Test);
    let secp = Secp256k1::new();
    let pubkey = privkey.public_key(&secp);
    let data = DbData {
        hash: hash.into(),
        pubkey: pubkey.inner.serialize(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: None,
    };
    conn.execute(
        "INSERT INTO test (
            hash, pubkey, created_at, updated_at
        ) VALUES (
            :hash,
            :pubkey,
            :created_at,
            :updated_at
        )",
        named_params! {
            ":hash": data.hash,
            ":pubkey": data.pubkey,
            ":created_at": data.created_at,
            ":updated_at": data.updated_at,
        },
    )?;

    let mut stmt = conn.prepare("SELECT * FROM test")?;
    let i_hash = stmt.column_index("hash")?;
    let i_pubkey = stmt.column_index("pubkey")?;
    let i_created_at = stmt.column_index("created_at")?;
    let i_updated_at = stmt.column_index("updated_at")?;
    let iter = stmt.query_map([], |row| {
        Ok(DbData {
            hash: row.get(i_hash)?,
            pubkey: row.get(i_pubkey)?,
            created_at: row.get(i_created_at)?,
            updated_at: row.get(i_updated_at)?,
        })
    })?;
    for data in iter {
        println!("{}", data?);
    }
    Ok(())
}
