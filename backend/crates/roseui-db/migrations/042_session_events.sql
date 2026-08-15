-- 对话轨迹（事件溯源）：把一次会话中发生的每条可观测事件结构化落库，
-- 供「轨迹回放」按 日期时刻 + 输入 + 输出 + 模型 + 工具(分别做了什么) 查询。
--
-- 设计要点：
-- 1. append-only：事件只增不删（会话删除时随 conversations 级联清理）。
-- 2. 与 messages 表解耦：messages 存「展示用消息」，session_events 存「可审计轨迹」，
--    两者靠 conversation_id 关联，互不阻塞。
-- 3. 脱敏在写入层（stream_persistence / request_trace 落库处）完成，
--    本表只接收已脱敏后的 input_json/output_json/token_usage_json。
-- 4. turn_seq 用于同会话内稳定排序（同毫秒事件按写入顺序定序，避免回放乱序）。
CREATE TABLE IF NOT EXISTS session_events (
    id               TEXT    PRIMARY KEY NOT NULL,
    conversation_id  TEXT    NOT NULL,
    turn_seq         INTEGER NOT NULL,
    event_kind       TEXT    NOT NULL
                                CHECK(event_kind IN ('text', 'thinking', 'tool_call', 'model_call')),
    role             TEXT,
    model            TEXT,
    input_json       TEXT    NOT NULL DEFAULT '{}',
    output_json      TEXT    NOT NULL DEFAULT '{}',
    token_usage_json TEXT    NOT NULL DEFAULT '{}',
    status           TEXT    CHECK(status IN ('finish', 'pending', 'error', 'work')),
    created_at       INTEGER NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_session_events_conv_seq
    ON session_events(conversation_id, turn_seq);
CREATE INDEX IF NOT EXISTS idx_session_events_conv_created
    ON session_events(conversation_id, created_at);
CREATE INDEX IF NOT EXISTS idx_session_events_conv_kind
    ON session_events(conversation_id, event_kind);
