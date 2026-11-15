import {
  DEFAULT_UI_LOCALE,
  normalize_ui_locale,
  type UiLocale,
} from "../i18n";

/** Allowed floor for configurable ports. */
export const MIN_DEV_PORT = 20000;
/** Project default frontend port. */
export const DEFAULT_DEV_PORT = 20015;

export type AppSettings = {
  /** 通用 */
  explain_hotkey: string;
  /** Show magnifier FAB after mouse text selection. */
  selection_explain_enabled: boolean;
  /** Vite / Tauri dev listen port; must be >= 20000. Restart tauri dev after change. */
  dev_port: number;
  /** UI + LLM output language. */
  ui_locale: UiLocale;

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

  /** MCP（预设，二期） */
  mcp_enabled: boolean;
  mcp_server_url: string;
};

export const default_settings: AppSettings = {
  explain_hotkey: "Ctrl+Shift+E",
  selection_explain_enabled: true,
  dev_port: DEFAULT_DEV_PORT,
  ui_locale: DEFAULT_UI_LOCALE,
  api_base_url: "https://api.openai.com/v1",
  api_key: "",
  api_key_set: false,
  model: "gpt-4o-mini",
  agent_prompt: "",
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

export function looks_like_masked_api_key(value: string): boolean {
  return /\*{4,}/.test(value.trim());
}

export function is_keep_api_key(value: string): boolean {
  const v = value.trim();
  return !v || v === "******" || v === "********" || looks_like_masked_api_key(v);
}
