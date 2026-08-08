use std::path::{Path, PathBuf};

use sqlx::SqlitePool;
use tracing::info;

use crate::error::DbError;
use crate::DatabaseInitError;

// ── Schema repair contract (unchanged) ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyHandoffColumn {
    pub(crate) table: &'static str,
    pub(crate) column: &'static str,
    pub(crate) definition: &'static str,
}

pub(crate) const LEGACY_HANDOFF_COLUMNS: &[LegacyHandoffColumn] = &[
    LegacyHandoffColumn {
        table: "cron_jobs",
        column: "skill_content",
        definition: "TEXT",
    },
    LegacyHandoffColumn {
        table: "cron_jobs",
        column: "description",
        definition: "TEXT",
    },
    LegacyHandoffColumn {
        table: "conversations",
        column: "pinned",
        definition: "INTEGER NOT NULL DEFAULT 0",
    },
    LegacyHandoffColumn {
        table: "conversations",
        column: "pinned_at",
        definition: "INTEGER",
    },
    LegacyHandoffColumn {
        table: "teams",
        column: "session_mode",
        definition: "TEXT",
    },
    LegacyHandoffColumn {
        table: "teams",
        column: "agents_version",
        definition: "TEXT NOT NULL DEFAULT '1.0.0'",
    },
];

pub(crate) async fn ensure_legacy_handoff_schema(pool: &SqlitePool) -> Result<(), DbError> {
    for column in LEGACY_HANDOFF_COLUMNS {
        ensure_legacy_handoff_column(pool, *column).await?;
    }
    Ok(())
}

async fn ensure_legacy_handoff_column(pool: &SqlitePool, column: LegacyHandoffColumn) -> Result<(), DbError> {
    let table_exists: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?")
        .bind(column.table)
        .fetch_one(pool)
        .await
        .map_err(DbError::Query)?;

    if !table_exists {
        return Ok(());
    }

    let column_exists: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM pragma_table_info(?) WHERE name = ?")
        .bind(column.table)
        .bind(column.column)
        .fetch_one(pool)
        .await
        .map_err(DbError::Query)?;

    if column_exists {
        return Ok(());
    }

    let sql = format!(
        "ALTER TABLE {} ADD COLUMN {} {}",
        column.table, column.column, column.definition
    );
    sqlx::query(&sql).execute(pool).await.map_err(DbError::Query)?;
    info!(
        table = column.table,
        column = column.column,
        "added missing legacy handoff column"
    );

    Ok(())
}

// ── Explicit migration (CLI `migrate --from-electron`) ────────────────

/// Per-table row count after a migration.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TableCount {
    pub table: String,
    pub rows: i64,
}

/// Outcome of an explicit Electron‑to‑DoDidDoneUi migration run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrateReport {
    /// Path to the source Electron database.
    pub source_path: String,
    /// Path to the target DoDidDoneUi database.
    pub target_path: String,
    /// Path to the timestamped backup (None when `dry_run`).
    pub backup_path: Option<String>,
    /// Whether this was a dry-run (no files were modified).
    pub dry_run: bool,
    /// Whether the migration completed without error.
    pub success: bool,
    /// Number of SQLx migrations applied.
    pub migrations_applied: i64,
    /// Per‑table row counts after migration.
    pub tables: Vec<TableCount>,
}

/// Key tables to verify after migration.
const VERIFY_TABLES: &[&str] = &[
    "users",
    "conversations",
    "messages",
    "providers",
    "cron_jobs",
    "teams",
    "assistants",
];

/// Migrate an Electron‑era `roseui.db` to an DoDidDoneUi‑compatible database.
///
/// # Arguments
/// * `source_db` — Path to the Electron `roseui.db`.
/// * `target_db` — Destination path for the migrated database.
/// * `dry_run` — If `true`, runs the full migration against a temporary copy
///   and discards it — no files are modified. Use this to validate before
///   committing.
/// * `backup_dir` — If `Some` and `dry_run` is `false`, a timestamped copy
///   of the source database is saved here before migration begins. If `None`,
///   the backup is placed next to `source_db`.
///
/// # Errors
/// Returns `DbError` when the source does not exist, the copy fails, or the
/// migration pipeline itself fails (schema repair + SQLx migrations).
pub async fn migrate_from_electron(
    source_db: &Path,
    target_db: &Path,
    dry_run: bool,
    backup_dir: Option<&Path>,
) -> Result<MigrateReport, DbError> {
    let source_db = source_db.canonicalize().map_err(|e| {
        DbError::Init(format!("source database not found: {e}"))
    })?;

    if !source_db.exists() {
        return Err(DbError::Init(format!(
            "source database does not exist: {}",
            source_db.display()
        )));
    }

    let backup_path = if dry_run {
        None
    } else {
        let backup = create_backup(&source_db, backup_dir)?;
        info!(
            source = %source_db.display(),
            backup = %backup.display(),
            "created backup of legacy database"
        );
        Some(backup)
    };

    let work_db = if dry_run {
        let tmp = target_db.with_extension("db.tmp");
        copy_db_file(&source_db, &tmp)?;
        tmp
    } else {
        copy_db_file(&source_db, target_db)?;
        target_db.to_path_buf()
    };

    let result = run_full_migration(&work_db).await;

    if dry_run {
        // Always clean up temp file on dry-run.
        let _ = std::fs::remove_file(&work_db);
        let _ = std::fs::remove_file(work_db.with_extension("db-wal"));
        let _ = std::fs::remove_file(work_db.with_extension("db-shm"));
    }

    match result {
        Ok(report) => Ok(MigrateReport {
            source_path: source_db.display().to_string(),
            target_path: target_db.display().to_string(),
            backup_path: backup_path.map(|p| p.display().to_string()),
            dry_run,
            success: true,
            ..report
        }),
        Err(e) => {
            if !dry_run {
                // Restore from backup on failure.
                if let Some(ref backup) = backup_path {
                    info!(
                        error = %e,
                        backup = %backup.display(),
                        "migration failed; restoring from backup"
                    );
                    let _ = std::fs::remove_file(target_db);
                    let _ = copy_db_file(backup, target_db);
                }
            }
            Err(e)
        }
    }
}

/// Create a timestamped backup of the source database.
fn create_backup(source: &Path, backup_dir: Option<&Path>) -> Result<PathBuf, DbError> {
    let dir = backup_dir.unwrap_or_else(|| source.parent().unwrap_or(Path::new(".")));
    std::fs::create_dir_all(dir).map_err(|e| {
        DbError::Init(format!("failed to create backup directory: {e}"))
    })?;

    let ts = roseui_common::now_ms();
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("roseui");
    let backup_name = format!("{stem}.backup.{ts}.db");
    let backup_path = dir.join(backup_name);

    copy_db_file(source, &backup_path)?;
    Ok(backup_path)
}

/// Copy a SQLite database file (data file only, not WAL/SHM).
fn copy_db_file(src: &Path, dst: &Path) -> Result<(), DbError> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DbError::Init(format!("failed to create target directory: {e}"))
        })?;
    }

    let tmp = dst.with_extension("db.tmp");
    std::fs::copy(src, &tmp).map_err(|e| {
        DbError::Init(format!("failed to copy database file: {e}"))
    })?;

    // Clean up WAL/SHM that may have been left by a previous copy.
    let _ = std::fs::remove_file(dst.with_extension("db-wal"));
    let _ = std::fs::remove_file(dst.with_extension("db-shm"));

    std::fs::rename(&tmp, dst).map_err(|e| {
        DbError::Init(format!("failed to rename database file: {e}"))
    })?;

    Ok(())
}

/// Run the full migration pipeline on a given database file and return
/// a report with applied counts and table verification.
///
/// Delegates to `init_database_staged` so the pipeline (schema repair →
/// migrate_repair → SQLx migrations → pool reopen → ensure_system_user) is
/// identical to a normal server startup. We then read the migration count
/// and per‑table row counts from the resulting pool.
async fn run_full_migration(db_path: &Path) -> Result<MigrateReport, DbError> {
    let db = crate::init_database_staged(db_path)
        .await
        .map_err(DatabaseInitError::into_source)?;

    let pool = db.pool();

    let migrations_applied: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE success = 1",
    )
    .fetch_one(pool)
    .await
    .map_err(DbError::Query)?;

    let mut tables = Vec::with_capacity(VERIFY_TABLES.len());
    for table in VERIFY_TABLES {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM \"{table}\""))
            .fetch_one(pool)
            .await
            .map_err(DbError::Query)?;
        tables.push(TableCount {
            table: table.to_string(),
            rows: count,
        });
    }

    db.close().await;

    Ok(MigrateReport {
        source_path: String::new(),
        target_path: String::new(),
        backup_path: None,
        dry_run: false,
        success: false,
        migrations_applied,
        tables,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_contains_initial_handoff_columns() {
        let actual: Vec<_> = LEGACY_HANDOFF_COLUMNS
            .iter()
            .map(|column| (column.table, column.column, column.definition))
            .collect();

        assert_eq!(
            actual,
            vec![
                ("cron_jobs", "skill_content", "TEXT"),
                ("cron_jobs", "description", "TEXT"),
                ("conversations", "pinned", "INTEGER NOT NULL DEFAULT 0"),
                ("conversations", "pinned_at", "INTEGER"),
                ("teams", "session_mode", "TEXT"),
                ("teams", "agents_version", "TEXT NOT NULL DEFAULT '1.0.0'"),
            ]
        );
    }

    #[test]
    fn migration_002_direct_legacy_reads_are_audited() {
        let repaired_by_handoff_contract = [
            ("cron_jobs", "skill_content"),
            ("cron_jobs", "description"),
            ("conversations", "pinned"),
            ("conversations", "pinned_at"),
            ("teams", "session_mode"),
            ("teams", "agents_version"),
        ];

        for (table, column) in repaired_by_handoff_contract {
            assert!(
                LEGACY_HANDOFF_COLUMNS
                    .iter()
                    .any(|entry| entry.table == table && entry.column == column),
                "migration 002 reads {table}.{column}; it must stay in the handoff repair contract"
            );
        }

        // These columns are also directly read by migration 002, but they are
        // not part of this repair contract because current evidence does not
        // show compatible drift for them. Keep them documented here so review
        // of future migration-002 edits has an explicit contract decision
        // point instead of a hidden assumption.
        let documented_non_contract_reads = [
            ("messages", "hidden", "RoseUi v22 adds it before v23-v26 issue path"),
            (
                "conversations",
                "source",
                "RoseUi v8 baseline before v23-v26 issue path",
            ),
            (
                "conversations",
                "channel_chat_id",
                "RoseUi v14 baseline before v23-v26 issue path",
            ),
            ("mailbox", "files", "RoseUi v25 adds it on the observed v23->v26 path"),
            (
                "remote_agents",
                "allow_insecure",
                "RoseUi v18 baseline before v23-v26 issue path",
            ),
            (
                "cron_jobs",
                "execution_mode",
                "RoseUi v22 baseline before v23-v26 issue path",
            ),
            (
                "cron_jobs",
                "agent_config",
                "RoseUi v22 baseline before v23-v26 issue path",
            ),
        ];

        let migration_002 = include_str!("../migrations/002_legacy_data_normalize.sql");
        for (table, column, reason) in documented_non_contract_reads {
            assert!(
                migration_002.contains(column),
                "documented migration 002 read {table}.{column} disappeared or was renamed; revisit the audit entry: {reason}"
            );
        }
    }
}
