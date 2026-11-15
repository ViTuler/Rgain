<script setup lang="ts">
import { ref } from "vue";
import IconButton from "../chrome/IconButton.vue";
import ViewPane from "./ViewPane.vue";
import type { RunRow } from "../../types/project";

defineProps<{
    project_label: string;
    project_path: string;
    runs: RunRow[];
    active_run_id: number | null;
    ingest_busy: boolean;
    ingest_message: string;
}>();

const emit = defineEmits<{
    open_settings: [];
    pick_folder: [];
    run_ingest: [];
    open_run: [run_id: number];
}>();

const explorer_open = ref(true);
const history_open = ref(true);
</script>

<template>
    <aside class="left_panel">
        <div class="sidebar_title" :title="project_path || project_label">
            <span class="sidebar_label">EXPLORER</span>
        </div>

        <div class="pane_stack">
            <ViewPane class="pane_explorer" title="目录" :expanded="explorer_open"
                @toggle="explorer_open = !explorer_open">
                <template #icon>
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="currentColor"
                            d="M1.5 2A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h13a1.5 1.5 0 0 0 1.5-1.5v-7A1.5 1.5 0 0 0 14.5 4H7.707L6.353 2.646A.5.5 0 0 0 6 2.5H1.5zm0 1h4.293l1.353 1.354A.5.5 0 0 0 7.5 5h7a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-.5.5h-13a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5z" />
                    </svg>
                </template>
                <template #actions>
                    <button type="button" class="pane_action" title="选择目录" aria-label="选择目录" :disabled="ingest_busy"
                        @click="emit('pick_folder')">
                        <svg viewBox="0 0 16 16" aria-hidden="true">
                            <path fill="currentColor"
                                d="M8 2a.5.5 0 0 1 .5.5V7H13a.5.5 0 0 1 0 1H8.5v4.5a.5.5 0 0 1-1 0V8H3a.5.5 0 0 1 0-1h4.5V2.5A.5.5 0 0 1 8 2z" />
                        </svg>
                    </button>
                </template>

                <div class="pane_content">
                    <p class="muted">
                        {{ project_path ? project_path : "点标题旁 + 选择本地代码/书籍目录。" }}
                    </p>
                    <button
                        type="button"
                        class="btn_accent"
                        :disabled="ingest_busy || !project_path"
                        @click="emit('run_ingest')"
                    >
                        {{ ingest_busy ? "解析中…" : "开始解析" }}
                    </button>
                    <p class="muted">Agent Prompt 在设置 · LLM API 中全局配置。</p>
                    <p v-if="ingest_message" class="status_msg">{{ ingest_message }}</p>
                </div>
            </ViewPane>

            <ViewPane class="pane_history" title="历史" :expanded="history_open" :badge="runs.length"
                @toggle="history_open = !history_open">
                <template #icon>
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="currentColor"
                            d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zM0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8zm8.5-3.5a.5.5 0 0 0-1 0V8a.5.5 0 0 0 .146.354l2 2a.5.5 0 0 0 .708-.708L8.5 7.793V4.5z" />
                    </svg>
                </template>

                <div class="pane_content">
                    <ul v-if="runs.length" class="run_list">
                        <li v-for="run in runs" :key="run.id">
                            <button
                                type="button"
                                class="run_item"
                                :class="{ active: active_run_id === run.id }"
                                @click="emit('open_run', run.id)"
                            >
                                <span class="run_id">#{{ run.id }}</span>
                                <span class="run_status" :data-status="run.status">{{ run.status }}</span>
                                <span class="run_time">{{ run.created_at }}</span>
                            </button>
                        </li>
                    </ul>
                    <ul v-else class="empty_list">
                        <li>暂无历史 · 解析后在此回放</li>
                    </ul>
                </div>
            </ViewPane>
        </div>

        <footer class="panel_dock">
            <div class="dock_spacer" />
            <IconButton label="设置" @click="emit('open_settings')">
                <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path fill="currentColor"
                        d="M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6m0 4.5a1.5 1.5 0 1 1 .001-3.001A1.5 1.5 0 0 1 12 13.5m9.848 1.072l-1.93-1.63a1.232 1.232 0 0 1 0-1.884l1.93-1.63a.62.62 0 0 0 .189-.662a10.5 10.5 0 0 0-2.209-3.804a.62.62 0 0 0-.668-.168l-2.381.848a1.3 1.3 0 0 1-.419.073a1.24 1.24 0 0 1-1.219-1.014l-.454-2.478a.62.62 0 0 0-.482-.493a10.7 10.7 0 0 0-4.408 0a.62.62 0 0 0-.482.493l-.453 2.478a1.24 1.24 0 0 1-1.638.942l-2.38-.848a.62.62 0 0 0-.669.168a10.5 10.5 0 0 0-2.21 3.805a.615.615 0 0 0 .189.661l1.929 1.63q.077.066.143.143c.444.52.379 1.299-.143 1.742l-1.929 1.63a.62.62 0 0 0-.189.662a10.5 10.5 0 0 0 2.21 3.803a.62.62 0 0 0 .669.168l2.38-.847a1.3 1.3 0 0 1 .419-.072c.588 0 1.11.417 1.219 1.014l.453 2.478a.62.62 0 0 0 .482.493a10.7 10.7 0 0 0 4.408 0a.62.62 0 0 0 .482-.493l.454-2.478q.018-.099.053-.194a1.24 1.24 0 0 1 1.585-.748l2.379.848q.105.036.208.036a.62.62 0 0 0 .461-.204a10.5 10.5 0 0 0 2.208-3.804a.615.615 0 0 0-.189-.662zm-2.756 3.017l-1.81-.645a2.747 2.747 0 0 0-3.615 2.085l-.345 1.877a9.2 9.2 0 0 1-2.642-.002l-.344-1.876a2.737 2.737 0 0 0-3.616-2.085l-1.81.644a8.9 8.9 0 0 1-1.319-2.266l1.461-1.235c.56-.472.9-1.135.96-1.864a2.7 2.7 0 0 0-.645-1.995a3 3 0 0 0-.315-.315L3.591 8.677A8.9 8.9 0 0 1 4.91 6.412l1.81.645q.447.158.921.159a2.75 2.75 0 0 0 2.695-2.244l.344-1.876a9.3 9.3 0 0 1 2.642-.002l.343 1.877a2.737 2.737 0 0 0 3.617 2.085l1.81-.645c.552.69.993 1.449 1.319 2.266l-1.46 1.235a2.7 2.7 0 0 0-.96 1.864a2.71 2.71 0 0 0 .962 2.31l1.46 1.234a8.9 8.9 0 0 1-1.318 2.267z" />
                </svg>
            </IconButton>
        </footer>
    </aside>
</template>

<style scoped>
.left_panel {
    display: flex;
    flex-direction: column;
    width: var(--panel_left_width);
    min-width: var(--panel_left_width);
    border-right: 1px solid var(--color_line);
    background: var(--color_surface);
}

.sidebar_title {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    padding: 0.55rem 0.75rem 0.45rem;
}

.sidebar_label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--color_muted);
}

.sidebar_project {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    color: var(--color_ink);
}

.pane_stack {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
}

.pane_explorer {
    flex: 1 1 auto;
    min-height: 0;
}

.pane_explorer :deep(.pane_body) {
    flex: 1;
}

.pane_history {
    flex: 0 0 auto;
}

.pane_history.expanded {
    flex: 0 1 42%;
    min-height: 120px;
    max-height: 55%;
}

.pane_content {
    padding: 0.45rem 0.65rem 0.7rem;
}

.muted {
    margin: 0 0 0.45rem;
    color: var(--color_muted);
    font-size: 0.75rem;
    line-height: 1.4;
    word-break: break-all;
}

.empty_list {
    margin: 0;
    padding-left: 1rem;
    color: var(--color_faint);
    font-size: 0.75rem;
}

.btn_accent {
    width: 100%;
    margin-bottom: 0.45rem;
    border-radius: var(--radius_sm);
    border: 1px solid var(--color_accent);
    padding: 0.4rem 0.65rem;
    font-size: 0.78rem;
    color: var(--color_on_accent);
    background: var(--color_accent);
}

.btn_accent:disabled {
    opacity: 0.55;
    cursor: not-allowed;
}

.status_msg {
    margin: 0;
    color: var(--color_accent_deep);
    font-size: 0.72rem;
    white-space: pre-wrap;
    word-break: break-word;
}

.run_list {
    margin: 0;
    padding: 0;
    list-style: none;
}

.run_item {
    width: 100%;
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    gap: 0.1rem 0.45rem;
    margin-bottom: 0.25rem;
    padding: 0.35rem 0.4rem;
    border: 1px solid transparent;
    border-radius: var(--radius_sm);
    background: transparent;
    color: var(--color_body);
    text-align: left;
}

.run_item:hover {
    border-color: var(--color_line);
    background: var(--color_surface_2);
}

.run_item.active {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
    background: color-mix(in srgb, var(--color_accent) 10%, var(--color_surface_2));
}

.run_id {
    font-size: 0.72rem;
    color: var(--color_ink);
    font-weight: 600;
}

.run_status {
    justify-self: end;
    font-size: 0.68rem;
    color: var(--color_muted);
}

.run_status[data-status="ok"] {
    color: var(--color_accent_deep);
}

.run_status[data-status="failed"] {
    color: var(--color_brand);
}

.run_time {
    grid-column: 1 / -1;
    font-size: 0.68rem;
    color: var(--color_faint);
}

.pane_action {
    display: inline-flex;
    width: 20px;
    height: 20px;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: 3px;
    background: transparent;
    color: var(--color_muted);
}

.pane_action:hover:not(:disabled) {
    color: var(--color_ink);
    background: color-mix(in srgb, var(--color_accent) 12%, transparent);
}

.pane_action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.pane_action svg {
    width: 14px;
    height: 14px;
}

.panel_dock {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.2rem;
    padding: 0.35rem 0.45rem;
    border-top: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface) 88%, var(--color_surface_2));
}

.dock_spacer {
    flex: 1;
}
</style>
