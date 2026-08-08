//! `dodiddoneui migrate --from-electron <path>`: Electron‑era roseui.db → DoDidDoneUi schema.
//!
//! Copies the source database, applies schema repair + SQLx migrations, and
//! produces a [MigrateReport] with per‑table row counts. On failure the
//! original source is restored from a timestamped backup.
//!
//! Supports `--dry-run` (validates against a temp copy, discards it) and
//! `--backup` (custom backup directory; defaults next to the source file).

use std::path::PathBuf;
use std::process::ExitCode;

use roseui_db::migrate_from_electron;

use crate::cli::{Cli, MigrateArgs};
use crate::commands::error::{CliBoundaryCode, CliBoundaryError};

const SUBCOMMAND: &str = "migrate";

pub(crate) async fn run_migrate(args: &MigrateArgs, cli: &Cli) -> Result<ExitCode, CliBoundaryError> {
    let target_db = default_target_db_for_data_dir(cli);
    let backup_dir = args.backup.as_deref();

    if args.dry_run {
        println!("╭─ Dry-run: Electron → DoDidDoneUi migration ───────────────────────────────────╮");
        println!("│ source:  {:<65} │", args.from_electron.display().to_string());
        println!("│ target:  {:<65} │", target_db.display().to_string());
        println!("╰─────────────────────────────────────────────────────────────────────────────╯");
    }

    let report = migrate_from_electron(&args.from_electron, &target_db, args.dry_run, backup_dir)
        .await
        .map_err(|e| {
            CliBoundaryError::new(
                if args.dry_run {
                    CliBoundaryCode::CliMigrateDryRunFailed
                } else {
                    CliBoundaryCode::CliMigrateDatabaseFailed
                },
                SUBCOMMAND,
                "migration failed",
            )
            .with_field("detail", e.to_string())
        })?;

    let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| String::from("{}"));
    println!("{json}");

    if args.dry_run {
        println!();
        println!("Dry-run passed: all migrations applied, per-table counts verified.");
        println!("Run without --dry-run to write the migrated database to disk.");
    }

    Ok(ExitCode::SUCCESS)
}

fn default_target_db_for_data_dir(cli: &Cli) -> PathBuf {
    cli.data_dir.join("dodiddoneui-backend.db")
}
