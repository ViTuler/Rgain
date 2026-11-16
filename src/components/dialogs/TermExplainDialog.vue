<script setup lang="ts">
import MarkdownBody from "../chrome/MarkdownBody.vue";
import { use_i18n } from "../../i18n";

defineProps<{
    term: string;
    explanation: string;
    context?: string;
    busy?: boolean;
    error?: string;
    hotkey_hint?: string;
}>();

const emit = defineEmits<{
    close: [];
}>();

const { t } = use_i18n();
</script>

<template>
    <div class="overlay" @click.self="emit('close')">
        <section class="sheet" role="dialog" aria-labelledby="term_title">
            <header>
                <div>
                    <span class="eyebrow">{{ t("term.eyebrow") }}</span>
                    <h1 id="term_title">{{ term || "—" }}</h1>
                </div>
                <button type="button" class="btn_icon" :aria-label="t('common.close')" @click="emit('close')">
                    ×
                </button>
            </header>

            <p v-if="context" class="context">{{ t("term.context", { text: context }) }}</p>
            <p v-if="hotkey_hint" class="hint">
                {{ t("term.hotkey_hint", { hotkey: hotkey_hint }) }}
            </p>

            <div class="body">
                <p v-if="busy" class="muted">{{ t("term.busy") }}</p>
                <p v-else-if="error" class="error">{{ error }}</p>
                <MarkdownBody v-else-if="explanation" size="term" :source="explanation" />
                <p v-else class="muted">{{ t("term.empty") }}</p>
            </div>
        </section>
    </div>
</template>

<style scoped>
.overlay {
    position: absolute;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space_lg);
    overflow: hidden;
    background: rgb(var(--ink_rgb) / 0.55);
}

.sheet {
    width: min(46rem, 100%);
    max-height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space_sm);
    padding: var(--space_lg);
    overflow: hidden;
    border-radius: var(--radius_lg);
    border: 1px solid var(--color_line);
    background: var(--color_surface_3);
    box-shadow: var(--shadow_panel);
}

header,
.context,
.hint {
    flex-shrink: 0;
}

header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space_sm);
}

.eyebrow {
    display: block;
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color_brand);
}

h1 {
    margin: 0.2rem 0 0;
    font-size: 1.15rem;
    color: var(--color_ink);
    overflow-wrap: anywhere;
}

.context {
    margin: 0;
    font-size: 0.78rem;
    color: var(--color_muted);
}

.hint {
    margin: 0;
    font-size: 0.72rem;
    color: var(--color_faint);
}

.body {
    flex: 1 1 auto;
    min-height: 4.5rem;
    padding: var(--space_md);
    overflow: auto;
    overscroll-behavior: contain;
    border-radius: var(--radius_md);
    background: var(--color_surface_2);
    border: 1px solid var(--color_line);
    color: var(--color_body);
    line-height: 1.55;
}

.muted {
    margin: 0;
    color: var(--color_faint);
}

.error {
    margin: 0;
    color: var(--color_brand);
}

.btn_icon {
    width: 2rem;
    height: 2rem;
    padding: 0;
    border: 0;
    border-radius: var(--radius_sm);
    font-size: 1.2rem;
    line-height: 1;
    color: var(--color_muted);
    background: transparent;
}
</style>
