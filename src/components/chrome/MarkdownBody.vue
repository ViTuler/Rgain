<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import { computed } from "vue";
import { render_markdown } from "../../lib/render_markdown";

const props = withDefaults(
    defineProps<{
        source: string;
        size?: "explain" | "term";
    }>(),
    { size: "explain" },
);

const html = computed(() => render_markdown(props.source));

function on_click(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const anchor = target.closest("a");
    if (!anchor) return;
    const href = anchor.getAttribute("href")?.trim() ?? "";
    if (!/^https?:\/\//i.test(href)) return;
    event.preventDefault();
    void openUrl(href);
}
</script>

<template>
    <div
        class="md_body"
        :class="size === 'term' ? 'md_term' : 'md_explain'"
        v-html="html"
        @click="on_click"
    />
</template>

<style scoped>
.md_body {
    overflow-x: auto;
    color: var(--color_body);
    line-height: 1.55;
}

.md_explain {
    font-size: var(--font_explain_px, 14px);
}

.md_term {
    font-size: var(--font_term_px, 14px);
}

.md_body :deep(p) {
    margin: 0.45rem 0;
}

.md_body :deep(p:first-child) {
    margin-top: 0;
}

.md_body :deep(p:last-child) {
    margin-bottom: 0;
}

.md_body :deep(h1),
.md_body :deep(h2),
.md_body :deep(h3),
.md_body :deep(h4) {
    margin: 0.75rem 0 0.3rem;
    font-size: 1.05em;
    line-height: 1.35;
    color: var(--color_ink);
}

.md_body :deep(ul),
.md_body :deep(ol) {
    margin: 0.35rem 0;
    padding-left: 1.15rem;
}

.md_body :deep(li + li) {
    margin-top: 0.15rem;
}

.md_body :deep(code) {
    font-size: 0.8em;
    padding: 0.05rem 0.28rem;
    border-radius: 4px;
    background: var(--color_surface_2);
}

.md_body :deep(pre) {
    margin: 0.5rem 0;
    padding: 0.55rem 0.65rem;
    overflow-x: auto;
    border-radius: var(--radius_sm);
    background: var(--color_surface_2);
    border: 1px solid var(--color_line);
}

.md_body :deep(pre code) {
    padding: 0;
    background: transparent;
}

.md_body :deep(table) {
    width: max-content;
    max-width: 100%;
    border-collapse: collapse;
    margin: 0.5rem 0;
    font-size: 0.9em;
}

.md_body :deep(th),
.md_body :deep(td) {
    border: 1px solid var(--color_line_strong);
    padding: 0.28rem 0.45rem;
    text-align: left;
    vertical-align: top;
}

.md_body :deep(th) {
    background: var(--color_surface_2);
    color: var(--color_ink);
}

.md_body :deep(img) {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 0.45rem 0;
    border-radius: var(--radius_sm);
}

.md_body :deep(blockquote) {
    margin: 0.5rem 0;
    padding-left: 0.7rem;
    border-left: 2px solid var(--color_accent);
    color: var(--color_muted);
}

.md_body :deep(a) {
    color: var(--color_accent_deep);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 0.12em;
}

.md_body :deep(.katex-display) {
    margin: 0.55rem 0;
    overflow-x: auto;
    overflow-y: hidden;
}
</style>
