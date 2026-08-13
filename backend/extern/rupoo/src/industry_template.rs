//! 行业方案模板 (Industry Solution Templates)
//!
//! 把 rupoo 已存在的散件（工具集、命令白名单、审批策略、审计脱敏、系统提示词）
//! 打包成可命名、可选择、可覆盖的垂直行业预设（法律/教育/医疗/金融）。
//!
//! 三层模型：
//!   ① 行业基线：模板预设（`builtin()`）；
//!   ② 公司覆盖：`TemplateOverride` → `apply()`；
//!   ③ 会话临时：审批「允许一次 AllowOnce」，落在 supervisor 层。
//!
//! 锁底线：`lock` 标记不可覆盖项，`apply()` 保证它们永远生效。

use serde::{Deserialize, Serialize};

use crate::config::{AgentProfile, SafetySection};

/// 一个行业方案的基线预设。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryTemplate {
    pub id: String,
    pub label: String,
    pub prompt: String,
    /// 允许的工具（白名单，default-deny）。
    pub allowed_tools: Vec<String>,
    /// 禁用的工具（叠加于 allowed_tools）。
    pub excluded_tools: Vec<String>,
    /// 命令白名单（shell_exec 内部的命令级最小权限）。
    pub allowed_commands: Vec<String>,
    pub forbidden_commands: Vec<String>,
    /// 审批策略（"always" / "dangerous_only"）。
    pub approval_policy: String,
    /// 直接放行的工具。
    pub auto_approve_tools: Vec<String>,
    #[serde(default)]
    pub lock: TemplateLock,
}

/// 锁底线 —— 覆盖无法突破的约束。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateLock {
    /// 永远禁用的工具。
    #[serde(default)]
    pub forced_excluded_tools: Vec<String>,
    /// 永远禁止的命令。
    #[serde(default)]
    pub forced_forbidden_commands: Vec<String>,
}

/// 用户/公司对模板的覆盖（第②层）。未提供的字段沿用模板。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateOverride {
    pub system_prompt: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub excluded_tools: Option<Vec<String>>,
    pub allowed_commands: Option<Vec<String>>,
    pub forbidden_commands: Option<Vec<String>>,
    pub approval_policy: Option<String>,
    pub auto_approve_tools: Option<Vec<String>>,
}

/// 把「模板 + 覆盖」展开成最终的 [`AgentProfile`] 与 [`SafetySection`]。
///
/// 规则：覆盖优先；锁底线兜底；excluded/forbidden 取「模板 ∪ 覆盖 ∪ 锁」。
pub fn apply(
    template: &IndustryTemplate,
    overrides: &TemplateOverride,
) -> (AgentProfile, SafetySection) {
    let forced_tools = &template.lock.forced_excluded_tools;
    let forced_cmds = &template.lock.forced_forbidden_commands;

    // 工具集
    let mut allowed = overrides
        .allowed_tools
        .clone()
        .unwrap_or_else(|| template.allowed_tools.clone());
    allowed.retain(|t| !forced_tools.contains(t));

    let mut excluded = overrides
        .excluded_tools
        .clone()
        .unwrap_or_else(|| template.excluded_tools.clone());
    for t in forced_tools {
        if !excluded.contains(t) {
            excluded.push(t.clone());
        }
    }

    // 命令级
    let mut allowed_commands = overrides
        .allowed_commands
        .clone()
        .unwrap_or_else(|| template.allowed_commands.clone());
    allowed_commands.retain(|c| !forced_cmds.contains(c));

    let mut forbidden = overrides
        .forbidden_commands
        .clone()
        .unwrap_or_else(|| template.forbidden_commands.clone());
    for c in forced_cmds {
        if !forbidden.contains(c) {
            forbidden.push(c.clone());
        }
    }

    // 直接放行的工具同样剔除锁定工具（工具根本不可用，放行无意义）。
    let mut auto_approve = overrides
        .auto_approve_tools
        .clone()
        .unwrap_or_else(|| template.auto_approve_tools.clone());
    auto_approve.retain(|t| !forced_tools.contains(t));

    let system_prompt = overrides
        .system_prompt
        .clone()
        .unwrap_or_else(|| template.prompt.clone());
    let approval_policy = overrides
        .approval_policy
        .clone()
        .unwrap_or_else(|| template.approval_policy.clone());

    let profile = AgentProfile {
        system_prompt: Some(system_prompt),
        label: Some(template.label.clone()),
        allowed_tools: Some(allowed),
        excluded_tools: Some(excluded),
    };

    let safety = SafetySection {
        approval_policy,
        auto_approve_tools: auto_approve,
        allowed_commands,
        forbidden_commands: forbidden,
        jail_root: ".".to_string(),
    };

    (profile, safety)
}

/// 从「模板 id + 覆盖 JSON」还原出最终配置（供持久化层闭环使用）。
///
/// `override_json` 为空或 `"{}"` 时视为「无覆盖」。解析失败返回错误字符串。
pub fn resolve(
    template_id: &str,
    override_json: &str,
) -> Result<(AgentProfile, SafetySection), String> {
    let template = get(template_id)
        .ok_or_else(|| format!("unknown industry template: {template_id}"))?;
    let overrides: TemplateOverride = if override_json.trim().is_empty() {
        TemplateOverride::default()
    } else {
        serde_json::from_str(override_json)
            .map_err(|e| format!("invalid template override json: {e}"))?
    };
    Ok(apply(&template, &overrides))
}

// ---------------------------------------------------------------------------
// 内置模板
// ---------------------------------------------------------------------------

/// 内置行业模板（不可变基线）。
pub fn builtin() -> Vec<IndustryTemplate> {
    vec![legal(), education(), medical(), finance()]
}

/// 按 id 获取内置模板。
pub fn get(id: &str) -> Option<IndustryTemplate> {
    builtin().into_iter().find(|t| t.id == id)
}

fn legal() -> IndustryTemplate {
    IndustryTemplate {
        id: "legal".into(),
        label: "法律合规助手".into(),
        prompt: "你是律所合规审查助手，遵循中国《个人信息保护法》等法规。只读分析，不得修改或删除任何文件，不得执行系统命令。".into(),
        allowed_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
            "web_search".into(),
        ],
        excluded_tools: vec![
            "shell_exec".into(),
            "file_write".into(),
            "file_edit".into(),
        ],
        allowed_commands: vec![],
        forbidden_commands: vec!["rm".into(), "curl".into(), "git push".into()],
        approval_policy: "always".into(),
        auto_approve_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
        ],
        lock: TemplateLock {
            forced_excluded_tools: vec!["shell_exec".into(), "file_write".into()],
            forced_forbidden_commands: vec!["rm".into()],
        },
    }
}

fn education() -> IndustryTemplate {
    IndustryTemplate {
        id: "education".into(),
        label: "教学辅导助手".into(),
        prompt: "你是教学辅导助手，帮助学生理解知识点、批改作业。可读写作业文件，但不得执行系统命令。".into(),
        allowed_tools: vec![
            "file_read".into(),
            "file_write".into(),
            "file_edit".into(),
            "list_directory".into(),
            "code_search".into(),
            "web_search".into(),
        ],
        excluded_tools: vec!["shell_exec".into()],
        allowed_commands: vec![],
        forbidden_commands: vec!["rm".into(), "sudo".into()],
        approval_policy: "dangerous_only".into(),
        auto_approve_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
        ],
        lock: TemplateLock {
            forced_excluded_tools: vec!["shell_exec".into()],
            forced_forbidden_commands: vec![],
        },
    }
}

fn medical() -> IndustryTemplate {
    IndustryTemplate {
        id: "medical".into(),
        label: "医疗数据助手".into(),
        prompt: "你是医疗数据助手，处理敏感个人健康信息。数据严禁出本机，只读分析，不得上传网络、不得执行系统命令、不得修改文件。".into(),
        allowed_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
        ],
        excluded_tools: vec![
            "shell_exec".into(),
            "file_write".into(),
            "file_edit".into(),
            "web_search".into(),
        ],
        allowed_commands: vec![],
        forbidden_commands: vec![
            "rm".into(),
            "curl".into(),
            "scp".into(),
            "git push".into(),
        ],
        approval_policy: "always".into(),
        auto_approve_tools: vec!["file_read".into(), "list_directory".into()],
        lock: TemplateLock {
            forced_excluded_tools: vec!["shell_exec".into(), "web_search".into()],
            forced_forbidden_commands: vec!["rm".into(), "scp".into()],
        },
    }
}

fn finance() -> IndustryTemplate {
    IndustryTemplate {
        id: "finance".into(),
        label: "金融风控助手".into(),
        prompt: "你是金融风控助手，处理交易与审计数据。可运行只读审计命令，但严禁修改、删除文件或执行危险命令。".into(),
        allowed_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
            "shell_exec".into(),
        ],
        excluded_tools: vec!["file_write".into(), "file_edit".into()],
        allowed_commands: vec![
            "ls".into(),
            "cat".into(),
            "grep".into(),
            "find".into(),
            "head".into(),
            "tail".into(),
            "wc".into(),
        ],
        forbidden_commands: vec![
            "rm".into(),
            "sudo".into(),
            "curl".into(),
            "git push".into(),
        ],
        approval_policy: "always".into(),
        auto_approve_tools: vec![
            "file_read".into(),
            "list_directory".into(),
            "code_search".into(),
        ],
        lock: TemplateLock {
            forced_excluded_tools: vec!["file_write".into(), "file_edit".into()],
            forced_forbidden_commands: vec!["rm".into(), "sudo".into()],
        },
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_templates_exist() {
        assert_eq!(builtin().len(), 4);
        for id in ["legal", "education", "medical", "finance"] {
            assert!(get(id).is_some(), "missing template: {id}");
        }
    }

    #[test]
    fn test_legal_locks_shell_against_override() {
        let t = get("legal").unwrap();
        // 覆盖试图把 shell_exec 加回白名单、并从黑名单移除。
        let o = TemplateOverride {
            allowed_tools: Some(vec!["shell_exec".into(), "file_read".into()]),
            excluded_tools: Some(vec![]),
            ..Default::default()
        };
        let (profile, safety) = apply(&t, &o);

        let allowed = profile.allowed_tools.unwrap();
        let excluded = profile.excluded_tools.unwrap();
        // shell_exec 绝不能在白名单，且必须在黑名单（双重封死）。
        assert!(!allowed.contains(&"shell_exec".to_string()));
        assert!(excluded.contains(&"shell_exec".to_string()));
        // file_read 正常保留。
        assert!(allowed.contains(&"file_read".to_string()));
        // rm 绝不能在命令白名单，且必须在命令黑名单。
        assert!(!safety.allowed_commands.contains(&"rm".to_string()));
        assert!(safety.forbidden_commands.contains(&"rm".to_string()));
    }

    #[test]
    fn test_override_can_change_unlocked_fields() {
        let t = get("legal").unwrap();
        let o = TemplateOverride {
            approval_policy: Some("dangerous_only".into()),
            system_prompt: Some("公司自定义提示词".into()),
            ..Default::default()
        };
        let (profile, safety) = apply(&t, &o);
        assert_eq!(safety.approval_policy, "dangerous_only");
        assert_eq!(profile.system_prompt.as_deref(), Some("公司自定义提示词"));
    }

    #[test]
    fn test_finance_command_whitelist_locked() {
        let t = get("finance").unwrap();
        // 覆盖试图把 rm 加进命令白名单。
        let o = TemplateOverride {
            allowed_commands: Some(vec!["rm".into(), "ls".into()]),
            ..Default::default()
        };
        let (_, safety) = apply(&t, &o);
        assert!(!safety.allowed_commands.contains(&"rm".to_string()));
        assert!(safety.allowed_commands.contains(&"ls".to_string()));
        assert!(safety.forbidden_commands.contains(&"rm".to_string()));
    }

    #[test]
    fn test_resolve_from_empty_json() {
        let (profile, safety) = resolve("legal", "{}").unwrap();
        assert_eq!(profile.label.as_deref(), Some("法律合规助手"));
        assert!(!profile.allowed_tools.unwrap().contains(&"shell_exec".to_string()));
        assert_eq!(safety.approval_policy, "always");
    }

    #[test]
    fn test_resolve_override_json_applies() {
        let (profile, safety) = resolve(
            "legal",
            r#"{"approval_policy":"dangerous_only","system_prompt":"公司自定义"}"#,
        )
        .unwrap();
        assert_eq!(safety.approval_policy, "dangerous_only");
        assert_eq!(profile.system_prompt.as_deref(), Some("公司自定义"));
        // 锁定底线仍生效。
        assert!(!profile.allowed_tools.unwrap().contains(&"shell_exec".to_string()));
    }

    #[test]
    fn test_resolve_unknown_template_errors() {
        assert!(resolve("nope", "{}").is_err());
    }

    #[test]
    fn test_resolve_invalid_json_errors() {
        assert!(resolve("legal", "not json").is_err());
    }

    #[test]
    fn test_override_can_add_excluded_tool() {
        let t = get("education").unwrap();
        // 教育模板未锁定 echo，覆盖可额外禁用 echo。
        let o = TemplateOverride {
            excluded_tools: Some(vec!["echo".into()]),
            ..Default::default()
        };
        let (profile, _) = apply(&t, &o);
        let excluded = profile.excluded_tools.unwrap();
        assert!(excluded.contains(&"echo".to_string()));
        // 锁定的 shell_exec 仍在。
        assert!(excluded.contains(&"shell_exec".to_string()));
    }
}
