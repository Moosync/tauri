use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use diesel::{
    connection::SimpleConnection,
    insert_into,
    query_dsl::methods::FilterDsl,
    r2d2::{self, ConnectionManager, Pool},
    ExpressionMethods, RunQueryDsl, SqliteConnection,
};

use serde::{Deserialize, Serialize};
use tracing::info;
use types::cache::CacheModel;
use types::errors::Result;

use super::migrations::run_migration_cache;
use types::cache_schema::{
    self,
    cache::{dsl::cache, url},
};

#[derive(Debug)]
pub struct CacheHolder {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl CacheHolder {
    #[tracing::instrument(level = "trace", skip(path))]
    pub fn new(path: PathBuf) -> Self {
        let db = Self {
            pool: Self::connect(path),
        };

        run_migration_cache(&mut db.pool.get().expect("Failed to get connection to DB"));
        db.pool.get().unwrap().batch_execute("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");
        db
    }

    #[tracing::instrument(level = "trace", skip(path))]
    fn connect(path: PathBuf) -> Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(path.to_str().unwrap());

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "trace", skip(self, _url, blob, expires))]
    pub fn set<T>(&self, _url: &str, blob: &T, expires: i32) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.pool.get().unwrap();

        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let new_expires = current_time + Duration::from_secs(expires.unsigned_abs() as u64);

        let cache_model = CacheModel {
            id: None,
            url: _url.to_string(),
            blob: serde_json::to_vec(blob)?,
            expires: new_expires.as_secs() as i64,
        };
        insert_into(cache)
            .values(cache_model.clone())
            .on_conflict(cache_schema::cache::url)
            .do_update()
            .set(cache_model)
            .execute(&mut conn)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, _url))]
    pub fn get<T>(&self, _url: &str) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut conn = self.pool.get().unwrap();

        let data: CacheModel = cache.filter(url.eq(_url)).first::<CacheModel>(&mut conn)?;
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        let expires = Duration::from_secs(data.expires as u64);
        if current_time > expires {
            info!("Cache expired");
            return Err("Cache expired".into());
        }

        let parsed: T = serde_json::from_slice(&data.blob)?;
        Ok(parsed)
    }
}
