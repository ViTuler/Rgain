<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import HotkeyCapture from "../chrome/HotkeyCapture.vue";
import UiSwitch from "../chrome/UiSwitch.vue";
import { locale_options, set_locale, use_i18n, type UiLocale } from "../../i18n";
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
    llm_ok: boolean | null;
}>();

const emit = defineEmits<{
    "update:model_value": [AppSettings];
    save: [];
    test_sidecar: [];
    test_llm: [];
    reset_prompt: [];
    close: [];
}>();

const { t } = use_i18n();
const draft = reactive<AppSettings>({ ...props.model_value });
const options = computed(() => locale_options.value);

watch(
    () => props.model_value,
    (value) => Object.assign(draft, value),
    { deep: true },
);

function on_input() {
    emit("update:model_value", { ...draft });
}

function on_locale_change(event: Event) {
    const value = (event.target as HTMLSelectElement).value as UiLocale;
    draft.ui_locale = value;
    set_locale(value);
    on_input();
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
        <section class="sheet" role="dialog" :aria-labelledby="'settings_title'">
            <header>
                <h1 id="settings_title">{{ t("settings.title") }}</h1>
            </header>

            <div class="sheet_scroll">
                <section class="group">
                    <h2>{{ t("settings.general") }}</h2>
                    <label>
                        {{ t("locale.label") }}
                        <select :value="draft.ui_locale" @change="on_locale_change">
                            <option
                                v-for="opt in options"
                                :key="opt.value"
                                :value="opt.value"
                            >
                                {{ opt.label }}
                            </option>
                        </select>
                        <span class="hint">{{ t("locale.hint") }}</span>
                    </label>
                    <label>
                        {{ t("settings.dev_port", { min: MIN_DEV_PORT }) }}
                        <input
                            v-model.number="draft.dev_port"
                            type="number"
                            :min="MIN_DEV_PORT"
                            max="65535"
                            step="1"
                            @input="on_input"
                        />
                        <span class="hint">{{ t("settings.dev_port_hint") }}</span>
                    </label>
                    <label class="switch_row">
                        <span class="switch_copy">
                            <span class="switch_title">{{ t("settings.selection_explain") }}</span>
                            <span class="hint">{{ t("settings.selection_explain_hint") }}</span>
                        </span>
                        <UiSwitch
                            :model_value="draft.selection_explain_enabled"
                            :label="t('settings.selection_explain')"
                            @update:model_value="
                                (v) => {
                                    draft.selection_explain_enabled = v;
                                    on_input();
                                }
                            "
                        />
                    </label>
                    <label>
                        {{ t("settings.explain_hotkey") }}
                        <HotkeyCapture
                            :model_value="draft.explain_hotkey"
                            @update:model_value="
                                (v) => {
                                    draft.explain_hotkey = v;
                                    on_input();
                                }
                            "
                            @confirm="emit('save')"
                        />
                        <span class="hint">{{ t("settings.explain_hotkey_hint") }}</span>
                    </label>
                </section>

                <section class="group">
                    <div class="group_head">
                        <h2>{{ t("settings.llm") }}</h2>
                        <span
                            v-if="llm_message"
                            class="llm_badge"
                            :data-ok="llm_ok === true ? '1' : '0'"
                            :title="llm_message"
                            :aria-label="llm_message"
                        >
                            <span class="llm_dot" aria-hidden="true" />
                            <span class="llm_text">{{ llm_message }}</span>
                        </span>
                        <button
                            type="button"
                            class="btn_inline"
                            :disabled="busy"
                            @click="emit('test_llm')"
                        >
                            {{ t("settings.test_llm") }}
                        </button>
                    </div>
                    <label>
                        API Base URL
                        <input v-model="draft.api_base_url" @input="on_input" />
                    </label>
                    <label>
                        API Key
                        <input
                            v-model="draft.api_key"
                            class="api_key_input"
                            type="text"
                            autocomplete="off"
                            spellcheck="false"
                            :placeholder="
                                draft.api_key_set
                                    ? t('settings.api_key_placeholder_set')
                                    : t('settings.api_key_placeholder')
                            "
                            @focus="on_api_key_focus"
                            @blur="on_api_key_blur"
                            @input="on_input"
                        />
                    </label>
                    <label>
                        Model
                        <input v-model="draft.model" @input="on_input" />
                    </label>
                    <label class="prompt_label">
                        <span class="prompt_head">
                            {{ t("settings.agent_prompt") }}
                            <button
                                type="button"
                                class="btn_link"
                                :disabled="busy"
                                @click="emit('reset_prompt')"
                            >
                                {{ t("settings.clear_prompt") }}
                            </button>
                        </span>
                        <textarea
                            v-model="draft.agent_prompt"
                            rows="10"
                            :placeholder="t('settings.agent_prompt_placeholder')"
                            @input="on_input"
                        />
                        <span class="hint">{{ t("settings.agent_prompt_hint") }}</span>
                    </label>
                </section>

                <section class="group muted_group">
                    <h2>
                        {{ t("settings.mcp") }}
                        <span class="badge">{{ t("settings.mcp_badge") }}</span>
                    </h2>
                    <label class="check_row">
                        <input
                            v-model="draft.mcp_enabled"
                            type="checkbox"
                            disabled
                            @change="on_input"
                        />
                        <span>{{ t("settings.mcp_enable") }}</span>
                    </label>
                    <label>
                        MCP Server URL
                        <input
                            v-model="draft.mcp_server_url"
                            disabled
                            :placeholder="t('settings.mcp_url_placeholder')"
                            @input="on_input"
                        />
                    </label>
                    <p class="hint">{{ t("settings.mcp_hint") }}</p>
                </section>
            </div>

            <div class="actions">
                <!-- <button
                    type="button"
                    class="btn_ghost"
                    :disabled="busy"
                    @click="emit('test_sidecar')"
                >
                    {{ t("settings.test_sidecar") }}
                </button> -->
                <button
                    type="button"
                    class="btn_accent"
                    :disabled="busy"
                    @click="emit('save')"
                >
                    {{ t("common.save") }}
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
    gap: 0.5rem;
}

.group_head .btn_inline {
    margin-left: auto;
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

.switch_row {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
}

.switch_copy {
    display: flex;
    flex: 1;
    min-width: 0;
    flex-direction: column;
    gap: 0.2rem;
}

.switch_title {
    color: var(--color_muted);
    font-size: 0.8rem;
}

.prompt_head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
}

input,
textarea,
select {
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
textarea:disabled,
select:disabled {
    opacity: 0.55;
    cursor: not-allowed;
}

.hint {
    font-size: 0.72rem;
    color: var(--color_faint);
}

input:focus,
textarea:focus,
select:focus {
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

.msg.sidecar {
    color: var(--color_body);
    white-space: pre-wrap;
    word-break: break-word;
}

.llm_badge {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 0;
    max-width: min(14rem, 42%);
    padding: 0.18rem 0.55rem 0.18rem 0.4rem;
    border: 1px solid var(--color_line);
    border-radius: 999px;
    background: var(--color_surface_2);
    color: var(--color_muted);
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    user-select: none;
}

.llm_dot {
    width: 7px;
    height: 7px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: #c44b4b;
    box-shadow: 0 0 0 2px color-mix(in srgb, #c44b4b 22%, transparent);
}

.llm_badge[data-ok="1"] .llm_dot {
    background: #2f9e6b;
    box-shadow: 0 0 0 2px color-mix(in srgb, #2f9e6b 22%, transparent);
}

.llm_badge[data-ok="1"] {
    color: var(--color_ink);
}

.llm_text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
