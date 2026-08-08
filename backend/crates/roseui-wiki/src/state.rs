use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;

use crate::ingest::IngestEngine;
use crate::repository::{IWikiRepository, SqliteWikiRepository};
use crate::service::WikiService;

/// Router state for the wiki module.
///
/// Holds the shared [`WikiService`] and the [`IngestEngine`] (which needs the
/// data directory to locate the read-only `wiki/raw/` source folder). Cloning
/// is cheap (Arc).
#[derive(Clone)]
pub struct WikiRouterState {
    pub service: Arc<WikiService>,
    pub ingest: Arc<IngestEngine>,
    pub data_dir: PathBuf,
}

/// Build the default [`WikiRouterState`] from the shared SQLite pool and the
/// application data directory.
///
/// M2 wires the data layer (migration `034_wiki.sql` + `WikiRepository`); the
/// pool is supplied by `roseui-app` at startup. The `data_dir` locates the
/// `wiki/raw/` read-only source directory used by the ingest pipeline.
pub fn build_wiki_state(pool: SqlitePool, data_dir: PathBuf) -> WikiRouterState {
    let repo: Arc<dyn IWikiRepository> = Arc::new(SqliteWikiRepository::new(pool.clone()));
    let service = Arc::new(WikiService::new(repo.clone()));
    let ingest = Arc::new(IngestEngine::new(&data_dir, repo));
    WikiRouterState {
        service,
        ingest,
        data_dir,
    }
}
