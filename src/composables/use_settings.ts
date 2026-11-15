import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  clamp_dev_port,
  default_settings,
  type AppSettings,
} from "../types/settings";

const settings = ref<AppSettings>({ ...default_settings });
const settings_loaded = ref(false);
const settings_error = ref("");

function normalize_settings(data: Partial<AppSettings>): AppSettings {
  const api_key_set = Boolean(data.api_key_set);
  return {
    ...default_settings,
    ...data,
    api_key_set,
    // Keep backend-provided masked key (`sk****xxxx`).
    api_key: data.api_key ?? "",
    agent_prompt: data.agent_prompt ?? default_settings.agent_prompt,
    mcp_enabled: Boolean(data.mcp_enabled),
    mcp_server_url: data.mcp_server_url ?? "",
    dev_port: clamp_dev_port(data.dev_port ?? default_settings.dev_port),
  };
}

export function use_settings() {
  async function load_settings() {
    settings_error.value = "";
    try {
      const data = await invoke<AppSettings>("get_settings");
      settings.value = normalize_settings(data);
      settings_loaded.value = true;
    } catch (err) {
      settings_error.value = String(err);
      settings.value = { ...default_settings };
      settings_loaded.value = true;
    }
  }

  async function save_settings(next: AppSettings) {
    settings_error.value = "";
    const payload = {
      ...normalize_settings(next),
      api_key: next.api_key ?? "",
    };
    const saved = normalize_settings(
      await invoke<AppSettings>("save_settings", { settings: payload }),
    );
    settings.value = saved;
    return saved;
  }

  return {
    settings,
    settings_loaded,
    settings_error,
    load_settings,
    save_settings,
  };
}
