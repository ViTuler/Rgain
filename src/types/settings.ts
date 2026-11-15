/** Allowed floor for configurable ports. */
export const MIN_DEV_PORT = 20000;
/** Project default frontend port. */
export const DEFAULT_DEV_PORT = 20015;

export type AppSettings = {
  /** 通用 */
  explain_hotkey: string;
  /** Vite / Tauri dev listen port; must be >= 20000. Restart tauri dev after change. */
  dev_port: number;

  /** LLM API */
  api_base_url: string;
  /**
   * From backend: masked `sk****xxxx` when set.
   * On save / test: empty or masked keeps stored key; plain text updates it.
   */
  api_key: string;
  api_key_set: boolean;
  model: string;
  /** Global agent system prompt shared by all projects. */
  agent_prompt: string;

  /** MCP（预设，二期） */
  mcp_enabled: boolean;
  mcp_server_url: string;
};

export const default_settings: AppSettings = {
  explain_hotkey: "Ctrl+Shift+E",
  dev_port: DEFAULT_DEV_PORT,
  api_base_url: "https://api.openai.com/v1",
  api_key: "",
  api_key_set: false,
  model: "gpt-4o-mini",
  agent_prompt: "",
  mcp_enabled: false,
  mcp_server_url: "",
};

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
