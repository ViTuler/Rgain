<script setup lang="ts">
import { reactive, watch } from "vue";
import type { AppSettings } from "../../types/settings";

const props = defineProps<{
  model_value: AppSettings;
  busy: boolean;
  message: string;
  sidecar_message: string;
}>();

const emit = defineEmits<{
  "update:model_value": [AppSettings];
  save: [];
  test_sidecar: [];
  close: [];
}>();

const draft = reactive<AppSettings>({ ...props.model_value });

watch(
  () => props.model_value,
  (value) => Object.assign(draft, value),
  { deep: true },
);

function on_input() {
  emit("update:model_value", { ...draft });
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <section class="sheet" role="dialog" aria-labelledby="settings_title">
      <header>
        <h1 id="settings_title">设置</h1>
        <button type="button" class="btn_icon" @click="emit('close')">×</button>
      </header>

      <label>
        API Base URL
        <input v-model="draft.api_base_url" @input="on_input" />
      </label>

      <label>
        API Key
        <input
          v-model="draft.api_key"
          type="password"
          autocomplete="off"
          placeholder="不会提交到仓库"
          @input="on_input"
        />
      </label>

      <label>
        Model
        <input v-model="draft.model" @input="on_input" />
      </label>

      <label>
        Python 路径（conda vi）
        <input v-model="draft.python_path" @input="on_input" />
      </label>

      <label>
        划词解释快捷键（占位）
        <input v-model="draft.explain_hotkey" @input="on_input" />
      </label>

      <div class="actions">
        <button type="button" class="btn_ghost" :disabled="busy" @click="emit('test_sidecar')">
          测试 Sidecar
        </button>
        <button type="button" class="btn_mint" :disabled="busy" @click="emit('save')">
          保存
        </button>
      </div>

      <p v-if="message" class="msg">{{ message }}</p>
      <p v-if="sidecar_message" class="msg sidecar">{{ sidecar_message }}</p>
    </section>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space_lg);
  background: color-mix(in srgb, #0e1210 72%, transparent);
}

.sheet {
  width: min(440px, 100%);
  display: flex;
  flex-direction: column;
  gap: var(--space_sm);
  padding: var(--space_lg);
  border-radius: var(--radius_md);
  border: 1px solid var(--color_border);
  background: var(--color_bg_panel);
  box-shadow: 0 18px 48px color-mix(in srgb, #000 45%, transparent);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space_xs);
}

h1 {
  margin: 0;
  font-size: 1.1rem;
}

label {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.8rem;
  color: var(--color_text_muted);
}

input {
  border-radius: var(--radius_sm);
  border: 1px solid var(--color_border);
  padding: 0.55rem 0.7rem;
  background: var(--color_bg_deep);
  color: var(--color_text_on_deep);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space_sm);
  margin-top: var(--space_sm);
}

.btn_mint,
.btn_ghost,
.btn_icon {
  border-radius: var(--radius_sm);
  border: 1px solid transparent;
  padding: 0.5rem 0.85rem;
}

.btn_mint {
  color: var(--color_text_on_mint);
  background: var(--color_accent_mint);
}

.btn_ghost {
  color: var(--color_text_on_deep);
  background: transparent;
  border-color: var(--color_border);
}

.btn_icon {
  width: 2rem;
  height: 2rem;
  padding: 0;
  font-size: 1.2rem;
  line-height: 1;
  color: var(--color_text_muted);
  background: transparent;
}

.msg {
  margin: 0;
  font-size: 0.8rem;
  color: var(--color_accent_mint);
}

.msg.sidecar {
  color: var(--color_text_on_deep);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
