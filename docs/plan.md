# Rgain 实现计划

> 依据：`tech_selection.md` · `parsing_rust_vs_python.md` · `design_tokens.md`  
> 栈：**Tauri 2 + Vue3 + TS**；解析侧挂 **Python**；主色 `#75eec5` / `#ff1b1b`  
> 状态：Phase D 完成 / Phase E 待做  
> Python conda：**`vi`**（B 需 `pymupdf` / `ebooklib` / `lxml`，见 `python/requirements.txt`）

---

## 0. 目标与非目标

**MVP 目标**

1. 选择本地目录（代码项目优先，PDF/EPUB 文本层可用）。  
2. 抽取内容 → 调 OpenAI-compatible API（内置短 agent prompt + 用户补充）。  
3. 校验 JSON 图数据 → 存 SQLite → Vue 3D 舞台按 Trunk 布局绘制。  
4. 点击节点看讲解；基础设置（API Key、模型、快捷键占位）。

**MVP 非目标**

- OCR 扫描版 PDF、完整 MCP Server、多窗口协同、账号云同步。

---

## 1. 仓库结构（目标）

```
rgain/
├── docs/                 # 选型与计划（已有）
├── src/                  # Vue3 前端
│   ├── styles/tokens.css
│   ├── views/
│   ├── components/scene/ # 3D
│   ├── composables/
│   └── types/graph.ts
├── src-tauri/            # Rust 薄编排 + SQLite
├── python/               # sidecar（conda env ≠ base）
│   ├── requirements.txt
│   └── rgain_parser/
└── prompts/              # 内置 agent / 划词解释 prompt
```

---

## 2. 阶段划分

### Phase A — 工程基线

| ID | 任务 | 状态 | 产出 |
|----|------|------|------|
| A1 | 色板落地 `tokens.css` | done | `src/styles/tokens.css` |
| A2 | 壳布局：左 / 中 / 右 | done | `App.vue` + panels |
| A3 | SQLite schema + Rust CRUD 骨架 | done | `src-tauri/src/db.rs` |
| A4 | Python sidecar health + echo | done | `python/rgain_parser`（stdlib） |
| A5 | 设置页 | done | API / model / `vi` python 路径 |

### Phase B — 解析与喂模

| ID | 任务 | 状态 | 产出 |
|----|------|------|------|
| B1 | Rust：目录 walk + 纯文本读取（扩展名白名单） | done | `ingest.rs` collect/extract |
| B2 | Python：PDF（PyMuPDF）/ EPUB（ebooklib）抽取 | done | sidecar `extract` |
| B3 | hash 缓存写入 `file_extracts` | done | sha256 + SQLite cache |
| B4 | 内置 agent prompt + LLM chat completions | done | `prompts/` + `llm.rs` |
| B5 | schema 校验失败 → 一次修复重试 | done | `graph.rs` validate + retry |
| BUI | LeftPanel 选目录 / 解析 / 历史回放 | done | dialog + `open_project` / `run_ingest` |

### Phase C — 讲解舞台（2D + 3D）

| ID | 任务 | 状态 | 产出 |
|----|------|------|------|
| C1 | TresJS/Three 场景 + 相机控制 | done | `LogicScene` + CameraControls |
| C2 | Trunk Timeline 布局（`axis_order` + `kind`） | done | `layout_trunk.ts` |
| C3 | 模块色映射（mint 主干 / crimson 强调 / categorical 模块） | done | trunk mint / branch crimson mix |
| C4 | 节点点击 → 右侧讲解；Hover 邻边高亮 | done | ExplainPanel + dim neighbors |
| C5 | 布局自适应（碰撞避让 / 深度间距） | done | pairwise XZ repulsion |
| C6 | 规范图契约 + Cytoscape/ELK 适配 | done | `types/graph.ts` · `graph_adapters.ts` |
| C7 | 2D Flow（json-to-graph 风格）+ 2D/3D 切换 | done | `LogicFlow2d` · `LogicStage` |
| C8 | 2D 节点位置缓存（按 run，SQLite） | done | `graph_layouts` · drag 防抖保存 |

### Phase D — 划词解释与历史

| ID | 任务 | 状态 | 产出 |
|----|------|------|------|
| D1 | 可配置全局快捷键（Tauri） | done | `global-shortcut` + 设置 `explain_hotkey` |
| D2 | 划词 + 上下文 → 解释 API → 浮层；写入 `term_explanations` | done | `explain_term` + TermExplainDialog |
| D3 | 历史 runs 列表回放图快照 | done | 历史高亮 `active_run_id` + `get_run_graph` |

### Phase E — MCP 余留（二期）

| ID | 任务 | 产出 |
|----|------|------|
| E1 | 冻结对外 graph/DB schema 文档 | MCP 可对接 |
| E2 | Python MCP：`list_runs` / `get_subgraph` / `search_nodes` | 外挂 AI 可读 |

---

## 3. 数据契约（冻结：`src/types/graph.ts`）

持久化 / LLM 产出：`meta` / `modules` / `groups?` / `nodes` / `edges` / `terms?`。

| 字段 | 作用 |
|------|------|
| `axis_order` / `kind` / `parent_id` | 3D Trunk 布局 |
| `group_id` / `groups` | 2D Cytoscape compound（`data.parent`） |
| `module_id` + `color_hint` | 两侧着色；无 `group_id` 时 2D 可按模块套盒 |

视图投影（不写库）：
- 2D → `to_cytoscape_elements` → Cytoscape.js + ELK（`flow-right` 默认）
- 3D → `layout_trunk_graph`

`meta.default_view` / `meta.default_flow_layout` 为打开偏好，UI 可覆盖。

---

## 4. Sidecar 约定（开发期）

- conda 环境：**`vi`**（**禁止在 base 装包**）。  
- 开发：自动 `vi` 解释器 `-m rgain_parser`；产品：自动启动打包的 `rgain_parser` 可执行文件（不再在设置里配 Python 路径）。  
- 协议 MVP：stdio，一行一个 JSON；字段 `cmd` / `ok` / `data` / `error`。  
- 退出：Tauri `RunEvent::Exit` 杀子进程。  
- API Key：仅 SQLite 后端持有；`get_settings` 不回传明文，前端展示 `******`。

---

## 5. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 模型 JSON 不稳定 | schema + 修复重试 + 前端容错空图 |
| 脏 PDF | Python 抽失败写 `warnings`；UI 提示 |
| 3D 毛球 | 默认 Trunk，不用纯力导向作主视图 |
| sidecar 未安装 | 设置页检测 health；纯文本模式仍可用 |

---

## 6. 建议实施顺序（下周起可执行）

1. **A1–A3**（UI 壳 + DB）与 **A4**（sidecar hello）并行  
2. **B1 → B4**（先代码目录跑通全链路，再接 PDF）  
3. **C1–C4**（假数据也可先画 3D，再接真 API）  
4. **D1–D3**  
5. schema 文档化后进 **E**

---

## 7. 完成定义（MVP Done）

- [x] 选一个本地代码仓库，一键解析并生成可旋转 3D 逻辑图  
- [x] 至少一种 PDF 或 EPUB 可抽取并进同一流水线  
- [x] 历史 run 可重新打开  
- [x] 主色 mint/crimson 在 UI 与 3D 中一致可辨  
- [x] 文档与代码中的包名均为 `rgain` / `com.vialon17.rgain`  
- [x] 划词解释快捷键可配置，结果写入本地 `term_explanations`
