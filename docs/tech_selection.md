# Brainstorm App — 技术选型

> 状态：技术选型已定（Vue3；Python sidecar；主色 mint/crimson）  
> 包名：`rgain` / `com.vialon17.rgain`  

> 日期：2026-11-11  
> 工作区：`h:\RainV\Rgain`  
> 团队栈：Vue / Python 为主，兼 Flutter  
> 关联：[parsing_rust_vs_python.md](./parsing_rust_vs_python.md)

---

## 1. 产品目标（一句话）

本地桌面 App：选定目录（书籍 PDF/EPUB + 代码项目）→ 多模型 API 按 Agent Prompt 归纳 → 返回结构化「主干 + 分支」逻辑图数据 → GUI 自适应 3D 空间绘制讲解；支持划词解释、SQLite 历史缓存，并为后续 MCP 留接口。

---

## 2. 能力拆解与技术约束

| 能力 | 约束 / 要点 |
|------|-------------|
| 多模型 API | OpenAI 兼容优先；可插拔 Provider；内置短 Prompt + 用户补充 |
| 本地文件阅读 | PDF / EPUB / 常见代码与文档：`md` `py` `cpp` `rc` `ts` `rs` 等 |
| 结构化输出 | 模型产出 JSON：节点、边、时间/逻辑轴、模块色、讲解文案 |
| 3D GUI | 主干 + 支路；按模块着色；自适应布局重构；可旋转/聚焦 |
| 划词解释 | 可配置全局快捷键；带上下文调 API 名词解释 |
| 本地缓存 | SQLite：解析任务、图数据、解释记录、设置 |
| MCP 预留 | 同一 SQLite / 图模型可被 MCP Server 读写 |

**关键结论：** 难点不在「能不能画 3D」，而在 **文档抽取 + LLM 结构化输出契约 + 图布局引擎 + 本地持久化与 MCP 同源数据**。前端 3D 应站在成熟的 WebGL 图可视化生态上，而不是从 Qt/Flutter 自研三维图引擎。

---

## 3. 候选技术栈对比

### 3.1 总表

| 维度 | **Tauri 2 + Vue3/TS** | Electron + Vue3/TS | PySide6 (Qt) | Flutter Desktop |
|------|------------------------|---------------------|--------------|-----------------|
| 安装包体量 | 很小（数 MB～十余 MB 级） | 大（~150MB+） | 中等 | 中等（~15–25MB） |
| 内存占用 | 低 | 高 | 中 | 中偏低 |
| 3D 图生态 | **强**（Three.js / TresJS / 3d-force-graph） | 同左（WebGL） | 弱（Qt3D / VisPy 交互图谱库少） | 弱（自绘/插件少） |
| 文件解析 | 轻量 Rust + **Python sidecar**（团队主路径） | npm 或同样侧挂 Python | **Python 解析最强**（PyMuPDF、ebooklib） | 需插件/FFI，生态一般 |
| LLM / 流式 | TS / Rust / **Python 编排**均可 | 极成熟 | 极成熟 | 可用但样板少 |
| SQLite | `rusqlite` / sqlx 原生 | better-sqlite3 | sqlite3 原生 | sqflite / drift |
| 全局快捷键 | Tauri global shortcut | electron globalShortcut | Qt QShortcut / 系统钩子 | 需平台通道 |
| MCP 预留 | Python MCP 读同一 DB（与 sidecar 同栈） | 同左或 Node MCP | Python MCP 很自然 | 需另起进程 |
| 学习/交付成本 | 低～中（Vue 熟；Rust 保持薄） | 低（纯 Web） | 中（3D 短板） | 中高（Dart + 弱 3D） |
| 与本产品适配 | **首选（已定）** | 备选（要最快原型） | 备选（解析优先、3D 降级） | **不推荐主路径** |

### 3.2 Tauri 2 + Vue3 + TypeScript（已定主栈）

**优势**

- 贴合团队 **Vue** 主力；3D 用 **TresJS**（Vue 版 R3F）或 `3d-force-graph` 挂载到 Vue 组件即可。
- 与产品形态重合：本地文件、轻量桌面、WebGL 3D、SQLite、后续 MCP。
- Rust 层保持「薄编排」：权限、快捷键、DB、sidecar 生命周期；业务解析交给 Python。

**劣势**

- 系统 WebView 跨平台差异需回归（Windows WebView2 通常可用）。
- 引入 Python sidecar 后有打包与进程管理成本（见解析专文）。

**适配性：** 在「Vue 团队 + 3D 讲解 + Python 文档智能」约束下最优。

### 3.3 Electron + Vue3 + TypeScript（快速原型备选）

**优势**

- 无 Rust；Vue 生态直接上；PDF.js 等也可先验证 UI。
- 与 Tauri 前端可大量复用（均为 Vue3 + Vite）。

**劣势**

- 体积与内存大；安全面更宽。
- 长期维护与分发不如 Tauri。

**适配性：** 仅当要极速验证 3D/数据契约时考虑；正式产品仍 Tauri。

### 3.4 PySide6（Qt）

**优势**

- 文档解析、Agent 编排、MCP（Python SDK）一条龙最顺。
- SQLite、快捷键、本地路径处理成熟。
- 团队若以 Python 为主，后端逻辑开发最快。

**劣势**

- **3D 知识/逻辑图是短板**：缺少接近 `force-graph-3d` / R3F 的成熟「可交互、可着色模块、可自适应布局」组件；自研成本高、观感易落后。
- 可嵌入 Qt WebEngine 跑 Three.js，但等于「Qt 壳 + Web 图」，复杂度接近 Electron/Tauri，却少了对应生态惯例。

**适配性：** 适合做成「解析 + Agent 引擎」微服务/sidecar，不适合单独承担最终 3D GUI。

### 3.5 Flutter Desktop

**优势**

- 跨端 UI 一致、包体尚可、自绘性能好。

**劣势**

- 3D 力导向/树状逻辑图库远弱于 WebGL；PDF/EPUB/代码索引与 LLM 流式样板少；MCP 需外挂。
- 与「逻辑空间图」这一核心体验不匹配。

**适配性：** 本阶段不作为主栈。

---

## 4. 推荐架构（定稿方向）

### 4.1 已定：Tauri 2 + Vue3 + Python Sidecar（解析主路径）

```
┌─────────────────────────────────────────────────────────┐
│  Frontend (Vue3 + TS + Vite)                            │
│  - 3D Scene: TresJS / three + force/DAG / trunk layout  │
│  - 讲解面板 / 划词解释浮层 / Prompt 编辑器              │
│  - 历史任务列表（读 SQLite via invoke）                 │
└──────────────────────────▲──────────────────────────────┘
                           │ Tauri commands / events
┌──────────────────────────┴──────────────────────────────┐
│  Rust Core (Tauri，保持薄)                              │
│  - 目录扫描；纯文本自读；pdf/epub 转发 sidecar          │
│  - sidecar 生命周期 / IPC 管家                          │
│  - SQLite 单一写者（收 ExtractResult / Graph JSON）     │
│  - 全局快捷键 / 设置；LLM 可由 Rust 或转 Python         │
│  - MCP 预留：同库 schema                                │
└──────────────────────────▲──────────────────────────────┘
                           │ stdio JSON 或 127.0.0.1 HTTP
┌──────────────────────────┴──────────────────────────────┐
│  Python Sidecar（conda env ≠ base，如 rgain）           │
│  - PDF/EPUB（PyMuPDF、ebooklib）主解析路径              │
│  - chunk / 清洗；后续 Agent 编排、OCR、MCP tools        │
└─────────────────────────────────────────────────────────┘
```

**解析分流（详见 [parsing_rust_vs_python.md](./parsing_rust_vs_python.md)）**

1. **代码与纯文本：** Rust 直接读。  
2. **PDF/EPUB：** Python sidecar 为 MVP 主路径（非「后期增强」）。  
3. **缓存：** 按文件 hash 存抽取结果，避免重复解析。

### 4.2 为何不把 PySide 当主壳

3D 讲解是产品差异化卖点；WebGL 图库（含 DAG/时间轴约束、点击展开、模块着色）已有可直接站的肩膀。Python 价值放在 **解析与 Agent**，通过 sidecar 接入，而不是反向用 Qt 硬扛 3D。

### 4.3 与「纯 Electron」的取舍

- **长期产品：** Tauri。  
- **极速验证 UI/数据契约：** 可临时 Electron，前端目录设计为可迁移。

---

## 5. 3D 视图与 UX / 色调（参考结论）

### 5.1 参考项目

| 项目 | 栈要点 | 可借鉴 |
|------|--------|--------|
| [Filamental](https://github.com/Scottnine/filamental) | Tauri + 3D 知识图 + **MCP 读同一 SQLite** | MCP 余留模式、本地优先、节点/边模型 |
| [Cerebrium UI](https://github.com/MakhnoGK/cerebrium-ui) | Tauri + React + `react-force-graph-3d` | 节点大小∝度、颜色∝类型、时间线旁路 |
| [Tesseract](https://github.com/geckse/tesseract-md-app) | Electron + `3d-force-graph` | 本地 Markdown 知识图交互 |
| [OpenKB visualize](https://github.com/VectifyAI/OpenKB) | 3D / mind-map / radial 三视图 | **纯力导向易成毛球** → 需层次/时间视图 |
| [Cubons](https://github.com/cubons-io/cubons) | Three.js + 多布局（含 time axis） | 时间轴 × 模块聚类布局思路 |
| [Unfold](https://github.com/amolk/unfold) | R3F 分支叙事时间线 | 「主干 + 未来分支」叙事感 |
| Reagraph / r3f-forcegraph | R3F 图库 | `tree*` / `dagMode` 自适应层次布局 |

### 5.2 视图形态建议（不限死「树」）

优先实现 **「逻辑主干轴 + 侧向分支」**（时间或章节沿主轴 Z/Y；模块向两侧展开），并预留切换：

1. **Trunk Timeline（默认）** — 最贴「时间-树状 / 故事节点」  
2. **Hierarchical DAG** — `dagMode: td | lr | zout` 自动布局  
3. **Module Clusters** — 同模块聚色团，跨模块边弱化  

GUI **只消费 API JSON**，布局算法在客户端；模型保证语义对应，客户端保证不重叠、可读。

### 5.3 色调与 UX 原则（对齐 Rain blog）

**骨架：暖黑 / 暖白；点缀：红 + 青**（与 `Rain/app/assets/styles/tokens.css` 同源）

| Token | Hex | 用途 |
|-------|-----|------|
| `--color_accent` | `#75eec5` | 青：主干、主 CTA、聚焦、进度 |
| `--color_brand` | `#ff1b1b` | 红：品牌点缀、关键节点、划词反馈（克制） |
| `--color_ink` / surfaces | `#171414` · `#f5efef` 及阶梯 | 黑白主面积，非纯黑纯白 |

细则见 [design_tokens.md](./design_tokens.md)。青面积 > 红；避免紫渐变与仪表盘卡片墙。

- **交互：** 点击节点 → 右侧讲解；Hover 邻边用青；关键路径用红描边。  
- **动效 / 信息架构：** 同前（主干生长、相机缓动；左目录 · 中舞台 · 右讲解）。

---

## 6. 数据与 MCP 余留（选型级约定）

### 6.1 模型返回契约（草案）

```json
{
  "meta": {
    "title": "string",
    "axis": "time | logic | story",
    "summary": "string"
  },
  "modules": [
    { "id": "mod_core", "name": "核心层", "color_hint": "slate" }
  ],
  "nodes": [
    {
      "id": "n1",
      "label": "开篇设定",
      "module_id": "mod_core",
      "axis_order": 0,
      "kind": "trunk | branch",
      "parent_id": null,
      "explain": "讲解文案…",
      "source_refs": [{ "path": "…", "span": "…" }]
    }
  ],
  "edges": [
    { "from": "n1", "to": "n2", "relation": "next | derives | contrasts" }
  ],
  "terms": [
    { "term": "DAG", "node_ids": ["n2"] }
  ]
}
```

客户端根据 `axis_order` + `kind` + `module_id` 做自适应布局；`color_hint` 可覆盖默认 palette。

### 6.2 SQLite 表意向

- `projects` — 根目录、类型（book/code）、hash  
- `runs` — prompt、模型、状态、创建时间  
- `graph_snapshots` — 完整 JSON（或规范化 nodes/edges 表）  
- `file_extracts` — path、hash、text/chunks  
- `term_explanations` — 词条、上下文、解释、run_id  
- `settings` — API keys（加密）、快捷键、默认 prompt  

MCP 后续直接挂在同一 DB（Filamental 模式）：`search_nodes` / `get_subgraph` / `list_runs` 等，**App 与 MCP 共享 schema，避免双写**。

### 6.3 Prompt

- App 内置 **短 system agent prompt**（归纳规则 + JSON schema 约束）。  
- 用户可追加 `user_supplement`。  
- 划词解释：独立短 prompt，注入当前节点 `explain` + 前后文。

---

## 7. 选型结论（给 Plan 用）

| 项 | 决定 |
|----|------|
| 桌面壳 | **Tauri 2** |
| UI | **Vue3 + TypeScript + Vite** |
| 3D | **TresJS / Three.js**；布局可接 force/DAG；默认 Trunk Timeline |
| 持久化 | **SQLite WAL**（Rust 单一写者） |
| LLM | Provider 抽象，OpenAI-compatible；编排可落 Python |
| 文件解析 | **PDF/EPUB → Python sidecar**；纯文本 → Rust |
| MCP | Schema 与 DB 先设计好；Python MCP 二期 |
| 不选 | Flutter 主栈；PySide 作唯一 GUI；全量解析纯 Rust |

**一句话：** Tauri+Vue 做壳与 3D；Python sidecar 做文档解析与后续智能；Rust 只管编排与 SQLite；MCP 与 App 共用库。主色 `#75eec5` / `#ff1b1b`（见 `design_tokens.md`）。

实现节奏见 [plan.md](./plan.md)。

---

## 8. 后续 Plan 建议议题（不在本次展开）

1. MVP 范围：仅代码目录 vs 代码+PDF 文本层  
2. JSON schema 冻结与校验（失败重试 / 修复 prompt）  
3. Trunk 布局算法与碰撞避免  
4. 快捷键方案（Windows 默认候选）与划词管道  
5. API Key 本地加密存储  
6. 包名 / 仓库结构 / conda sidecar 环境名  
7. 与参考项目的许可与「仅借鉴思路不拷贝代码」边界  

---

## 9. 参考链接

- Filamental: https://github.com/Scottnine/filamental  
- Cerebrium UI: https://github.com/MakhnoGK/cerebrium-ui  
- Tesseract: https://github.com/geckse/tesseract-md-app  
- EchoMind (Tauri RAG): https://github.com/lisering/EchoMind  
- r3f-forcegraph: https://github.com/vasturiano/r3f-forcegraph  
- Unfold: https://github.com/amolk/unfold  
- Cubons: https://github.com/cubons-io/cubons  
- Tauri vs Electron vs Flutter (2026 对比文): https://kanopylabs.com/blog/tauri-vs-electron-vs-flutter-desktop-apps  
