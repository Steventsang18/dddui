use crate::error::DbError;
use crate::models::IndustryTemplateConfig;

/// Industry template selection + company-level override data access.
///
/// `get_config` returns `None` if the user has not picked a template yet
/// (caller falls back to a default template or "no template").
/// `upsert_config` inserts or replaces the user's single row.
#[async_trait::async_trait]
pub trait IIndustryTemplateRepository: Send + Sync {
    /// Returns the user's template config, or `None` if none persisted.
    async fn get_config(&self, user_id: &str) -> Result<Option<IndustryTemplateConfig>, DbError>;

    /// Inserts or replaces the user's template config.
    async fn upsert_config(
        &self,
        user_id: &str,
        template_id: &str,
        override_json: &str,
    ) -> Result<IndustryTemplateConfig, DbError>;

    /// Removes the user's template config (falls back to no template).
    async fn delete_config(&self, user_id: &str) -> Result<(), DbError>;
}
