# Rgain 设计色板

> 对齐 Rain blog（`h:/RainV/Rain/app/assets/styles/tokens.css`）  
> **骨架：暖黑 / 暖白；点缀：红 `#ff1b1b` + 青 `#75eec5`**

## 原则

- 黑白（暖相）占面积主体；红、青只做点缀与交互态。  
- 青（accent）负责高亮、主按钮、主干轴、链接感。  
- 红（brand）克制使用：徽章、关键节点、划词反馈、告警。  
- 避免纯 `#000` / `#fff`；Rain 用 `#171414` 与 `#f5efef`。

## 品牌与强调

| Token | Hex | 角色 |
|-------|-----|------|
| `color_brand` | `#ff1b1b` | 品牌红，点缀 |
| `color_accent` | `#75eec5` | 青色高亮 |
| `color_accent_deep` | 深 `#1f7d5b` / 夜 `#8ed9bc` | 浅底可读青 / 深底链接青 |
| `color_accent_ink` | `#17614a` | 落在亮青上的字色 |
| `color_accent_soft` | 浅 `#e9fbf5` / 夜 `#17332e` | 轻底 chip |

## 黑白骨架

| Token | 浅色 | 深色（App 默认） | 说明 |
|-------|------|------------------|------|
| `color_ink` | `#171414` | `#f3ecec` | 主文字 |
| `color_body` | `#3f3a3a` | `#d0c4c2` | 正文 |
| `color_muted` | `#656565` | `#aea19f` | 次要 |
| `color_bright` | `#f5efef` | `#322a29` | 最亮/翻转面 |
| `color_surface` | `#f7f3f3` | `#1c1716` | 侧栏 |
| `color_surface_2` | `#efe8e8` | `#141110` | 舞台底 |
| `color_line` | `#e7dfdf` | `#3c3331` | 分割线 |
| `color_night` | `#171414` | `#100e0e` | 固定深底块 |

## 3D 用法

- 舞台底：`color_surface_2` / `color_night`  
- 主干边/选中：`color_accent`  
- 关键支路：`color_brand`（细描边，不大面积填）  
- 模块 categorical：仍可用冷色环，但降饱和，避免压过红青

## CSS / 主题

见 `src/styles/tokens.css`。

- **默认 Light**（与 Rain 一致）：`:root` 为浅色暖白骨架  
- **Dark**：`html[data-theme="dark"]`  
- 主题仅前端：顶栏 sun/moon 切换，持久化 `localStorage.rgain_theme`（不写 Rust/SQLite，避免开发热重载）  
- 3D 舞台区域仍使用 `--color_night` 深底
