use sqlx::SqlitePool;

use crate::error::DbError;
use crate::models::IndustryTemplateConfig;
use crate::repository::IIndustryTemplateRepository;

/// SQLite-backed implementation of [`IIndustryTemplateRepository`].
#[derive(Clone, Debug)]
pub struct SqliteIndustryTemplateRepository {
    pool: SqlitePool,
}

impl SqliteIndustryTemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IIndustryTemplateRepository for SqliteIndustryTemplateRepository {
    async fn get_config(&self, user_id: &str) -> Result<Option<IndustryTemplateConfig>, DbError> {
        let row = sqlx::query_as::<_, IndustryTemplateConfig>(
            "SELECT * FROM industry_template_config WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn upsert_config(
        &self,
        user_id: &str,
        template_id: &str,
        override_json: &str,
    ) -> Result<IndustryTemplateConfig, DbError> {
        let now = roseui_common::now_ms();

        sqlx::query(
            "INSERT INTO industry_template_config (user_id, template_id, override_json, updated_at) \
             VALUES (?, ?, ?, ?) \
             ON CONFLICT(user_id) DO UPDATE SET \
                template_id = excluded.template_id, \
                override_json = excluded.override_json, \
                updated_at = excluded.updated_at",
        )
        .bind(user_id)
        .bind(template_id)
        .bind(override_json)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(IndustryTemplateConfig {
            user_id: user_id.to_string(),
            template_id: template_id.to_string(),
            override_json: override_json.to_string(),
            updated_at: now,
        })
    }

    async fn delete_config(&self, user_id: &str) -> Result<(), DbError> {
        sqlx::query("DELETE FROM industry_template_config WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_database_memory;

    const USER_A: &str = "system_default_user";
    const USER_B: &str = "user_b";

    async fn setup() -> (SqliteIndustryTemplateRepository, crate::Database) {
        let db = init_database_memory().await.unwrap();
        sqlx::query(
            "INSERT INTO users (id, user_type, username, password_hash, status, session_generation, created_at, updated_at) \
             VALUES (?, 'local', ?, 'hash', 'active', 0, 1, 1)",
        )
        .bind(USER_B)
        .bind(USER_B)
        .execute(db.pool())
        .await
        .unwrap();
        let repo = SqliteIndustryTemplateRepository::new(db.pool().clone());
        (repo, db)
    }

    #[tokio::test]
    async fn get_config_returns_none_when_empty() {
        let (repo, _db) = setup().await;
        assert!(repo.get_config(USER_A).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn upsert_creates_config() {
        let (repo, _db) = setup().await;
        let c = repo
            .upsert_config(USER_A, "legal", "{}")
            .await
            .unwrap();

        assert_eq!(c.user_id, USER_A);
        assert_eq!(c.template_id, "legal");
        assert_eq!(c.override_json, "{}");
        assert!(c.updated_at > 0);
    }

    #[tokio::test]
    async fn upsert_then_get_returns_same() {
        let (repo, _db) = setup().await;
        let json = r#"{"approval_policy":"dangerous_only","system_prompt":"公司自定义"}"#;
        repo.upsert_config(USER_A, "legal", json).await.unwrap();

        let c = repo.get_config(USER_A).await.unwrap().unwrap();
        assert_eq!(c.template_id, "legal");
        assert_eq!(c.override_json, json);
    }

    #[tokio::test]
    async fn upsert_overwrites_existing() {
        let (repo, _db) = setup().await;
        repo.upsert_config(USER_A, "legal", "{}").await.unwrap();
        let c = repo
            .upsert_config(USER_A, "finance", r#"{"allowed_tools":["file_read"]}"#)
            .await
            .unwrap();

        assert_eq!(c.template_id, "finance");

        let fetched = repo.get_config(USER_A).await.unwrap().unwrap();
        assert_eq!(fetched.template_id, "finance");
    }

    #[tokio::test]
    async fn config_is_scoped_by_user() {
        let (repo, _db) = setup().await;
        repo.upsert_config(USER_A, "legal", "{}").await.unwrap();
        repo.upsert_config(USER_B, "medical", "{}").await.unwrap();

        let a = repo.get_config(USER_A).await.unwrap().unwrap();
        let b = repo.get_config(USER_B).await.unwrap().unwrap();
        assert_eq!(a.template_id, "legal");
        assert_eq!(b.template_id, "medical");
    }

    #[tokio::test]
    async fn delete_removes_config() {
        let (repo, _db) = setup().await;
        repo.upsert_config(USER_A, "legal", "{}").await.unwrap();
        repo.delete_config(USER_A).await.unwrap();

        assert!(repo.get_config(USER_A).await.unwrap().is_none());
    }
}
