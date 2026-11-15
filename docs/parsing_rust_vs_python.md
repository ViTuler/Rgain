# 文件解析：Rust 内建 vs Python 侧挂

> 前提壳层已定为：**Tauri 2 + Vue3 + TypeScript**  
> 团队栈：**Vue / Python 为主，兼 Flutter**（Rust 非主力）  
> 日期：2026-11-11

---

## 1. 两种方案分别是什么

| | **Rust 内建** | **Python 侧挂（Sidecar）** |
|--|---------------|---------------------------|
| 形态 | 解析逻辑写在 `src-tauri`，与 App 同进程 | Rust 只负责 spawn / IPC；解析在独立 Python 进程 |
| 通信 | Vue `invoke` → Rust 直接返回文本/chunk | Vue → Rust →（stdio JSON / 本地 HTTP）→ Python |
| 典型库 | `lopdf` / `pdf-extract` / `rbook` / zip+xml | `pymupdf`、`ebooklib`、`python-docx`、可选 OCR |
| 分发 | 单二进制为主 | 需捆绑解释器或 PyInstaller/`externalBin`，或约定本机 conda env |

---

## 2. 优劣对比（按本产品关心的维度）

| 维度 | Rust 内建 | Python 侧挂 | 对本项目的含义 |
|------|-----------|-------------|----------------|
| **开发速度（你们）** | 弱：要啃 crate API、PDF 脏文件排错 | **强**：PyMuPDF / ebooklib 上手快 | 解析迭代会频繁 → Python 更贴团队 |
| **PDF 实战质量** | 中：规整 PDF 可；线性化、奇怪编码、扫描件常踩坑，需多层 fallback | **强**：文本层、版面、表格、渲染页更成熟；OCR 路线现成 | 电子书场景 Python 更稳 |
| **EPUB** | 中：可做（zip+OPF）；边缘格式要自己补 | **强**：`ebooklib` 等现成 | 书籍向产品偏 Python |
| **代码/md/cpp…** | **强且简单**：Rust 读文本即可，不必 Python | 也能做，但无必要 | **纯文本应留在 Rust**，别什么都丢 sidecar |
| **性能 / 内存** | **优**：无进程切换、适合大批小文件扫描 | 中：进程常驻或冷启动有成本；大 PDF 一次性加载也吃内存 | 目录扫代码用 Rust；重文档用 Python |
| **包体与安装** | **优**：接近单包 | 差：要带 runtime 或要求用户装 env；Windows 打包最烦 | 内部分发可接受 conda；对外安装包要 PyInstaller 等 |
| **进程生命周期** | 无额外负担 | 要处理：启动、健康检查、崩溃重启、退出杀子进程、端口占用 | Tauri 侧要写一层 sidecar 管家（可模板化） |
| **崩溃隔离** | 解析 panic 可能影响主进程（可 catch） | **优**：坏 PDF 搞崩 Python，App 壳仍在 | 脏文档多时 Python 更安全 |
| **与 LLM Agent** | Rust 调 HTTP 也可，但你们更熟 Python | **强**：chunk、prompt 拼装、结构化校验可同进程 | 后期 Agent 编排可落在 Python |
| **与 MCP 余留** | Rust/Node MCP 读 SQLite | **Python MCP SDK 很自然**，可与解析共用工具链 | 二期 MCP 可直接挂 Python 侧 |
| **可维护边界** | 解析与壳耦合在同一语言 | **清晰**：壳=Tauri/Vue；智能=Python | 符合「Vue 画图 + Python 干脏活」分工 |
| **Flutter 经验迁移** | 无 | 无直接迁移；但「原生壳 + 脚本引擎」心智类似 sidecar | 不影响选型 |

---

## 3. 结论（结合你们的技术栈）

**文件解析主路径：侧挂 Python；Rust 只做薄编排 + 轻量文本。**

| 文件类型 | 建议归属 |
|----------|----------|
| `.md` `.txt` `.py` `.cpp` `.h` `.rc` `.ts` `.vue` 等源码/纯文本 | **Rust 直接读**（快、无依赖） |
| `.pdf` `.epub`（及后续 docx 等） | **Python sidecar** |
| 扫描版 PDF / OCR | **仅 Python**（或明确标「暂不支持」） |
| chunk / 送给模型前的清洗 | **优先 Python**（与 Agent 同栈） |
| 结果写入 SQLite | **Rust 写库**（或 Python 写、Rust 只读——二选一，建议 **单一写者**：初期 Rust 收 JSON 再入库，避免双写） |

一句话：**不要用 Rust 重写 PyMuPDF 生态；也不要把 `read_file("main.py")` 交给 Python。**

---

## 4. 推荐落地形态（MVP）

```
Vue3 (选择目录 / 看进度 / 3D)
    │ invoke
    ▼
Tauri Rust
    │ 1) walk 目录，分流：text → 自读；pdf/epub → 排队
    │ 2) spawn / 复用 Python sidecar
    │ 3) 收回 ExtractResult JSON → 写 SQLite → 再调 LLM（或转交 Python）
    ▼
Python sidecar  (conda env: `vi`，禁止 base)
    │ pymupdf / ebooklib / …
    │ stdin/stdout 行协议 或 127.0.0.1 HTTP
    ▼
{ path, hash, kind, text, pages?, chapters?, warnings[] }
```

**IPC 建议（简单优先）**

- MVP：`stdio` + 一行一个 JSON（请求/响应），易调试。  
- 需要流式进度（大书分页）：升级为本地 HTTP + SSE，或 stdout 多条 `progress` 事件。

**环境约定**

- 开发：使用 conda env **`vi`**；配置里的 `python_path` 指向该解释器。  
- 发布：再评估 PyInstaller 打成 `rgain-parser` 二进制，经 Tauri `externalBin` 捆绑。  
- 依赖清单见下文或后续 `python/requirements.txt`。

---

## 5. 什么时候才值得「改回 / 补上」Rust 解析

满足多数再考虑把某类格式迁回 Rust：

1. 某格式调用极高频、Python 启动/IPC 成瓶颈；  
2. 该格式用纯 Rust 已稳定（例如只做 EPUB 元数据 + 纯文本 spine）；  
3. 要做「无 Python 也能跑」的精简安装包。

在此之前，Rust 解析投入 ROI 偏低——你们主力不在 Rust，脏 PDF 排错成本会拖慢 3D/Agent 主线。

---

## 6. 风险与对策（选 Python 侧挂时）

| 风险 | 对策 |
|------|------|
| 用户没装 Python / env 错 | 设置页检测解释器；开发文档写明 `conda create -n rgain …`；发布走捆绑 |
| 僵尸进程 | Tauri `RunEvent::Exit` 必 kill；Windows 注意 job object / 子进程树 |
| 大文件阻塞 UI | 异步任务 + 进度事件；解析与 3D 线程分离 |
| 双写 SQLite | 约定：**仅 Rust（或仅 Python）写库**；另一侧只产出 JSON |
| 安全 | sidecar 只绑 `127.0.0.1`；不暴露局域网；路径校验在 Rust 白名单目录内 |

---

## 7. 与总选型的关系

- 壳与 3D：**Tauri + Vue3 + TS + TresJS/Three**（Vue 侧对应 R3F 生态）。  
- 解析与后续 Agent/MCP 工具：**Python sidecar**。  
- Rust：目录、权限、快捷键、SQLite、sidecar 生命周期、把结果交给前端——**薄而稳**，不堆业务。

详见 [tech_selection.md](./tech_selection.md) 中已同步的栈修正。
