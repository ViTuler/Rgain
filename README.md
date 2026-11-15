# Rgain

本地 Brainstorm 桌面应用：选定书籍/代码目录 → 多模型归纳 → **3D 空间逻辑图**讲解。

## 技术栈

| 层 | 选型 |
|----|------|
| 壳 / UI | Tauri 2 + Vue3 + TypeScript |
| 3D | TresJS / Three.js（Trunk Timeline + DAG） |
| 解析 | 纯文本 → Rust；PDF/EPUB → Python sidecar |
| 缓存 | SQLite（Rust 单一写者） |
| 色板 | 暖黑/暖白骨架 + 青 `#75eec5` / 红 `#ff1b1b`（对齐 Rain blog） |

文档见 [`docs/`](./docs/)。

## 开发

前置：

1. **Node + npm**（本仓库只用 npm，不用 pnpm/yarn）
2. **Rust toolchain**（[rustup](https://rustup.rs/) 默认安装即可；需 `cargo` 在 PATH）
3. **conda env `vi`**（仅开发态 sidecar；PDF/EPUB：`pip install -r python/requirements.txt`，勿装到 base）
4. Windows WebView2

```bash
npm install
npm run tauri dev
```

开发默认端口 **20015**（可在设置中修改，≥20000；保存后重启 `tauri dev`）。端口同步文件：`rgain.dev.json`。

**Sidecar 解析器**

| 模式 | 启动方式 |
|------|----------|
| 开发（debug） | 自动使用 `E:\Conda\envs\vi\python.exe -m rgain_parser`（可用 `RGAIN_PYTHON` 覆盖） |
| 产品（release） | 自动启动同目录 / `sidecars/` 下的 PyInstaller 产物 `rgain_parser[.exe]`（可用 `RGAIN_SIDECAR` 覆盖） |

Sidecar 自检（不启动 Tauri）：

```bash
E:\Conda\envs\vi\python.exe python/smoke_test.py
```

设置页：API Base / Key / Model（Key 仅后端保存，前端只显示脱敏串）+「测试连接」。  
Phase B：左栏选目录 →「开始解析」→ 图 JSON 写入 SQLite，历史可回放。  
Phase C：中部 2D/3D 逻辑舞台（点节点开右侧讲解；无数据时显示 demo 图）。  
Phase D：设置里配置划词快捷键（默认 `Ctrl+Shift+E`）→ 选中文本或当前节点 → 弹窗释义并写入 SQLite；历史列表可回放并高亮当前 run。

## 文档索引

- [技术选型](./docs/tech_selection.md)
- [解析：Rust vs Python](./docs/parsing_rust_vs_python.md)
- [实现计划](./docs/plan.md)
- [设计色板](./docs/design_tokens.md)
