# Developer Guide

项目约 1200 行代码，分 4 个 Rust 源文件 + 1 个 HTML 文件。

## 代码地图

```
文件（行数）                  职责
─────────────────────────────────────────
main.rs (190)                 HTTP 路由 + 跨平台终端启动
session_manager/mod.rs (93)   数据结构 + 删除编排
session_manager/claude.rs (304)   JSONL 扫描/解析/删除
session_manager/utils.rs (205)    时间解析、文本提取、消息渲染
static/index.html (435)       前端单页应用
```

## 关键设计决策

### 大文件优化
`read_head_tail_lines()` 对大文件（>16KB）只读头部和尾部：
- 头部 10 行：提取 sessionId、cwd、首条用户消息
- 尾部：seek 到最后 16KB，读 30 行，提取最后活跃时间、摘要、自定义标题

避免加载整个 JSONL 到内存。

### 标题优先级
```
customTitle > 首条用户消息 > 项目目录名 > sessionId 前 8 位
```
`customTitle` 来自 JSONL 尾部 `"type":"custom-title"` 行。

### 消息倒序
后端 `load_messages_paginated()` 读全文件后 `.rev()` 倒序，外层切片分页。文件通常 <10MB，性能可接受。

### 安全删除
两层校验：
1. `canonicalize()` 解析后验证 source path 在 `~/.claude/projects/` 下
2. 从目标文件 re-parse session ID 与请求参数比对

### 前端状态管理
单 HTML 文件，无框架。全局变量管理状态：
```js
let sessions = [];           // 会话列表
let selectedSession = null;  // 当前选中的会话
let messages = { items:[], total:0, offset:0, source:'' };  // 分页消息
let batchMode = false;       // 批量管理模式
let checkedSessions = new Set();
let collapsedFolders = new Set();
```

DOM 通过 `innerHTML` 字符串拼接渲染，无虚拟 DOM。

### 消息分页
初始加载 50 条最新消息，滚动到顶部触发 `loadMore()`。加载更早消息时记录 `scrollHeight`，插到 DOM 后恢复位置。

### 终端启动
`spawn()` 子进程后立即 `return Ok(())`，不等待进程退出。进程通过 `stdio::null()` 脱离，页面不感知其生命周期。

## 修改指南

### 添加新的消息类型渲染
编辑 `utils.rs` → `extract_text_from_item()`，添加新的 `if item_type == "your_type"` 分支。返回的字符串会被 marked 渲染为 HTML。

### 修改端口
编辑 `main.rs` 第 172 行 `"0.0.0.0:8888"`。

### 修改每页消息数量
前端 `index.html` 中的 `PAGE_SIZE = 50`，后端默认值在 `MessagesQuery` 的 `#[serde(default)]`。
