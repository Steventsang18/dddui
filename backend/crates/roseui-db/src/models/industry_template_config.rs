use roseui_common::TimestampMs;
use serde::{Deserialize, Serialize};

/// Row mapping for the `industry_template_config` table.
///
/// One row per user: which builtin industry template is active, plus a
/// `serde_json` serialization of `rupoo::industry_template::TemplateOverride`.
/// `override_json == "{}"` means "no override" (template baseline applies).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IndustryTemplateConfig {
    pub user_id: String,
    pub template_id: String,
    pub override_json: String,
    pub updated_at: TimestampMs,
}
