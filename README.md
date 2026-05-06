# dataspl

一个基于 `axum` 的演示服务，展示了如何构建：

- 使用 `tokio` 和 `axum` 提供 HTTP 服务
- 通过 Server-Sent Events (SSE) 支持流式 AI 响应
- 与 DeepSeek LLM API 集成的聊天与技能执行
- 从 `skills/` 目录加载 Markdown 格式的技能定义
- 提供 REST API 和静态前端页面

## 功能概览

- `POST /api/login` - 简单登录接口
- `GET /api/verify` - Token 验证接口
- `POST /api/chat` - 聊天接口，支持与 LLM 的交互
- `POST /api/generate-aispl` - 生成 AiSPL 脚本
- `GET /api/health` - 健康检查
- `GET /api/skills` - 列出已加载技能
- `POST /api/skills/execute` - 执行指定技能
- 静态资源托管：`static/` 目录，默认返回 `static/index.html`

## 项目结构

- `src/main.rs` - 程序入口，初始化日志、LLM 客户端和 Axum 路由
- `src/router.rs` - 路由定义与静态文件回退
- `src/handler.rs` - 处理 API 请求和 SSE 响应
- `src/llm.rs` - DeepSeek LLM 客户端与流式响应解析
- `src/state.rs` - 应用状态管理和 token 驱动的简单验证
- `src/skill/` - 技能加载、管理与执行逻辑
- `skills/` - Markdown 技能定义目录
- `static/` - 静态前端页面与资源

## 依赖

该项目使用以下主要依赖：

- `axum`
- `tokio`
- `tower-http`
- `serde`, `serde_json`
- `reqwest`
- `tracing`, `tracing-subscriber`
- `regex`

## 快速开始

1. 安装 Rust 工具链：

```powershell
rustup toolchain install stable
```

2. 设置 DeepSeek API Key：

```powershell
$env:DEEPSEEK_API_KEY = 'your-api-key-here'
```

或永久设置：

```powershell
setx DEEPSEEK_API_KEY "your-api-key-here"
```

3. 运行服务：

```powershell
cd e:\RustProjects\axumdemo
cargo run
```

4. 打开浏览器访问：

```text
http://127.0.0.1:3000
```

## 技能定义格式

本项目会在启动时加载 `skills/` 目录下所有 `.md` 文件，并将其解析为可执行技能。

一个典型技能文件应包含 YAML frontmatter，用于定义 ID、名称、描述和版本；以及 `prompt` 代码块，用于指定 LLM 调用时的模板。

示例：

```text
---
id: example-skill
name: 示例技能
description: 将输入文本转换为大写
version: 1.0.0
---

~~~prompt
请将以下文本转换为大写：{input}
~~~
```

可选项：

- `examples`：Markdown 中可包含示例说明，当前实现会解析 `### Example` 样式示例
- `context`：执行时可传入上下文变量替换模板中的 `{key}` 占位符

## 运行时行为

- 启动时会从 `skills/` 目录自动加载技能，加载失败会写入日志但不会终止服务。
- `AppState` 管理简单 token 过期验证，并维护 LLM 客户端与技能容器。
- 所有静态资源均通过 `tower_http::services::ServeDir` 从 `static/` 提供。

## 开发与调试

- 日志通过 `tracing` 输出，默认环境过滤器为 `axumdemo=info,tower_http=info`
- 可通过设置环境变量 `RUST_LOG` 自定义日志级别，例如：

```powershell
$env:RUST_LOG = 'axumdemo=debug,tower_http=debug'
cargo run
```

## 扩展建议

- 增加技能热加载逻辑
- 增加更完善的认证与授权
- 支持更多 LLM 提供商
- 添加前端 UI 交互增强和错误提示

## 许可证

当前仓库未指定许可证，请根据实际需求补充 `LICENSE` 文件。
