# Rgain

本地桌面 Brainstorm：选定书籍或代码目录 → 调用大模型归纳 → 在 **2D 逻辑图**上浏览结构与节点讲解。

当前版本以 2D Flow 为主舞台；3D Trunk 仍在实验中，界面入口暂未开放。

## 功能概览

- **导入解析**：选择代码/书籍目录，或单本图书（PDF / EPUB / MOBI）
- **逻辑图**：Cytoscape + ELK 布局（Flow / Tree / Radial / Force）；点击节点查看讲解与源文件引用
- **解析历史**：结果写入本机 SQLite，可回放、重命名、删除
- **划词释义**：默认快捷键 `Ctrl+Shift+E`（可在设置中修改），释义写入本地历史
- **设置**：界面语言（zh / en / ja / ko）、LLM Base / Key / Model、Agent Prompt、「测试连接」、开发端口、Sidecar 自检  
  API Key 仅保存在 Rust 后端，前端只显示脱敏串

## 技术栈

| 层 | 选型 |
|----|------|
| 桌面壳 / UI | Tauri 2 + Vue 3 + TypeScript + Vite |
| 2D 图 | Cytoscape、ELK |
| 3D（实验） | TresJS / Three.js |
| 文本与代码 | Rust 直读 |
| PDF / EPUB / MOBI | Python sidecar（`python/rgain_parser`） |
| 本地缓存 | SQLite（Rust 单一写者） |

## 环境要求

1. **Node.js + npm**（本仓库只用 npm）
2. **Rust toolchain**（[rustup](https://rustup.rs/)，需 `cargo` 在 PATH）
3. **Python 3**（开发态解析图书时需要；建议独立虚拟环境 / conda env，勿装到 `base`）
4. **Windows WebView2**（Tauri 桌面运行时）

## 快速开始

```bash
npm install
npm run tauri dev
```

仅前端：

```bash
npm run dev
```

发布构建：

```bash
npm run tauri build
```

开发默认端口 **20015**（≥ 20000）。可在设置页修改，或写入仓库根目录 `rgain.dev.json`；也可用环境变量 `RGAIN_DEV_PORT` 覆盖。改端口后需重启 `tauri dev`。

### Python Sidecar（图书解析）

```bash
# 激活你的开发环境后
pip install -r python/requirements.txt
```

| 模式 | 行为 |
|------|------|
| 开发（debug） | 使用 Python 模块 `rgain_parser`；可用 `RGAIN_PYTHON` 指定解释器 |
| 产品（release） | 启动同目录或 `sidecars/` 下的 `rgain_parser[.exe]`；可用 `RGAIN_SIDECAR` 覆盖 |

自检（不启动 Tauri）：

```bash
python python/smoke_test.py
```

## 支持的文件类型

- **图书**：`.pdf`、`.epub`、`.mobi`
- **代码 / 文本**（目录扫描）：常见源码与配置扩展名（如 `md`、`py`、`rs`、`ts`、`vue`、`go`、`json` 等）
- 自动跳过：`.git`、`node_modules`、`target`、`dist`、`__pycache__`、虚拟环境等目录

## 使用流程

1. 在设置中填写 LLM API Base、Key、Model，并「测试连接」
2. 左栏选择目录或图书 →「开始解析」
3. 中部查看逻辑图，点击节点阅读右侧讲解
4. 需要时划词 / 快捷键触发术语释义

## 配置与数据

| 项 | 说明 |
|----|------|
| `rgain.dev.json` | 开发端口等本地同步配置 |
| `RGAIN_PYTHON` | 覆盖开发态 Python 路径 |
| `RGAIN_SIDECAR` | 强制使用指定 sidecar 二进制 |
| `RGAIN_DEV_PORT` | 覆盖 Vite 开发端口 |
| 应用数据 | 本机 `app_data_dir/rgain.sqlite3`（项目、运行、图、设置） |

密钥与 `.env` 请勿提交；仓库已忽略 `.env`、本地数据库与构建产物。

## 仓库说明

- 版本：`0.1.0`（早期可用）
- 内部设计稿与选型笔记在本地 `docs/`（默认不纳入 Git），公开说明以本 README 为准

## 许可证

本项目采用 [Apache License 2.0](./LICENSE)。
