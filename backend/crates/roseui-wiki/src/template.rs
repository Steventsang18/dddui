//! Wiki starter templates.
//!
//! A template seeds a fresh knowledge base with a small set of example pages so
//! the user immediately sees how `[[links]]`, frontmatter (`category` /
//! `doc_type` / `status`) and MOC (Map of Content) pages work. Templates are
//! pure seed *data* — they never change the schema and the user can delete every
//! seeded page to fall back to a blank general-purpose notebook.
//!
//! Two modes ship today:
//! - `legal`  — a legal-domain starter (contract review, statutes, opinions)
//! - `blank`  — no seed pages; roseui acts as a plain local notebook

use crate::error::WikiError;

/// One seed page inside a template.
pub struct TemplatePage {
    pub title: String,
    pub content_md: String,
    pub category: String,
    pub doc_type: String,
    pub source_ref: String,
    pub status: String,
    pub extra_json: String,
}

/// A resolved template: its id and the pages to seed.
pub struct TemplateDef {
    pub id: &'static str,
    pub pages: Vec<TemplatePage>,
}

/// Resolve a template by id. Unknown ids fall back to `blank`.
pub fn template_def(id: &str) -> Result<TemplateDef, WikiError> {
    match id.trim().to_ascii_lowercase().as_str() {
        "legal" => Ok(legal_template()),
        _ => Ok(blank_template()),
    }
}

/// List available template ids (for the frontend chooser).
pub fn available_templates() -> Vec<&'static str> {
    vec!["legal", "blank"]
}

fn blank_template() -> TemplateDef {
    TemplateDef {
        id: "blank",
        pages: vec![],
    }
}

fn legal_template() -> TemplateDef {
    let moc = TemplatePage {
        title: "法律工作区首页".to_string(),
        content_md: "# 法律工作区\n\n欢迎使用 RoseUi 法律知识库。本页是内容地图（MOC），用 [[ ]] 串联关键资料：\n\n- 合同审查：[[合同审查要点索引]]\n- 常用法条：[[常用法条汇编]]\n- 示范意见：[[法律意见书模板]]\n\n> 提示：点击任意 [[页面名]] 可跳转；右侧「反向链接」能看到谁引用了当前页。".to_string(),
        category: "索引".to_string(),
        doc_type: "moc".to_string(),
        source_ref: String::new(),
        status: "定稿".to_string(),
        extra_json: "{}".to_string(),
    };
    let contract_index = TemplatePage {
        title: "合同审查要点索引".to_string(),
        content_md: "# 合同审查要点\n\n审查合同时逐项核对（参考 [[常用法条汇编]]）：\n\n1. 主体资格与签章\n2. 标的与数量\n3. 价款与支付\n4. 违约责任（参见 [[法律意见书模板]] 中的示范条款）\n5. 争议解决与管辖\n".to_string(),
        category: "合同法".to_string(),
        doc_type: "指引".to_string(),
        source_ref: String::new(),
        status: "定稿".to_string(),
        extra_json: "{\"scope\":\"review\"}".to_string(),
    };
    let statutes = TemplatePage {
        title: "常用法条汇编".to_string(),
        content_md: "# 常用法条\n\n- 《民法典》合同编相关条款\n- 违约责任认定标准\n\n> 本页为示例，请按实际适用的法律填充，并用 source_ref 记录条文编号。".to_string(),
        category: "民法".to_string(),
        doc_type: "法条".to_string(),
        source_ref: String::new(),
        status: "草稿".to_string(),
        extra_json: "{}".to_string(),
    };
    let opinion = TemplatePage {
        title: "法律意见书模板".to_string(),
        content_md: "# 法律意见书（模板）\n\n## 一、事实概述\n\n## 二、法律分析\n（依据 [[常用法条汇编]]）\n\n## 三、结论与建议\n\n> 状态：草稿，审阅后改为「审阅中」「定稿」。".to_string(),
        category: "实务".to_string(),
        doc_type: "意见书".to_string(),
        source_ref: String::new(),
        status: "草稿".to_string(),
        extra_json: "{}".to_string(),
    };
    TemplateDef {
        id: "legal",
        pages: vec![moc, contract_index, statutes, opinion],
    }
}
