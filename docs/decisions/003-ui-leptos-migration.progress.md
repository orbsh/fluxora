# ui-leptos 迁移进度（Handoff）

> 目标：继续 ADR 003（`docs/decisions/003-ui-leptos-migration.md`）——把 `crates/ui`（Dioxus）迁移到 `crates/ui-leptos`（Leptos 0.8 builder API + `crates/ui_leptos_macro` 宏生成 dispatch）。
> 日期：2026-08-04 · 状态：**进行中（未编译通过）**

## 一、当前命令行入口（重要）

- **sccache 在沙箱里是坏的**：所有 cargo 命令必须带 `RUSTC_WRAPPER=`（或 `SCCACHE_DISABLE=1`）。
- **cargo 需要网络**（拉取构建依赖），必须用 `sandbox_permissions: "require_escalated"`。
  - 已批准前缀规则：`["cargo", "check"]`、`["cargo", "update"]`。
- trunk 已装：`/nix/store/ca3ycph0454lcqgj2paq2s33dbwc51gf-system-path/bin/trunk`
- **wasm32-unknown-unknown 目标未安装** —— 后续 `trunk build` 前需先装（`rustup target add wasm32-unknown-unknown`）。

## 二、已完成的构建修复

### 1. `crates/ui-leptos/Cargo.toml`
- `brick` 依赖从 workspace 继承改为**直接 path**（workspace 继承无法覆盖 `default-features`）：
  ```toml
  brick = { path = "../brick", default-features = false, features = ["classify", "merge", "ops", "render"] }
  ```
  > 关闭 `dioxus` feature，避免 `BindVariant` 里的 `Signal<Value>` 字段（`#[cfg(feature="dioxus")]`）依赖 dioxus。

### 2. `crates/ui_leptos_macro/src/lib.rs`
- `syn::Option` → `Option`（第 66 行）
- `PathBuf` 不实现 `ToTokens`：改为 `let path_str = path.to_string_lossy().into_owned();` 并用 `include_bytes!(#path_str)`。
- `has_id` 分支的语句包进 `{ ... }` 花括号（原先生成非法代码）。
- `dispatch` 返回类型改为 `leptos::prelude::AnyView`（原 `View` 需要泛型）。

### 3. 构建状态
- `cargo check -p ui_leptos_macro` ✅ **通过**
- `cargo check -p ui-leptos` ❌ **失败**（见第四节错误清单）

## 三、关键 Leptos 0.8 API 事实（已在 registry 源码核实）

- 版本：`leptos 0.8.20`、`leptos_dom 0.8.8`、`tachys 0.2.18`、`reactive_graph 0.2.14`
- Builder 链：`div().class(...).id(...).style(...).child(...).on(ev::click, cb).inner_html(...).into_any()`
- 需要的 trait：`ClassAttribute`、`GlobalAttributes`、`StyleAttribute`、`OnAttribute`、`ElementChild`、`InnerHtmlAttribute`、`IntoAny`、`IntoRender`、`Render`（`leptos::prelude::*`）
- 事件描述符来自 `tachys/src/html/event.rs`：`ev::click`、`ev::input`、`ev::keydown`；事件类型为 `web_sys::MouseEvent`、`web_sys::InputEvent`、`web_sys::KeyboardEvent`
- **响应式闭包**：任何 `FnMut() -> V + Send + 'static`（`V: Render`）都实现 `Render`（`tachys/src/reactive_graph/mod.rs` 的 `ReactiveFunction`）。所以 `move || dispatch(...)` 就是一个响应式闭包，**根上不需要 `view!`**。
- `mount_to(parent: web_sys::HtmlElement, f: FnOnce() -> N)`：需先 query DOM 元素并用 `dyn_into::<web_sys::HtmlElement>()` 转换（`leptos/src/mount.rs` line 175）。
- `RwSignal<T, LocalStorage>`（`RwSignal::new_local`）用于非 Send 类型（如 `Rc<RefCell<SplitSink>>`）。
- `For` 组件（`leptos::prelude`）用于 keyed 列表。
- `spawn_local` 在 `leptos::lib::spawn_local`（prelude 里经 `leptos::*` 可用）。
- `provide_context`/`use_context` 在普通函数里可用（`reactive_graph/src/owner/context.rs`：`provide_context<T: Send + Sync + 'static>`、`use_context<T: Clone + 'static>() -> Option<T>`）。
- `IntoView` 为 `T: Sized + Render + RenderHtml + Send`（`leptos/src/into_view.rs`）。`AnyView` 满足，故 `dispatch` 返回 `AnyView` 可直接被 `mount_to` 的闭包消费。

## 四、`cargo check -p ui-leptos` 当前错误（待修）

1. `E0583`：`components` 模块不存在（`src/lib.rs` 里 `pub mod components;`）
2. `E0583`：`widgets` 模块不存在（`src/lib.rs` 里 `pub mod widgets;`）
3. `E0255`：`render.rs` 里 `dispatch` 重复定义——宏已生成 `pub fn dispatch`，`pub use dispatch;` 是多余的，删掉即可。
4. `E0107`：`ctx.rs:149` `render_brick` 返回类型 `leptos::prelude::View` 缺泛型 → 改 `AnyView`。
5. `E0599`/`E0277`（ws.rs）：`RwSignal<Rc<RefCell<SplitSink>>>` 不满足 Send/Sync → 用 `RwSignal::new_local`（`LocalStorage`）或直接 `Rc<RefCell<SplitSink>>`。
6. `E0425`：`spawn_local` 未找到 → 从 `leptos::prelude` 引入。
7. `E0308` 若干：`ctx.rs` 里 `dispatch_msg` 的借用问题 + `render_brick` 返回类型。

## 五、关键设计问题（必须解决）

### 1. `BindVariant::signal` 字段被 cfg 掉（非 dioxus 构建）
`brick/src/lib.rs` 约 99-110 行：`Submit { signal }` 和 `Field { signal }` 字段带 `#[cfg(feature = "dioxus")]`。
由于 ui-leptos 构建 brick **不带 dioxus**，这些字段**不存在**。依赖它们的组件（button_/input_/form_）必须改用**本地 `RwSignal`** 或通过 `provide_context` 共享信号。

### 2. `components` 模块缺失（所有文件）
`src/lib.rs` 引用了 `pub mod components;` 和 `pub mod widgets;`，但都不存在。宏分派到 `crate::components::#comp`，所以 `components/mod.rs` 必须 re-export 所有 widget 函数（`pub use crate::widgets::*;`）并定义容器组件。

### 3. 现有骨架文件需重写
| 文件 | 问题 | 处理 |
|---|---|---|
| `src/lib.rs` | mount 是坏的 | 全文重写（URL 解析 + 响应式 mount） |
| `src/ctx.rs` | `View` 泛型、`render_brick` 返回类型、`dispatch_msg` 借用 | 修 `AnyView`、加 `render_children` 助手 |
| `src/ws.rs` | `RwSignal<Rc<RefCell<SplitSink>>>` 不满足 Send/Sync | 用 `RwSignal::new_local` 或直接 `Rc<RefCell<SplitSink>>` |

## 六、组件签名（宏约定）

- **has_id 变体**（宏里 `(c, ctx, id: String)`）：placeholder、chart、diagram、fold、rack
- **非 has_id**（宏里 `(c, ctx)`）：case、float、form、popup、svg、group、path、button、image、input、select、table、thead、tbody、tr、th、td、text、textarea、render
- 所有组件返回 `leptos::prelude::AnyView`。

## 七、下一步（按顺序）

1. 重写 `ws.rs`（`RwSignal<T, LocalStorage>` 或直接 `Rc<RefCell<SplitSink>>`）
2. 重写 `ctx.rs`（`render_brick` → `AnyView`，修 `dispatch_msg` 借用，加 `render_children` 助手）
3. 创建 `hooks.rs`（从 `crates/ui/src/libs/hooks.rs` 移植，`use_context::<Status>()` → `&Ctx` 参数）
4. 创建 `widgets/`（6 个文件 + `widgets/mod.rs`）
5. 创建 `components/`（约 10 个文件 + `components/mod.rs`，re-export widgets）
6. 重写 `lib.rs`（mount：URL 解析 + 响应式根闭包）
7. 加 `main.rs`、`index.html`、`trunk.toml`（参考 `crates/ui/src/main.rs` + `index.html`）
8. 迭代 `RUSTC_WRAPPER= cargo check -p ui-leptos`（escalated）直至干净
9. 装 wasm32 目标 + `trunk build` 验证

## 八、参考：Dioxus `main.rs` 的 URL 解析（需移植进 mount）

- host 来自 `#main[data-host]` 属性或 `location.host()`
- token 来自 `?token=` URL 参数或 `data-token`
- codec 来自 `?codec=`（默认 CBOR）
- 拼出 `ws://{host}/channel?token=...&codec=...`
- 所有逻辑必须移植进 `mount()`。

## 九、Ctx 结构（ctx.rs 已有，保留）

```rust
pub struct Ctx {
    pub ws: WebSocketHandle,
    pub codec: ActiveCodec,
    pub layout: RwSignal<Brick>,
    pub data: RwSignal<HashMap<String, Brick>>,
    pub list: RwSignal<HashMap<String, Vec<Brick>>>,
}
```
- `Ctx::new()` 目前硬编码 `ws://localhost:3000/channel` → 改为接收 `url` + `ActiveCodec` 参数。
- `dispatch_msg` 逻辑在 ctx.rs 已从 `store.rs` 移植（Create/Set/Join/Tmpl），基本可用。

## 十、Brick 枚举变体（`crates/brick/src/lib.rs`）

`case, placeholder, chart, diagram, float, fold, form, popup, svg, group, path, rack, button, image, input, select, table, thead, tbody, tr, th, td, text, textarea, render`（render 需 `render` feature，已启用）

## 十一、brick 结构体上的助手方法（`classify.rs`/`render.rs`）

- `SizeAttr::size_style()`、`ImageAttr::size_style()`、`PositionAttr::into_style()`、`DirectionAttr::into_style()`
- `BrickOps` trait：`get_type()`、`borrow_sub()`、`borrow_attrs()`、`get_bind()`、`get_id()`、`set_bind()`、`borrow_sub_mut()`
- `Classify` trait：`get_class()`、`get_selector()`、`is_horizontal()`、`add_class()`、`delete_class()`
- `Brick::render(&mut self, env)`：minijinja 模板渲染（ctx.rs 已用）
- `JsType`：`input_type()`、`default_value()`；变体 `bool`、`number`、`text`、`password`、`button`、`submit`

## 十二、form/input/button 信号共享方案（待实现）

由于 `signal` 字段被 cfg 掉，建议用 `provide_context` 共享：
- `form_` 定义 `FormState { fields: HashMap<String, RwSignal<Value>>, confirm: RwSignal<Value> }` 并 `provide_context`。
- `input_`/`textarea_` 通过 `use_context::<FormState>()` 取 `fields[field]` 写入。
- `button_` 通过 `use_context::<FormState>()` 取 `confirm` 切换。
- `form_` 用 `Effect` 监听 `confirm`，为 true 时 `spawn_local` 发送 `ctx.send(event, None, v)`。
- 注意：`provide_context` 需在 children 的响应式闭包**执行前**调用（同一 owner 作用域内）。

## 十三、Dioxus 参考源文件（port 时对照）

- hooks：`crates/ui/src/libs/hooks.rs`
- store/ws：`crates/ui/src/libs/store.rs`、`crates/ui/src/libs/ws.rs`
- 容器组件：`crates/ui/src/libs/components/{container,frame,dynamic,chart,diagram,float,fold,form,popup,rack,render,svg}.rs`
- 叶子组件：`crates/ui/src/libs/components/widgets/{button,image,input,select,table,text,textarea}.rs`

## 十四、已读过的 Dioxus 组件源码要点（移植备忘）

- `case_`：css = `["case","f"]` + id + grid 样式；`use_common_css`。
- `placeholder_`：绑定 `Source` 时从 `data` 取源，包 Frame；有 id 时做 fade-in-out 动画。
- `float_`：css = `["float","f"]` + `PositionAttr::into_style()`。
- `fold_`：css = `["g"]` + id；`item[0]` 作 header；`show` 信号控制展开；grid 布局。
- `form_`：见第十二节。
- `popup_`：css = `["popup","f"]` + `DirectionAttr::into_style()`；sub[0]=placeholder、sub[1]=modal。
- `rack_`：css = `["rack","f"]`；`ItemContainer` 按 selector 索引；`use_source_id` 取 source；遍历 `list[source]`；scroll 时滚到底。
- `svg_`/`group_`/`path_`：css 各加 `svg`/`group`/`path`；`size_style()`/`style` map。
- `chart_`/`diagram_`：`use_default` 取数据，`document::eval` 跑 ApexCharts/mermaid。
- `button_`：`use_default` 取文本；`Submit` 变体切换 confirm 信号；`oneshot` 属性。
- `image_`：`use_default` 取 src；`ImageAttr.size_style()` + desc。
- `input_`：`Field`（写 scope 信号）/`Event`（Enter 发送）区分；`JsType` 决定 type/value。
- `select_`：`use_source_list` 取 options；`use_source_value` 取 current；点选切换。
- `table_`/`thead_`/`tbody_`/`tr_`/`th_`/`td_`：直接渲染 children。
- `text_`：`use_source_value` 取文本；markdown 格式用 `to_html_with_options` + `dangerous_inner_html`。
- `textarea_`：`use_source_value` 初值；Enter 发送 `use_target_value`。
- `render_`：空视图（占位）。

## 十五、git 状态

- 已暂存：`Cargo.lock`、`Cargo.toml`、`crates/ui-leptos/*`、`crates/ui_leptos_macro/*`、`docs/decisions/003-ui-leptos-migration.md`
- 分支 `main`，未提交。**提交前请先让 `cargo check -p ui-leptos` 通过。**