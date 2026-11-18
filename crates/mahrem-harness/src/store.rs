//! SQLite'taki örnek durumları.
//!
//! Bitmiş örneklerin doğruluk kaynağı shard'lardır; açılışta veritabanı
//! shard'lara göre düzeltilir. Veritabanı reddedilen ve başarısız örnekleri,
//! deneme sayılarını ve çalıştırmanın meta bilgisini tutar.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, OptionalExtension as _, params};

use crate::HarnessError;
use crate::runner::Tally;

/// Örnek durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Done,
    Rejected,
    Failed,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Done => "done",
            Status::Rejected => "rejected",
            Status::Failed => "failed",
        }
    }
}

pub(crate) struct Store {
    conn: Connection,
}

impl Store {
    pub(crate) fn open(path: &Path) -> Result<Self, HarnessError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS items (
                 id TEXT PRIMARY KEY,
                 idx INTEGER NOT NULL,
                 status TEXT NOT NULL CHECK (status IN ('done', 'rejected', 'failed')),
                 attempts INTEGER NOT NULL,
                 reason TEXT,
                 shard TEXT
             );",
        )?;
        Ok(Store { conn })
    }

    pub(crate) fn meta(&self, key: &str) -> Result<Option<String>, HarnessError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| r.get(0))
            .optional()?)
    }

    pub(crate) fn set_meta(&self, key: &str, value: &str) -> Result<(), HarnessError> {
        self.conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        )?;
        Ok(())
    }

    pub(crate) fn mark(
        &self,
        id: &str,
        index: u64,
        status: Status,
        attempts: u32,
        reason: Option<&str>,
        shard: Option<&str>,
    ) -> Result<(), HarnessError> {
        self.conn.execute(
            "INSERT INTO items (id, idx, status, attempts, reason, shard)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                 status = excluded.status,
                 attempts = items.attempts + excluded.attempts,
                 reason = excluded.reason,
                 shard = excluded.shard",
            params![
                id,
                i64::try_from(index).unwrap_or(i64::MAX),
                status.as_str(),
                attempts,
                reason,
                shard
            ],
        )?;
        Ok(())
    }

    /// Veritabanını shard'lara göre düzeltir: shard'daki her örnek `done`,
    /// shard'da olmayan `done` kayıtları silinir (yeniden çalıştırılır).
    /// Düzeltilen kayıt sayısını döner.
    pub(crate) fn reconcile(
        &mut self,
        in_shards: &HashMap<String, (u64, String)>,
    ) -> Result<u64, HarnessError> {
        let tx = self.conn.transaction()?;
        let mut fixed = 0;
        let done: HashSet<String> = {
            let mut stmt = tx.prepare("SELECT id FROM items WHERE status = 'done'")?;
            stmt.query_map([], |r| r.get(0))?
                .collect::<Result<_, _>>()?
        };
        for id in done.iter().filter(|id| !in_shards.contains_key(*id)) {
            tx.execute("DELETE FROM items WHERE id = ?1", [id])?;
            fixed += 1;
        }
        for (id, (index, shard)) in in_shards.iter().filter(|(id, _)| !done.contains(*id)) {
            tx.execute(
                "INSERT INTO items (id, idx, status, attempts, reason, shard)
                 VALUES (?1, ?2, 'done', 1, NULL, ?3)
                 ON CONFLICT(id) DO UPDATE SET status = 'done', reason = NULL, shard = excluded.shard",
                params![id, i64::try_from(*index).unwrap_or(i64::MAX), shard],
            )?;
            fixed += 1;
        }
        tx.commit()?;
        Ok(fixed)
    }

    /// Yeniden çalıştırılmayacak örnekler: bitmiş ve reddedilmiş.
    pub(crate) fn finished(&self) -> Result<HashSet<String>, HarnessError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM items WHERE status IN ('done', 'rejected')")?;
        let ids = stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        Ok(ids)
    }

    pub(crate) fn counts(&self) -> Result<Tally, HarnessError> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM items GROUP BY status")?;
        let mut counts = Tally::default();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        for row in rows {
            let (status, n) = row?;
            let n = u64::try_from(n).unwrap_or(0);
            match status.as_str() {
                "done" => counts.done = n,
                "rejected" => counts.rejected = n,
                _ => counts.failed = n,
            }
        }
        Ok(counts)
    }

    /// Ret gerekçeleri, en sık olandan başlayarak.
    pub(crate) fn reject_reasons(&self) -> Result<Vec<(String, u64)>, HarnessError> {
        let mut stmt = self.conn.prepare(
            "SELECT reason, COUNT(*) AS n FROM items WHERE status = 'rejected'
             GROUP BY reason ORDER BY n DESC, reason",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    u64::try_from(r.get::<_, i64>(1)?).unwrap_or(0),
                ))
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }
}
