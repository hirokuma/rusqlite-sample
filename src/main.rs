use anyhow::Result;
use bitcoin::{NetworkKind, PrivateKey, key::Secp256k1};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
struct DbData {
    #[serde(with = "serde_bytes_array")]
    hash: [u8; 32],
    #[serde(with = "serde_bytes_array")]
    pubkey: [u8; 33],
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
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

#[derive(Debug, Serialize, Deserialize)]
struct DbUpdateData {
    #[serde(with = "serde_bytes_array")]
    hash: [u8; 32],
    #[serde(with = "serde_bytes_array")]
    pubkey: [u8; 33],
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
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
        serde_rusqlite::to_params_named(&data)?
            .to_slice()
            .as_slice(),
    )?;

    let hash = Sha256::digest(b"Guten Tag");
    let updata = DbUpdateData {
        hash: hash.into(),
        pubkey: data.pubkey,
        updated_at: OffsetDateTime::now_utc(),
    };
    conn.execute(
        "UPDATE
            test
        SET
            hash = :hash,
            updated_at = :updated_at
        WHERE
            pubkey = :pubkey
        ",
        serde_rusqlite::to_params_named(&updata)?
            .to_slice()
            .as_slice(),
    )?;

    let mut stmt = conn.prepare("SELECT * FROM test")?;
    let rows: Vec<DbData> =
        serde_rusqlite::from_rows(stmt.query([])?).collect::<serde_rusqlite::Result<_>>()?;
    for data in rows {
        println!("{}", data);
    }
    Ok(())
}

mod serde_bytes_array {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<const N: usize, S>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, const N: usize, D>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|bytes: Vec<u8>| D::Error::invalid_length(bytes.len(), &"a byte array"))
    }
}
