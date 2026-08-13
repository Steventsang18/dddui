-- 行业方案模板：用户所选模板 + 公司级覆盖（第②层）持久化。
-- 每个用户一行：激活哪个内置模板 + 一段 rupoo::industry_template::TemplateOverride 的
-- serde_json 序列化。override_json = '{}' 表示「无覆盖」，模板基线原样生效。
CREATE TABLE IF NOT EXISTS industry_template_config (
    user_id       TEXT PRIMARY KEY,
    template_id   TEXT NOT NULL,
    override_json TEXT NOT NULL DEFAULT '{}',
    updated_at    INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
