<script setup lang="ts">
withDefaults(
    defineProps<{
        title: string;
        expanded: boolean;
        badge?: string | number;
    }>(),
    {
        badge: undefined,
    },
);

const emit = defineEmits<{
    toggle: [];
}>();
</script>

<template>
    <section class="view_pane" :class="{ expanded }">
        <div class="pane_header" role="button" tabindex="0" :aria-label="`${title} Section`" :aria-expanded="expanded"
            @click="emit('toggle')" @keydown.enter.prevent="emit('toggle')" @keydown.space.prevent="emit('toggle')">
            <span class="twisty" :class="{ open: expanded }" aria-hidden="true" />
            <span class="pane_icon" aria-hidden="true">
                <slot name = "icon" />
            </span>
            <h3 class="pane_title">{{ title }}</h3>
            <span v-if="badge !== undefined && badge !== ''" class="pane_badge">{{ badge }}</span>
            <div class="pane_actions" @click.stop>
                <slot name = "actions" />
            </div>
        </div>

        <div v-show="expanded" class="pane_body">
            <slot />
        </div>
    </section>
</template>

<style scoped>
.view_pane {
    display: flex;
    flex-direction: column;
    min-height: 22px;
}

.view_pane.expanded {
    min-height: 0;
}

.pane_header {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    height: 22px;
    padding: 0 0.35rem 0 0.15rem;
    border-top: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface) 92%, var(--color_surface_2));
    color: var(--color_muted);
    cursor: pointer;
    user-select: none;
    line-height: 22px;
}

.pane_header:hover {
    color: var(--color_ink);
}

.pane_header:focus-visible {
    outline: 1px solid var(--color_accent_deep);
    outline-offset: -1px;
}

.twisty {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    position: relative;
}

.twisty::before {
    content: "";
    position: absolute;
    left: 5px;
    top: 4px;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 4px 0 4px 6px;
    border-color: transparent transparent transparent currentColor;
    opacity: 0.85;
    transform: rotate(0deg);
    transition: transform 0.12s var(--ease_soft);
}

.twisty.open::before {
    transform: rotate(90deg);
}

.pane_icon {
    display: inline-flex;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    color: var(--color_muted);
}

.pane_icon :deep(svg) {
    width: 14px;
    height: 14px;
    display: block;
}

.pane_title {
    margin: 0;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: inherit;
}

.pane_badge {
    margin-right: 0.2rem;
    padding: 0 0.35rem;
    border-radius: 8px;
    background: color-mix(in srgb, var(--color_accent) 18%, transparent);
    color: var(--color_accent_deep);
    font-size: 10px;
    font-weight: 600;
    line-height: 16px;
}

.pane_actions {
    display: flex;
    align-items: center;
    gap: 0.1rem;
    opacity: 0;
    transition: opacity 0.12s var(--ease_soft);
}

.pane_header:hover .pane_actions,
.pane_header:focus-within .pane_actions {
    opacity: 1;
}

.pane_body {
    flex: 1;
    min-height: 0;
    overflow: auto;
}
</style>
