import {
  DEFAULT_UI_LOCALE,
  normalize_ui_locale,
  type UiLocale,
} from "../i18n";

/** Allowed floor for configurable ports. */
export const MIN_DEV_PORT = 20000;
/** Project default frontend port. */
export const DEFAULT_DEV_PORT = 20015;

export const FONT_PX_MIN = 10;
export const FONT_PX_MAX = 32;
export const DEFAULT_EXPLAIN_FONT_PX = 14;
export const DEFAULT_TERM_FONT_PX = 14;
export const DEFAULT_GRAPH_FONT_PX = 12;

export type AppSettings = {
  /** 通用 */
  explain_hotkey: string;
  /** Show magnifier FAB after mouse text selection. */
  selection_explain_enabled: boolean;
  /** Vite / Tauri dev listen port; must be >= 20000. Restart tauri dev after change. */
  dev_port: number;
  /** UI + LLM output language. */
  ui_locale: UiLocale;
  /** Right-side node explain body, px. */
  explain_font_px: number;
  /** Selection / term explain body, px. */
  term_font_px: number;
  /** Logic-graph label size, px. */
  graph_font_px: number;

  /** LLM API */
  api_base_url: string;
  /**
   * From backend: masked `sk****xxxx` when set.
   * On save / test: empty or masked keeps stored key; plain text updates it.
   */
  api_key: string;
  api_key_set: boolean;
  model: string;
  /** User-managed agent prompt (optional). Built-in schema prompt is separate. */
  agent_prompt: string;
  /**
   * Max extracted-text size (MB) per LLM request when ingesting large books/projects.
   * Oversized corpora are split and merged in one session.
   */
  chunk_size_mb: number;
  /** Ask reasoning models to think before answering. Default on. */
  think_enabled: boolean;
  /** Directory names or relative paths skipped when scanning a project folder. */
  ignore_paths: string[];

  /** MCP（预设，二期） */
  mcp_enabled: boolean;
  mcp_server_url: string;
};

/** Default text budget per LLM pass (~0.5 MiB). */
export const DEFAULT_CHUNK_SIZE_MB = 0.5;
export const MIN_CHUNK_SIZE_MB = 0.1;
export const MAX_CHUNK_SIZE_MB = 8;

/** Directory names skipped while scanning a project folder. Checked by default. */
export const DEFAULT_IGNORE_PATHS = [
  "node_modules",
  "dist",
  "dist-ssr",
  "build",
  "out",
  "target",
  ".git",
  ".svn",
  "__pycache__",
  ".venv",
  "venv",
  ".tox",
  ".pytest_cache",
  ".mypy_cache",
  ".ruff_cache",
  ".cache",
  ".parcel-cache",
  ".turbo",
  ".next",
  ".nuxt",
  ".output",
  "coverage",
  "htmlcov",
  ".nyc_output",
  "vendor",
  "bower_components",
  ".gradle",
  ".dart_tool",
  "bin",
  "obj",
  ".idea",
  ".vscode",
];

export const default_settings: AppSettings = {
  explain_hotkey: "Ctrl+Shift+E",
  selection_explain_enabled: true,
  dev_port: DEFAULT_DEV_PORT,
  ui_locale: DEFAULT_UI_LOCALE,
  explain_font_px: DEFAULT_EXPLAIN_FONT_PX,
  term_font_px: DEFAULT_TERM_FONT_PX,
  graph_font_px: DEFAULT_GRAPH_FONT_PX,
  api_base_url: "https://api.openai.com/v1",
  api_key: "",
  api_key_set: false,
  model: "gpt-4o-mini",
  agent_prompt: "",
  chunk_size_mb: DEFAULT_CHUNK_SIZE_MB,
  think_enabled: true,
  ignore_paths: [...DEFAULT_IGNORE_PATHS],
  mcp_enabled: false,
  mcp_server_url: "",
};

export { normalize_ui_locale };

export function clamp_dev_port(value: unknown): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return DEFAULT_DEV_PORT;
  const port = Math.floor(n);
  if (port < MIN_DEV_PORT) return MIN_DEV_PORT;
  if (port > 65535) return 65535;
  return port;
}

export function clamp_font_px(value: unknown, fallback: number): number {
  const n = Math.round(Number(value));
  if (!Number.isFinite(n)) return fallback;
  if (n < FONT_PX_MIN) return FONT_PX_MIN;
  if (n > FONT_PX_MAX) return FONT_PX_MAX;
  return n;
}

export function normalize_ignore_path(raw: string): string {
  let text = raw.trim().replace(/\\/g, "/");
  while (text.startsWith("./")) text = text.slice(2);
  text = text.replace(/^\/+|\/+$/g, "");
  if (!text || text === "." || text === ".." || text.split("/").includes("..")) return "";
  if (/^[a-zA-Z]:/.test(text)) return "";
  if (text.length > 120) return "";
  return text;
}

export function normalize_ignore_paths(value: unknown): string[] {
  const source = Array.isArray(value) ? value : DEFAULT_IGNORE_PATHS;
  const out: string[] = [];
  const seen = new Set<string>();
  for (const item of source) {
    const path = normalize_ignore_path(String(item));
    const key = path.toLowerCase();
    if (!path || seen.has(key)) continue;
    seen.add(key);
    out.push(path);
    if (out.length >= 64) break;
  }
  return out;
}

export function clamp_chunk_size_mb(value: unknown): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return DEFAULT_CHUNK_SIZE_MB;
  const rounded = Math.round(n * 100) / 100;
  if (rounded < MIN_CHUNK_SIZE_MB) return MIN_CHUNK_SIZE_MB;
  if (rounded > MAX_CHUNK_SIZE_MB) return MAX_CHUNK_SIZE_MB;
  return rounded;
}

export function looks_like_masked_api_key(value: string): boolean {
  return /\*{4,}/.test(value.trim());
}

export function is_keep_api_key(value: string): boolean {
  const v = value.trim();
  return !v || v === "******" || v === "********" || looks_like_masked_api_key(v);
}
