# Claude Session Manager

管理 Claude Code 会话记录的 Web 应用。

从 [cc-switch](https://github.com/farion1231/cc-switch) 的会话扫描逻辑移植而来。

## 快速启动

```bash
cargo run
# 浏览器打开 http://localhost:8888
```

## 功能

- 会话列表 — 按项目目录分组，可折叠/展开
- 搜索 — 匹配标题、摘要、项目名、sessionId
- 消息查看 — markdown 渲染 + 分页滚动加载
- 思考过程 — 引用块展示 LLM thinking
- 工具调用 — 显示参数 JSON + 输出内容
- 删除 — 单个/批量，删除前路径安全校验
- 恢复 — 点击按钮在终端中启动 `claude --resume`
- 侧边栏拖拽 — 调宽度 + j/k 键盘快捷键
- 暗色模式 — 跟随系统

## 技术栈

Rust + Axum 0.7 + Tokio 1，前端单页 HTML（marked.js CDN 渲染 markdown）。

## 项目结构

```
src/
├── main.rs                # Axum 服务器 + 4 个 API + 跨平台终端启动
├── session_manager/
│   ├── mod.rs             # SessionMeta/SessionMessage 结构，路由编排
│   ├── claude.rs          # Claude Code 扫描器（~/.claude/projects/）
│   └── utils.rs           # 时间解析、文本提取、tool_use/thinking 渲染
└── static/
    └── index.html         # 前端 UI（单页应用）
```

## API

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/sessions` | 列出所有会话 |
| GET | `/api/sessions/messages?source=...&offset=0&limit=50` | 分页获取消息（最新在前） |
| DELETE | `/api/sessions?source=...&session_id=...` | 删除会话 |
| POST | `/api/resume?command=...&cwd=...` | 在终端中恢复会话 |

## 删除行为

| 操作 | 删除的内容 |
|------|-----------|
| 删除会话 | `~/.claude/projects/<项目>/<session-id>.jsonl` |
| | 同名 sidecar 目录 `<session-id>/`（子 agent + tool results） |

删除前校验路径在合法目录下且 session ID 匹配。`claude --resume` 将不可用。
