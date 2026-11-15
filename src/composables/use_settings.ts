import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { default_settings, type AppSettings } from "../types/settings";

const settings = ref<AppSettings>({ ...default_settings });
const settings_loaded = ref(false);
const settings_error = ref("");

export function use_settings() {
  async function load_settings() {
    settings_error.value = "";
    try {
      const data = await invoke<AppSettings>("get_settings");
      settings.value = { ...default_settings, ...data };
      settings_loaded.value = true;
    } catch (err) {
      settings_error.value = String(err);
      settings.value = { ...default_settings };
      settings_loaded.value = true;
    }
  }

  async function save_settings(next: AppSettings) {
    settings_error.value = "";
    const saved = await invoke<AppSettings>("save_settings", { settings: next });
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
