<script setup lang="ts">
import { reactive, watch } from "vue";
import {
    looks_like_masked_api_key,
    MIN_DEV_PORT,
    type AppSettings,
} from "../../types/settings";

const props = defineProps<{
    model_value: AppSettings;
    busy: boolean;
    message: string;
    sidecar_message: string;
    llm_message: string;
}>();

const emit = defineEmits<{
    "update:model_value": [AppSettings];
    save: [];
    test_sidecar: [];
    test_llm: [];
    reset_prompt: [];
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

function on_api_key_focus() {
    if (looks_like_masked_api_key(draft.api_key)) {
        draft.api_key = "";
        on_input();
    }
}

function on_api_key_blur() {
    if (!draft.api_key.trim() && draft.api_key_set) {
        draft.api_key = props.model_value.api_key || "";
        on_input();
    }
}
</script>

<template>
    <div class="overlay" @click.self="emit('close')">
        <section class="sheet" role="dialog" aria-labelledby="settings_title">
            <header>
                <h1 id="settings_title">设置</h1>
            </header>

            <div class="sheet_scroll">
                <section class="group">
                    <h2>通用</h2>
                    <label>
                        开发监听端口（≥ {{ MIN_DEV_PORT }}）
                        <input v-model.number="draft.dev_port" type="number" :min="MIN_DEV_PORT" max="65535" step="1"
                            @input="on_input" />
                        <span class="hint">保存后写入 rgain.dev.json；需重启 tauri dev 生效</span>
                    </label>
                    <label>
                        划词解释快捷键
                        <input
                            v-model="draft.explain_hotkey"
                            placeholder="Ctrl+Shift+E"
                            @input="on_input"
                        />
                        <span class="hint">全局快捷键；保存后立即重注册。选中文本后触发解释。</span>
                    </label>
                </section>

                <section class="group">
                    <div class="group_head">
                        <h2>LLM API</h2>
                        <button type="button" class="btn_inline" :disabled="busy" @click="emit('test_llm')">
                            测试连接
                        </button>
                    </div>
                    <label>
                        API Base URL
                        <input v-model="draft.api_base_url" @input="on_input" />
                    </label>
                    <label>
                        API Key
                        <input v-model="draft.api_key" class="api_key_input" type="text" autocomplete="off"
                            spellcheck="false" :placeholder="draft.api_key_set ? 'sk****xxxx' : '输入 API Key'"
                            @focus="on_api_key_focus" @blur="on_api_key_blur" @input="on_input" />
                    </label>
                    <label>
                        Model
                        <input v-model="draft.model" @input="on_input" />
                    </label>
                    <p v-if="llm_message" class="msg llm">{{ llm_message }}</p>
                    <label class="prompt_label">
                        <span class="prompt_head">
                            Agent Prompt
                            <button type="button" class="btn_link" :disabled="busy" @click="emit('reset_prompt')">
                                恢复内置
                            </button>
                        </span>
                        <textarea v-model="draft.agent_prompt" rows="10" placeholder="全局 system prompt，对所有项目生效"
                            @input="on_input" />
                        <span class="hint">全局共用；解析时作为 system</span>
                    </label>
                </section>

                <section class="group muted_group">
                    <h2>MCP <span class="badge">预设</span></h2>
                    <label class="check_row">
                        <input v-model="draft.mcp_enabled" type="checkbox" disabled @change="on_input" />
                        <span>启用 MCP Server（Phase E）</span>
                    </label>
                    <label>
                        MCP Server URL
                        <input v-model="draft.mcp_server_url" disabled placeholder="预留 · 例如 http://127.0.0.1:3100"
                            @input="on_input" />
                    </label>
                    <p class="hint">二期：对外暴露 list_runs / get_subgraph / search_nodes</p>
                </section>
            </div>

            <div class="actions">
                <button type="button" class="btn_ghost" :disabled="busy" @click="emit('test_sidecar')">
                    测试 Sidecar
                </button>
                <button type="button" class="btn_accent" :disabled="busy" @click="emit('save')">
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
    background: rgb(var(--ink_rgb) / 0.62);
}

.sheet {
    width: min(520px, 100%);
    max-height: min(88vh, 760px);
    display: flex;
    flex-direction: column;
    gap: var(--space_sm);
    padding: var(--space_lg);
    border-radius: var(--radius_lg);
    border: 1px solid var(--color_line);
    background: var(--color_surface_3);
    box-shadow: var(--shadow_panel);
}

.sheet_scroll {
    display: flex;
    flex-direction: column;
    gap: var(--space_md);
    overflow: auto;
    min-height: 0;
    padding-right: 0.15rem;
}

header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

h1 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--color_ink);
}

.group {
    display: flex;
    flex-direction: column;
    gap: var(--space_sm);
    padding-bottom: var(--space_sm);
    border-bottom: 1px solid var(--color_line);
}

.group:last-child {
    border-bottom: 0;
    padding-bottom: 0;
}

.group_head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
}

.group h2 {
    margin: 0;
    font-size: 0.78rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color_ink);
}

.btn_inline {
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    line-height: 1;
    color: var(--color_accent_deep);
    cursor: pointer;
}

.btn_inline:hover:not(:disabled) {
    color: var(--color_accent);
}

.btn_inline:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.badge {
    margin-left: 0.35rem;
    padding: 0.05rem 0.35rem;
    border-radius: 999px;
    border: 1px solid var(--color_line);
    font-size: 0.65rem;
    font-weight: 600;
    letter-spacing: 0;
    text-transform: none;
    color: var(--color_faint);
}

.muted_group {
    opacity: 0.85;
}

label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.8rem;
    color: var(--color_muted);
}

.check_row {
    flex-direction: row;
    align-items: center;
    gap: 0.45rem;
}

.prompt_head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
}

input,
textarea {
    border-radius: var(--radius_sm);
    border: 1px solid var(--color_line);
    padding: 0.55rem 0.7rem;
    background: var(--color_surface_2);
    color: var(--color_ink);
    font: inherit;
    tab-size: 4;
}

.api_key_input {
    font-family: ui-monospace, "Cascadia Code", "Consolas", monospace;
    letter-spacing: 0.02em;
}

textarea {
    resize: vertical;
    min-height: 10rem;
    line-height: 1.45;
    tab-size: 4;
}

input:disabled,
textarea:disabled {
    opacity: 0.55;
    cursor: not-allowed;
}

.hint {
    font-size: 0.72rem;
    color: var(--color_faint);
}

.hint code {
    font-size: 0.7rem;
}

input:focus,
textarea:focus {
    border-color: var(--color_accent_deep);
    outline: none;
}

.actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space_sm);
    margin-top: var(--space_xs);
}

.btn_accent,
.btn_ghost,
.btn_link {
    border-radius: var(--radius_sm);
    border: 1px solid transparent;
    padding: 0.5rem 0.85rem;
}

.btn_accent {
    color: var(--color_on_accent);
    background: var(--color_accent);
}

.btn_ghost {
    color: var(--color_ink);
    background: transparent;
    border-color: var(--color_line);
}

.btn_link {
    padding: 0.15rem 0.4rem;
    font-size: 0.72rem;
    color: var(--color_accent_deep);
    background: transparent;
}

.btn_link:disabled,
.btn_ghost:disabled,
.btn_accent:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.msg {
    margin: 0;
    font-size: 0.8rem;
    color: var(--color_accent_deep);
}

.msg.llm,
.msg.sidecar {
    color: var(--color_body);
    white-space: pre-wrap;
    word-break: break-word;
}
</style>
