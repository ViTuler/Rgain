<script setup lang="ts">
import { ref, watch } from "vue";
import {
    format_hotkey_event,
    is_cancel_key,
    is_confirm_key,
} from "../../lib/hotkey_format";
import { use_i18n } from "../../i18n";

const props = defineProps<{
    model_value: string;
    placeholder?: string;
}>();

const emit = defineEmits<{
    "update:model_value": [string];
    confirm: [];
}>();

const { t } = use_i18n();
const listening = ref(false);
const draft = ref(props.model_value);

watch(
    () => props.model_value,
    (value) => {
        if (!listening.value) draft.value = value;
    },
);

function on_focus() {
    listening.value = true;
    draft.value = props.model_value;
}

function on_blur() {
    listening.value = false;
    if (draft.value.trim()) {
        emit("update:model_value", draft.value.trim());
    } else {
        draft.value = props.model_value;
    }
}

function on_keydown(event: KeyboardEvent) {
    if (!listening.value) return;
    event.preventDefault();
    event.stopPropagation();

    if (is_cancel_key(event)) {
        draft.value = props.model_value;
        listening.value = false;
        (event.target as HTMLInputElement).blur();
        return;
    }

    if (is_confirm_key(event)) {
        const next = draft.value.trim() || props.model_value;
        draft.value = next;
        emit("update:model_value", next);
        listening.value = false;
        (event.target as HTMLInputElement).blur();
        emit("confirm");
        return;
    }

    const combo = format_hotkey_event(event);
    if (!combo) return;
    draft.value = combo;
    emit("update:model_value", combo);
}

function display_value(): string {
    if (listening.value && !draft.value) return "";
    return draft.value || props.model_value;
}
</script>

<template>
    <div class="hotkey_capture" :data-listening="listening ? '1' : '0'">
        <input
            type="text"
            class="hotkey_input"
            readonly
            :value="display_value()"
            :placeholder="listening ? t('settings.hotkey_listening') : (placeholder || 'Ctrl+Shift+E')"
            :aria-label="t('settings.explain_hotkey')"
            @focus="on_focus"
            @blur="on_blur"
            @keydown="on_keydown"
        />
        <span class="hotkey_hint_inline">
            {{ listening ? t("settings.hotkey_listening_hint") : t("settings.hotkey_click_hint") }}
        </span>
    </div>
</template>

<style scoped>
.hotkey_capture {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
}

.hotkey_input {
    border-radius: var(--radius_sm);
    border: 1px solid var(--color_line);
    padding: 0.55rem 0.7rem;
    background: var(--color_surface_2);
    color: var(--color_ink);
    font: inherit;
    cursor: pointer;
}

.hotkey_capture[data-listening="1"] .hotkey_input {
    border-color: var(--color_accent_deep);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color_accent) 22%, transparent);
    cursor: text;
}

.hotkey_input:focus {
    outline: none;
}

.hotkey_hint_inline {
    font-size: 0.7rem;
    color: var(--color_faint);
}
</style>
