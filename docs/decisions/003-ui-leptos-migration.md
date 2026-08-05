# ADR 003: UI 从 Dioxus 迁移到 Leptos（builder API + 宏生成）

**日期**: 2026-08-04
**状态**: 提议
**影响范围**: `crates/ui`（现有 Dioxus 前端）、`crates/ui_macro`、新增 `crates/ui-leptos` 与 `crates/ui_leptos_macro`

## 背景

`crates/ui` 目前基于 **Dioxus 0.7.9**（WASM/web），通过 JSON DSL（`Brick`）驱动动态 UI。迁移诉求：

1. **不需要多端渲染**：Dioxus 的 desktop/mobile 能力是纯负担，实际只用 web。
2. **dioxus-cli 版本耦合**：`dx` 工具必须与 `dioxus` crate 版本一致，每次升级都会互相破坏。倾向用标准化工具（`trunk`）。
3. **不喜欢 HTML 模拟 DSL**：`view!`/`rsx!` 的 `<div>` 模板风格不合口味。Yew / Leptos 默认 `view!` 是 HTML 风格（不考虑）；Sycamore 前景区间于"可维护但不确定"。

筛选后落到 **Leptos**：活跃维护、生态成熟、纯 CSR 可走 `trunk`（不碰 `cargo-leptos`）、且其 **builder API**（`div().child(...).on(...)`）是普通 Rust 函数链，**非 HTML DSL**，可满足第 3 条。

## 关键洞察：builder API 满足"不用 HTML DSL"

Leptos 的 `view!` 展开后本质就是 builder 语法（`view!` 里的 `div().child(...)` 是它的直接展开）。因此：

- **`view!` 与 builder 是同一层**，builder 是底层 Rust 表达式。
- 我们**不需要 `view!`**，直接用 builder API 写组件。
- 这正是"不用 HTML DSL、又不重造框架"的结合点：响应式、事件、生命周期、keyed diff 全归 Leptos，DOM 构建由 builder 链完成。

> 反向方案"宏直出 web_sys + 自写信号"等于重造一个前端框架（事件样板、`Closure` 泄漏、状态容器、列表 diff、XSS 都要自己补），**否决**。

## 为什么用 Leptos 而非 Dioxus：更轻量

除了工具链与 DSL 诉求，Leptos 在**运行时与体积**上比 Dioxus 更轻，且这个"轻"正好对应"只要 web"的诉求：

| 维度 | Dioxus 0.7 | Leptos 0.8 | 谁更轻 |
|---|---|---|---|
| 渲染模型 | 内核仍保留虚拟 DOM（VDOM）diff | 纯细粒度响应式，**无 VDOM**，直接操作 DOM | Leptos |
| wasm 体积 | 较大（hello world 未压缩 ~200-400KB） | 较小（~100-200KB） | Leptos |
| 依赖树 | `dioxus-core`/`signals`/`html`/`hooks` + desktop/mobile 可选 | `leptos_dom` + `reactive_graph`，纯 web 依赖更少 | Leptos |
| 运行时内存 / 大列表更新 | 有 VDOM diff 开销 | 只更新变化的信号，无 reconciliation | Leptos |
| 编译时间 | `rsx!` 宏较重 | `view!` 宏也重，通常略快 | Leptos（轻微） |

关键点：

- **核心差异是"无 VDOM"**：Leptos 编译期就知道每个信号绑定到哪个 DOM 节点，只更新变化部分；Dioxus 0.7 虽已混合信号，但内核仍跑 VDOM reconciliation。
- **对你们场景影响有限**：渲染逻辑是 `Brick` 树 → 组件，框架运行时只是总大小的一部分。Dioxus 的额外负担来自 **desktop/mobile 支持 + VDOM**——你正好不需要，这部分是纯浪费；Leptos 纯 web、无 VDOM，正好贴合。
- **"更轻"不等于"更简单"**：Leptos 细粒度响应式对新手更陡，但你们用 builder API + 宏生成，这部分心智由宏承接。

## 架构设计

### 1. 保持"文件组件 + 宏只做分发"

现有 `ui_macro::gen_dispatch!` 已经成熟：读 `brick` 源码 → 生成 `match` 分发 → 每个分支调用一个**文件组件**（`widgets/*.rs` / `components/*.rs`）。迁移时保留此结构，只把：

- 组件输出从 `Element`（`rsx!` 结果）改为 `View`（builder 链结果）
- 宏生成的 `rsx!(#var_ { ... })` 改为 `crate::components::#var_(c, ctx, id)` 函数调用

### 2. 递归责任归属

- **叶子组件**（`button_`/`input_`/`text_`/`image_`/`select_`/`table_` 等）：不递归，返回单个 `View`。
- **容器组件**（`case_`/`rack_`/`fold_`/`float_`/`popup_`/`form_`/`svg_`/`group_`/`placeholder_`/`chart_`/`diagram_`）：递归 `sub`，内部调用 `crate::render::dispatch`。

宏生成的 `dispatch` 只分派到这两个角色，不内联组件逻辑。

### 3. 宏生成的代码形态

```rust
// ui_leptos_macro 展开后（render.rs）
pub fn dispatch(brick: &Brick, ctx: &Crate::Ctx) -> leptos::prelude::View {
    match brick {
        Brick::case(c)  => crate::components::case_(c, ctx),
        Brick::button(c) => crate::components::button_(c, ctx, id), // 有 has_id 的额外传 id
        // ...
    }
}
```

`has_id` 变体（`#[render_brick(has_id = "true")]`）由宏生成回退 id 计数器（沿用现有 `format!("{tag}-{}", n)` 逻辑），传给组件用于 chart/mermaid/rack 滚动等。

### 4. 状态容器 `Ctx`

把现有 `Status`（Dioxus `Signal` 集合）改为 `Ctx`：

```rust
#[derive(Clone)]
pub struct Ctx {
    pub ws: WebSocketHandle,          // gloo-net WS（框架无关，直用）
    pub codec: ActiveCodec,           // message::codec
    pub layout: RwSignal<Brick>,      // 顶层布局
    pub data: RwSignal<HashMap<String, Brick>>,
    pub list: RwSignal<HashMap<String, Vec<Brick>>>,
}
```

- `RwSignal` 是 Leptos 的响应式信号（`.get()`/`.set()`/`.update()`）。
- WS 收消息 → `Effect` 订阅 → `dispatch_msg`（复用现有 `store.rs` 的 `Create/Set/Join/Tmpl` 逻辑，含 minijinja 模板渲染与 merge）。
- 组件间通信通过共享 `Ctx` 信号，不需组件引用。

### 5. 模块结构

```
crates/ui-leptos/
├─ Cargo.toml
├─ index.html
├─ trunk.toml
└─ src/
   ├─ lib.rs            # mount 入口
   ├─ ctx.rs            # Ctx 状态容器 + dispatch_msg
   ├─ render.rs         # gen_dispatch! 生成的 dispatch
   ├─ ws.rs             # WebSocketHandle（gloo-net）
   ├─ dom.rs            # 轻量 web_sys 助手（chart/mermaid/滚动用）
   ├─ components/       # 容器组件（case/rack/fold/float/popup/form/svg/group/placeholder/chart/diagram）
   └─ widgets/          # 叶子组件（button/image/input/select/table/text/textarea）

crates/ui_leptos_macro/
├─ Cargo.toml
└─ src/
   ├─ lib.rs            # gen_dispatch!（读 brick 源码，生成 builder 分发）
   └─ attrs.rs          # 解析 file/entry/object 参数
```

## 现有组件 → Leptos 映射

| 现有 Dioxus | Leptos（builder） |
|---|---|
| `rsx!` / `Element` | builder 链（`div().child(...).on(...)`）返回 `View` |
| `#[component]` | 普通函数 `fn xxx_(brick: &Xxx, ctx: &Ctx) -> View` |
| `use_signal` / `Signal{read,write}` | `RwSignal`（`.get()/.set()/.update()`） |
| `Global` 静态信号 | 根组件 `provide_context` + `use_context` |
| `use_memo` / `use_effect` | `Memo` / `Effect` |
| `use_resource` | `Resource` |
| `Event<FormData>` / `KeyboardData` | `web_sys::InputEvent` / `KeyboardEvent`（builder `on:input` 等） |
| `document::eval`（chart/mermaid/滚动） | `web_sys::Window::eval`（`dom.rs` 助手） |
| `document::Style/Script` + `asset!` | `index.html` 静态 `<link>/<script>` + trunk |
| `dangerous_inner_html`（markdown） | `NodeRef` + `set_inner_html` |
| `RenderError` / `CapturedError` | `View` 无错误类型，改走空视图/`NodeRef` |
| `key` prop / 列表 | `Each`/`Indexed`（builder 内 `Keyed`） |
| `spawn`（local） | `leptos::prelude::spawn_local` |
| `dioxus::logger::tracing` | `tracing-wasm` 直连 |
| `dx serve` / `dx build --web` | `trunk serve` / `trunk build` |

## 构建与工具链

- **`trunk`**：标准 web 打包工具，无版本耦合。`trunk.toml` 配置资源代理（apexcharts/mermaid 代理到后端，对应原 `Dioxus.toml` 的 `[[web.proxy]]`）。
- `x.nu` 的 `ui` 模块相应更新：`dx serve` → `trunk serve --port <port>`，`dx build --web` → `trunk build --release`。
- 依赖 `brick` 关闭 `dioxus` feature（`default-features = false` + `features = ["classify","merge","ops","render"]`），避免 `BindVariant` 里的 `Signal<Value>` 字段依赖 dioxus。

## 实施步骤

1. 搭建 `crates/ui-leptos` 与 `crates/ui_leptos_macro` 骨架，注册进 workspace。
2. 实现 `Ctx` 状态容器 + `ws.rs`（复用 gloo-net）。
3. 实现 `ui_leptos_macro::gen_dispatch`（builder 分发）。
4. 实现文件组件（容器 + 叶子），返回 builder `View`。
5. 配置 `trunk` + `index.html`，接入 `main.rs`。
6. 安装 wasm32-unknown-unknown 目标 + trunk，`cargo check`/`trunk build` 验证。
7. 对照原 `crates/ui` 逐组件迁移，补齐交互（输入、列表 merge、chart/mermaid/滚动）。

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| Leptos builder API 细节（`HtmlElement`/`Attr` 类型）与 0.8 版本差异 | 先做最小可编译 PoC（`case_` + `text_` + `button_`）验证 API |
| 原生 `view!` 移除后模板能力（如 `dangerous_inner_html`）需手动 | 用 `NodeRef` + `set_inner_html` 封装 |
| WS 消息在 `Effect` 中订阅，异步时序需验证 | 先跑通 `create` 布局，再迭代 `set`/`join` |
| 列表 keyed diff 性能 | 用 Leptos `Each`/`Keyed`，避免全量重建 |
| 与 `brick` 现有 `dioxus` feature 解耦 | 已验证 `BindVariant` 的 `signal` 字段 `#[cfg(feature="dioxus")]`，关闭后正常 |

## 备选方案对比

| 方案 | 满足不用 HTML DSL | 不重造框架 | 工具链 | 轻量 | 前景 |
|---|---|---|---|---|---|
| **Leptos builder（采纳）** | ✅ | ✅ | trunk（标准） | 轻（无 VDOM） | 活跃/生态大 |
| Sycamore builder | ✅ | ✅ | trunk（标准） | 轻（无 VDOM） | 社区小、前景不确定 |
| 宏直出 web_sys + 自写信号 | ✅ | ❌（重造框架） | trunk | 最轻（无框架） | 自维护成本高 |
| 继续 Dioxus | ❌ | — | dioxus-cli（版本耦合） | 较重（VDOM + 多端） | 活跃 |

**结论：采纳 "Leptos builder API + 宏生成 dispatch" 方案。**