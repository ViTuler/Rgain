# Rgain

本地 Brainstorm 桌面应用：选定书籍/代码目录 → 多模型归纳 → **3D 空间逻辑图**讲解。

## 技术栈

| 层 | 选型 |
|----|------|
| 壳 / UI | Tauri 2 + Vue3 + TypeScript |
| 3D | TresJS / Three.js（Trunk Timeline + DAG） |
| 解析 | 纯文本 → Rust；PDF/EPUB → Python sidecar |
| 缓存 | SQLite（Rust 单一写者） |
| 主色 | `#75eec5` · `#ff1b1b` |

文档见 [`docs/`](./docs/)。

## 开发

前置：

1. **Node + npm**（本仓库只用 npm，不用 pnpm/yarn）
2. **Rust toolchain**（[rustup](https://rustup.rs/) 默认安装即可；需 `cargo` 在 PATH）
3. **conda env `vi`**（sidecar；Phase A 无需额外 pip 包）
4. Windows WebView2

```bash
npm install
npm run tauri dev
```

Sidecar 自检（不启动 Tauri）：

```bash
E:\Conda\envs\vi\python.exe python/smoke_test.py
```

设置页默认 Python：`E:\Conda\envs\vi\python.exe`。

## 文档索引

- [技术选型](./docs/tech_selection.md)
- [解析：Rust vs Python](./docs/parsing_rust_vs_python.md)
- [实现计划](./docs/plan.md)
- [设计色板](./docs/design_tokens.md)
