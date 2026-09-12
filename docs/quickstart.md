# Quickstart

本文档描述如何在本地把 Fluxora 完整跑起来：Gateway（WebSocket 路由）+ UI（Leptos/WASM）+ Chat 服务（示例业务）。

所有操作通过仓库根目录的 `x.nu`（Nushell 任务脚本）驱动，先进入 Nushell 并加载：

```nu
use x.nu
```

## 前置依赖

- Rust（wasm32-unknown-unknown target）
- [trunk](https://trunkrs.dev/)（UI 构建与服务）
- Nushell
- Docker（`$env.CNTRCTL` 指定容器运行时，默认行为按 docker/podman 通用参数书写）
- [websocat](https://github.com/vi/websocat)（调试 WS，可选）

## 一、消息队列（Redpanda）

Gateway 通过 Kafka 协议与 Redpanda 通信，`up` 会建好 gateway.toml 里声明的 topic：

```nu
x rpk up --external localhost
```

`--external` 是广播给容器外客户端的地址，本机开发用 `localhost`，容器内访问宿主机用 `host.docker.internal`。

## 二、Gateway

```nu
x gw up
```

监听 `localhost:3000`（WebSocket 入口 `/channel`）。代码热更：watch 模式在源码变更时自动重启进程。

## 三、UI

```nu
x ui up
```

即 `trunk serve`，跑在 `x.toml` 的 `[leptos].port`（默认 3001）。访问 `http://localhost:3001`，静态资源（apexcharts/mermaid/main.css）通过 trunk proxy 透传到 Gateway:3000。

生产构建用 `x ui build`（release + 输出 `dist/` 体积统计）。

## 四、Chat 服务（示例业务）

```nu
x pg up          # PostgreSQL 容器（数据落 data/postgres）
x pg migrate
x chat up        # cargo run --bin chat，监听 3003
```

## 验证

打开 `http://localhost:3001` 即渲染默认 layout。也可不经 UI 直接向 Gateway 投递消息：

```nu
x send 00.chat_layout.yaml   # data/message/ 下的样例消息
x send 02.concat.yaml        # 流式 concat 增量合并
x receiver                   # 查看当前 WS session
```

`x watch message` 监听 `data/message/` 目录，保存文件即推送，适合调 UI。

## 一键串起来

```nu
x serve            # gw up + ui up（--rpk 连消息队列一起起）
```

## 其他常用命令

| 命令 | 用途 |
|---|---|
| `x pg cli` | DuckDB ATTACH PostgreSQL 交互查询 |
| `x gw client` | websocat 直连 `/channel` 调试 |
| `x jsonschema` | 导出 Brick JSON Schema（AI 生成 JSON 的契约） |
| `x macro brick` / `x macro ui` | 跑 proc-macro 测试 |
| `x test benchmark <n>` | oha 压测（0=WS 管理面，1/2=chat API） |

## 架构速览

```
┌──────────┐   WS    ┌──────────┐  outgo   ┌─────────────────┐
│   UI     │◄───────►│ Gateway  │─────────►│ Business Service│
│ (Leptos) │         │ (Axum)   │          │ (Chat, CRM, AI…)│
└──────────┘         └────┬─────┘          └────────┬────────┘
                          │   income                │
                          └─────────────────────────┘
```

UI 与业务逻辑通过事件名解耦：组件声明 bind 到事件，Gateway 把 UI 事件投递到 `outgo` 队列，业务服务消费后把结果经 `income` 队列推回对应 WS session，UI 收到消息后按 `Brick` JSON 动态渲染。详细设计见 `README.md` 与 `docs/decisions/`。
