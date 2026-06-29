# API Reference

Base URL: `http://localhost:8888`

## 通用响应格式

```jsonc
// 成功
{ "providerId": "...", ... }

// 删除/恢复操作
{ "success": true }
{ "success": false, "error": "错误信息" }
```

---

## GET /api/sessions

列出所有 Claude Code 会话。

**响应**: `SessionMeta[]`

```typescript
interface SessionMeta {
  providerId: string;        // 固定 "claude"
  sessionId: string;         // UUID
  title?: string;            // 优先级: customTitle > 首条用户消息 > 项目目录名
  summary?: string;          // 最后一条非 meta 消息（截断到 160 字符）
  projectDir?: string;       // 会话所在工作目录
  createdAt?: number;        // Unix 毫秒
  lastActiveAt?: number;     // Unix 毫秒
  sourcePath?: string;       // JSONL 文件绝对路径
  resumeCommand?: string;    // "claude --resume <sessionId>"
}
```

**示例**:
```bash
curl http://localhost:8888/api/sessions
```

---

## GET /api/sessions/messages

分页获取会话消息，最新在前。

**参数**:

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| source | string | 必填 | JSONL 文件路径（URL 编码） |
| offset | number | 0 | 偏移量 |
| limit | number | 50 | 每页条数 |

**响应**: `PaginatedMessages`

```typescript
interface PaginatedMessages {
  total: number;             // 总消息数
  offset: number;            // 当前偏移
  limit: number;             // 每页条数
  messages: SessionMessage[];
}

interface SessionMessage {
  role: string;              // "user" | "assistant" | "tool" | "system"
  content: string;           // markdown 格式文本
  ts?: number;               // Unix 毫秒
}
```

**示例**:
```bash
curl "http://localhost:8888/api/sessions/messages?source=%2F~%2F.claude%2Fprojects%2F...jsonl&limit=10"
```

---

## DELETE /api/sessions

删除会话的 JSONL 文件和 sidecar 目录。

**参数**:

| 参数 | 类型 | 说明 |
|------|------|------|
| source | string | JSONL 文件路径 |
| session_id | string | 会话 UUID |

**响应**: `{ success: boolean, error?: string }`

**删除内容**:
- `~/.claude/projects/.../<session-id>.jsonl`
- `~/.claude/projects/.../<session-id>/`（子 agent + tool-results）

**安全检查**:
1. 验证 source 路径在 `~/.claude/projects/` 下
2. 验证文件内 sessionId 与参数匹配

**示例**:
```bash
curl -X DELETE "http://localhost:8888/api/sessions?source=%2Fpath%2Fto%2Ffile.jsonl&session_id=uuid"
```

---

## POST /api/resume

在终端中启动 `claude --resume`。

**参数**:

| 参数 | 类型 | 说明 |
|------|------|------|
| command | string | 完整 resume 命令 |
| cwd | string | 工作目录 |

**响应**: `{ success: boolean, error?: string }`

**终端探测顺序**:

| 平台 | 终端 |
|------|------|
| Linux | gnome-terminal → ghostty → konsole → xfce4-terminal → kitty → alacritty → wezterm → x-terminal-emulator → xterm |
| macOS | Terminal.app → iTerm2（通过 osascript） |
| Windows | cmd /c start → wt |

**示例**:
```bash
curl -X POST "http://localhost:8888/api/resume?command=claude%20--resume%20uuid&cwd=%2F~%2Fproject"
```
