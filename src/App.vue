<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import LeftPanel from "./components/panels/LeftPanel.vue";
import ScenePlaceholder from "./components/panels/ScenePlaceholder.vue";
import ExplainPanel from "./components/panels/ExplainPanel.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";
import { use_settings } from "./composables/use_settings";
import type { AppSettings } from "./types/settings";

const { settings, load_settings, save_settings } = use_settings();

const show_settings = ref(false);
const busy = ref(false);
const settings_message = ref("");
const sidecar_status = ref("未检测");
const sidecar_message = ref("");
const runs_count = ref(0);
const draft_settings = ref<AppSettings | null>(null);

const editing_settings = computed({
  get: () => draft_settings.value ?? settings.value,
  set: (value: AppSettings) => {
    draft_settings.value = value;
  },
});

onMounted(async () => {
  await load_settings();
  draft_settings.value = { ...settings.value };
  try {
    runs_count.value = await invoke<number>("count_runs");
  } catch {
    runs_count.value = 0;
  }
  await refresh_sidecar_status();
});

async function refresh_sidecar_status() {
  try {
    const result = await invoke<{ ok: boolean; detail: string }>("sidecar_health");
    sidecar_status.value = result.ok ? "online" : `offline · ${result.detail}`;
  } catch (err) {
    sidecar_status.value = `offline · ${String(err)}`;
  }
}

function open_settings() {
  draft_settings.value = { ...settings.value };
  settings_message.value = "";
  sidecar_message.value = "";
  show_settings.value = true;
}

async function on_save_settings() {
  if (!draft_settings.value) return;
  busy.value = true;
  settings_message.value = "";
  try {
    await save_settings(draft_settings.value);
    settings_message.value = "已保存到本地 SQLite";
    await refresh_sidecar_status();
  } catch (err) {
    settings_message.value = `保存失败：${String(err)}`;
  } finally {
    busy.value = false;
  }
}

async function on_test_sidecar() {
  if (!draft_settings.value) return;
  busy.value = true;
  sidecar_message.value = "";
  try {
    await save_settings(draft_settings.value);
    const result = await invoke<{ ok: boolean; detail: string }>("sidecar_echo", {
      payload: { ping: "rgain", ts: Date.now() },
    });
    sidecar_message.value = result.detail;
    sidecar_status.value = result.ok ? "online" : `offline · ${result.detail}`;
  } catch (err) {
    sidecar_message.value = String(err);
    sidecar_status.value = `offline · ${String(err)}`;
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="app_shell">
    <header class="top_bar">
      <div class="brand_block">
        <span class="brand_mark" aria-hidden="true" />
        <span class="brand">Rgain</span>
        <span class="subtitle">brainstorm · local logic map</span>
      </div>
      <div class="top_actions">
        <span class="pill">黑白 · 青红</span>
        <button type="button" class="btn_bar" @click="open_settings">设置</button>
      </div>
    </header>

    <div class="workspace">
      <LeftPanel
        project_label="未打开项目"
        :runs_count="runs_count"
        @open_settings="open_settings"
      />
      <ScenePlaceholder :sidecar_status="sidecar_status" />
      <ExplainPanel selected_label="" />
    </div>

    <SettingsPanel
      v-if="show_settings"
      v-model:model_value="editing_settings"
      :busy="busy"
      :message="settings_message"
      :sidecar_message="sidecar_message"
      @save="on_save_settings"
      @test_sidecar="on_test_sidecar"
      @close="show_settings = false"
    />
  </div>
</template>

<style scoped>
.app_shell {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: var(--color_surface_2);
  color: var(--color_ink);
}

.top_bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header_height);
  padding: 0 var(--space_md);
  border-bottom: 1px solid var(--color_line);
  background: color-mix(in srgb, var(--color_surface) 92%, transparent);
}

.brand_block {
  display: flex;
  align-items: center;
  gap: 0.65rem;
}

.brand_mark {
  width: 0.55rem;
  height: 0.55rem;
  border-radius: 2px;
  background: var(--color_brand);
  box-shadow: 0 0 0 3px color-mix(in srgb, #ff1b1b 22%, transparent);
}

.brand {
  font-size: 1.05rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--color_ink);
}

.subtitle {
  color: var(--color_muted);
  font-size: 0.78rem;
}

.top_actions {
  display: flex;
  align-items: center;
  gap: var(--space_sm);
}

.pill {
  padding: 0.2rem 0.55rem;
  border-radius: 999px;
  border: 1px solid var(--color_line);
  color: var(--color_accent_deep);
  font-size: 0.72rem;
}

.btn_bar {
  border-radius: var(--radius_sm);
  border: 1px solid var(--color_line);
  padding: 0.35rem 0.7rem;
  background: transparent;
  color: var(--color_ink);
  transition: border-color 0.2s var(--ease_soft), color 0.2s var(--ease_soft);
}

.btn_bar:hover {
  border-color: var(--color_accent);
  color: var(--color_accent);
}

.workspace {
  display: flex;
  flex: 1;
  min-height: 0;
}
</style>
