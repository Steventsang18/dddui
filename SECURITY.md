# 安全政策 (Security Policy)

## 本地优先 / 数据不出本机

DoDidDoneUi 是**本地运行的私有平台**：所有数据（对话、知识库、配置、API Key）存储在运行本机的
SQLite 数据库中，默认不向任何外部服务发送。模型 API 调用由用户自备 Key，直接发往用户配置的模型
服务商，不经过本项目服务器。

## 默认安全配置

- 后端默认仅绑定 `127.0.0.1`（回环地址），不对外暴露。
- 跨源请求（CORS）在生产同源模式下不启用；开发模式仅白名单 `http://localhost:5173`。
- 保留 JWT + CSRF 双提交 Cookie 机制；校验 `Host` 请求头防御 DNS Rebinding；WebSocket 握手校验 Origin。
- 遥测默认关闭。

## 报告漏洞

**请勿在公开 Issue 中披露安全漏洞。**

请通过以下方式私报告：
- 邮件至安全联系人（见仓库 Releases 页或维护者主页）
- 或 GitHub 的 **Private Vulnerability Reporting**（Security → Report a vulnerability）

我们会尽快确认、修复并协调披露时间。

## 已知边界

本平台具备执行本地 shell 命令与读写本地磁盘的能力（用于 Agent 工具）。请仅在受信任的本地环境使用，
不要将端口暴露到不可信网络。
