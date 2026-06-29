# Architecture

## 整体架构

```
浏览器 (http://localhost:8888)
    │
    ▼
┌─────────────────────────────────────┐
│  src/static/index.html              │  单页应用（~435行）
│  · 会话列表（分组折叠）              │
│  · 消息查看（分页 + markdown）       │
│  · 删除/恢复/搜索                    │
└──────────────┬──────────────────────┘
               │ fetch /api/...
               ▼
┌─────────────────────────────────────┐
│  src/main.rs (Axum 0.7)             │  路由层（~190行）
│  · GET  /api/sessions                │
│  · GET  /api/sessions/messages       │
│  · DELETE /api/sessions              │
│  · POST /api/resume                  │
│  · ServeDir("src/static")            │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  session_manager/claude.rs          │  扫描器（~300行）
│  · scan_sessions()                   │
│  · load_messages()                   │
│  · load_messages_paginated()         │
│  · delete_session()                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│  session_manager/utils.rs           │  工具函数（~230行）
│  · parse_timestamp_to_ms()           │
│  · extract_text / _from_item()       │
│  · read_head_tail_lines()            │
│  · truncate_summary()                │
│  · path_basename()                   │
└─────────────────────────────────────┘
```

## 数据流

### 会话扫描

```
~/.claude/projects/<项目名>/<session-id>.jsonl
    │
    ├── read_head_tail_lines(path, head=10, tail=30)
    │       │
    │       ├── 头部：sessionId, cwd, createdAt, 首条用户消息
    │       └── 尾部：lastActiveAt, summary, customTitle
    │
    ├── 标题优先级: customTitle > 首条用户消息 > cwd basename
    ├── 过滤: agent- 前缀文件（子 agent 会话）
    └── 返回 SessionMeta
```

### 消息分页

```
请求: GET /api/sessions/messages?source=...&offset=0&limit=50
    │
    ├── load_messages(path) 读取全部 JSONL 行
    ├── 倒序排列（最新在前）
    ├── 切片 [offset : offset+limit]
    └── 返回 { total, offset, limit, messages }
```

### 消息渲染

```
JSONL 行 { message: { role, content } }
    │
    ├── content 为 String → 直接返回
    ├── content 为 Array → 逐项 extract_text_from_item()
    │       │
    │       ├── type=text → 返回文本
    │       ├── type=thinking → 引用块 "> 🤔 **思考过程**"
    │       ├── type=tool_use → 工具名 + JSON 参数
    │       └── type=tool_result → 输出内容（错误标记）
    │
    └── 前端用 marked.parse() 渲染 markdown
```

## 删除流程

```
DELETE /api/sessions?source=...&session_id=...
    │
    ├── 解析 session ID，与文件内容校验匹配
    ├── 验证 source 路径在 ~/.claude/projects/ 目录下
    ├── 删除 <session-id>.jsonl
    ├── 删除 <session-id>/ 同名目录（子 agent、tool-results）
    └── 返回 { success: true/false, error }
```

## 终端启动

```
POST /api/resume?command=claude --resume <id>&cwd=<path>
    │
    ├── Linux: gnome-terminal → ghostty → konsole → ... → xterm
    ├── macOS: osascript Terminal.app → iTerm2
    ├── Windows: cmd /c start → wt
    └── 返回 { success: true/false, error }
```

## 依赖关系

```
main.rs
  ├── axum (HTTP 路由)
  ├── tower-http (CORS + 静态文件)
  ├── tokio (异步运行时)
  └── session_manager/
        ├── claude.rs
        │     ├── serde_json (JSONL 解析)
        │     └── utils.rs
        │           └── chrono (RFC3339 时间解析)
        └── mod.rs
              ├── serde (序列化)
              └── dirs (获取 home 目录)
```
