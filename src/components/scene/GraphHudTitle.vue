<script setup lang="ts">
import { ref, watch } from "vue";
import { use_i18n } from "../../i18n";

const props = withDefaults(
    defineProps<{
        title: string;
        summary?: string;
        /** light = 2D surface HUD; dark = 3D night HUD */
        tone?: "light" | "dark";
        default_open?: boolean;
    }>(),
    {
        summary: "",
        tone: "light",
        default_open: true,
    },
);

const { t } = use_i18n();
const summary_open = ref(props.default_open);

watch(
    () => [props.title, props.summary] as const,
    () => {
        // New graph: reopen summary so the fresh synopsis is visible.
        summary_open.value = props.default_open;
    },
);

function toggle_summary() {
    summary_open.value = !summary_open.value;
}
</script>

<template>
    <div class="hud_title" :data-tone="tone">
        <button
            type="button"
            class="title_toggle"
            :aria-expanded="summary_open"
            :aria-label="summary_open ? t('common.collapse_summary') : t('common.expand_summary')"
            @click="toggle_summary"
        >
            <span class="chevron" :class="{ open: summary_open }" aria-hidden="true">
                <svg viewBox="0 0 16 16">
                    <path
                        fill="currentColor"
                        d="M4.646 6.646a.5.5 0 0 1 .708 0L8 9.293l2.646-2.647a.5.5 0 0 1 .708.708l-3 3a.5.5 0 0 1-.708 0l-3-3a.5.5 0 0 1 0-.708z"
                    />
                </svg>
            </span>
            <h1 class="title">{{ title }}</h1>
        </button>

        <div class="summary_shell" :class="{ open: summary_open }">
            <div class="summary_inner">
                <p v-if="summary" class="desc">{{ summary }}</p>
                <p v-else class="desc muted">{{ t("common.no_summary") }}</p>
            </div>
        </div>
    </div>
</template>

<style scoped>
.hud_title {
    pointer-events: none;
}

.title_toggle {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    max-width: 100%;
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    pointer-events: auto;
}

.chevron {
    display: inline-flex;
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    transform: rotate(-90deg);
    transition:
        transform 0.22s ease,
        background 0.15s ease,
        color 0.15s ease;
}

.chevron.open {
    transform: rotate(0deg);
}

.chevron svg {
    width: 14px;
    height: 14px;
}

.title {
    margin: 0;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 1.05rem;
    font-weight: 650;
    line-height: 1.25;
}

.summary_shell {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows 0.28s ease;
}

.summary_shell.open {
    grid-template-rows: 1fr;
}

.summary_inner {
    min-height: 0;
    overflow: hidden;
}

.desc {
    margin: 0.35rem 0 0;
    font-size: 0.78rem;
    line-height: 1.4;
}

.desc.muted {
    opacity: 0.7;
}

/* light (2D) */
.hud_title[data-tone="light"] .chevron {
    color: var(--color_muted);
}

.hud_title[data-tone="light"] .title_toggle:hover .chevron {
    color: var(--color_ink);
    background: color-mix(in srgb, var(--color_accent) 12%, transparent);
}

.hud_title[data-tone="light"] .title {
    color: var(--color_ink);
}

.hud_title[data-tone="light"] .desc {
    color: var(--color_muted);
}

/* dark (3D) */
.hud_title[data-tone="dark"] .chevron {
    color: var(--color_on_dark_soft);
}

.hud_title[data-tone="dark"] .title_toggle:hover .chevron {
    color: var(--color_on_dark);
    background: rgb(var(--bright_rgb) / 0.1);
}

.hud_title[data-tone="dark"] .title {
    color: var(--color_on_dark);
}

.hud_title[data-tone="dark"] .desc {
    color: var(--color_on_dark_soft);
}
</style>
