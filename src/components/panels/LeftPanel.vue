<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { use_i18n } from "../../i18n";
import IconButton from "../chrome/IconButton.vue";
import ViewPane from "./ViewPane.vue";
import type { RunRow } from "../../types/project";

const props = defineProps<{
    project_label: string;
    project_path: string;
    runs: RunRow[];
    active_run_id: number | null;
    ingest_busy: boolean;
    ingest_message: string;
    sidecar_status: string;
}>();

const { t } = use_i18n();

const sidecar_online = computed(() => props.sidecar_status === "online");
const sidecar_label = computed(() =>
    sidecar_online.value
        ? t("explorer.sidecar_online")
        : t("explorer.sidecar_offline", { status: props.sidecar_status }),
);

const emit = defineEmits<{
    open_settings: [];
    pick_folder: [];
    run_ingest: [];
    open_run: [run_id: number];
    rename_run: [payload: { run_id: number; title: string }];
    delete_run: [run_id: number];
}>();

const explorer_open = ref(true);
const history_open = ref(true);
const editing_run_id = ref<number | null>(null);
const edit_draft = ref("");
const edit_input = ref<HTMLInputElement | null>(null);

function run_title(run: RunRow): string {
    const title = run.title?.trim();
    return title || `Run #${run.id}`;
}

function bind_edit_input(el: unknown) {
    edit_input.value = el instanceof HTMLInputElement ? el : null;
}

async function start_rename(run: RunRow, event: Event) {
    event.stopPropagation();
    event.preventDefault();
    editing_run_id.value = run.id;
    edit_draft.value = run_title(run);
    await nextTick();
    edit_input.value?.focus();
    edit_input.value?.select();
}

function cancel_rename() {
    editing_run_id.value = null;
    edit_draft.value = "";
}

function commit_rename(run: RunRow) {
    if (editing_run_id.value !== run.id) return;
    const next = edit_draft.value.trim();
    editing_run_id.value = null;
    if (!next || next === run_title(run)) {
        edit_draft.value = "";
        return;
    }
    emit("rename_run", { run_id: run.id, title: next });
    edit_draft.value = "";
}

function on_delete(run: RunRow, event: Event) {
    event.stopPropagation();
    event.preventDefault();
    emit("delete_run", run.id);
}

function on_open(run: RunRow) {
    if (editing_run_id.value === run.id) return;
    emit("open_run", run.id);
}
</script>

<template>
    <aside class="left_panel">
        <div class="sidebar_title" :title="project_path || project_label">
            <span class="sidebar_label">{{ t("explorer.title") }}</span>
        </div>

        <div class="pane_stack">
            <ViewPane class="pane_explorer" :title="t('explorer.dir')" :expanded="explorer_open"
                @toggle="explorer_open = !explorer_open">
                <template #icon>
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="currentColor"
                            d="M1.5 2A1.5 1.5 0 0 0 0 3.5v9A1.5 1.5 0 0 0 1.5 14h13a1.5 1.5 0 0 0 1.5-1.5v-7A1.5 1.5 0 0 0 14.5 4H7.707L6.353 2.646A.5.5 0 0 0 6 2.5H1.5zm0 1h4.293l1.353 1.354A.5.5 0 0 0 7.5 5h7a.5.5 0 0 1 .5.5v7a.5.5 0 0 1-.5.5h-13a.5.5 0 0 1-.5-.5v-9a.5.5 0 0 1 .5-.5z" />
                    </svg>
                </template>
                <template #actions>
                    <button type="button" class="pane_action" :title="t('explorer.pick_folder')"
                        :aria-label="t('explorer.pick_folder')" :disabled="ingest_busy"
                        @click="emit('pick_folder')">
                        <svg viewBox="0 0 16 16" aria-hidden="true">
                            <path fill="currentColor"
                                d="M8 2a.5.5 0 0 1 .5.5V7H13a.5.5 0 0 1 0 1H8.5v4.5a.5.5 0 0 1-1 0V8H3a.5.5 0 0 1 0-1h4.5V2.5A.5.5 0 0 1 8 2z" />
                        </svg>
                    </button>
                </template>

                <div class="pane_content">
                    <p class="muted">
                        {{ project_path ? project_path : t("explorer.pick_hint") }}
                    </p>
                    <button
                        type="button"
                        class="btn_accent"
                        :disabled="ingest_busy || !project_path"
                        @click="emit('run_ingest')"
                    >
                        {{ ingest_busy ? t("explorer.parsing") : t("explorer.start_parse") }}
                    </button>
                    <p class="muted">{{ t("explorer.prompt_hint") }}</p>
                    <p v-if="ingest_message" class="status_msg">{{ ingest_message }}</p>
                </div>
            </ViewPane>

            <ViewPane class="pane_history" :title="t('explorer.history')" :expanded="history_open" :badge="runs.length"
                @toggle="history_open = !history_open">
                <template #icon>
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="currentColor"
                            d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zM0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8zm8.5-3.5a.5.5 0 0 0-1 0V8a.5.5 0 0 0 .146.354l2 2a.5.5 0 0 0 .708-.708L8.5 7.793V4.5z" />
                    </svg>
                </template>

                <div class="pane_content">
                    <ul v-if="runs.length" class="run_list">
                        <li
                            v-for="run in runs"
                            :key="run.id"
                            class="run_row"
                            :class="{ active: active_run_id === run.id, editing: editing_run_id === run.id }"
                        >
                            <button
                                type="button"
                                class="run_item"
                                :disabled="editing_run_id === run.id"
                                @click="on_open(run)"
                            >
                                <div class="run_title_row">
                                    <template v-if="editing_run_id === run.id">
                                        <input
                                            :ref="(el) => bind_edit_input(el)"
                                            v-model="edit_draft"
                                            class="run_title_input"
                                            maxlength="120"
                                            :aria-label="t('explorer.rename_title')"
                                            @click.stop
                                            @keydown.enter.prevent="commit_rename(run)"
                                            @keydown.esc.prevent="cancel_rename"
                                            @blur="commit_rename(run)"
                                        />
                                    </template>
                                    <template v-else>
                                        <span class="run_title" :title="run_title(run)">{{ run_title(run) }}</span>
                                        <button
                                            type="button"
                                            class="run_icon run_edit"
                                            :title="t('explorer.rename_title')"
                                            :aria-label="t('explorer.rename_title')"
                                            @click="start_rename(run, $event)"
                                        >
                                            <svg viewBox="0 0 16 16" aria-hidden="true">
                                                <path fill="currentColor"
                                                    d="M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-9.5 9.5a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168l9.5-9.5zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207 11.207 2.5zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.293l6.5-6.5z" />
                                            </svg>
                                        </button>
                                    </template>
                                </div>
                                <div class="run_meta">
                                    <span class="run_id">#{{ run.id }}</span>
                                    <span class="run_status" :data-status="run.status">{{ run.status }}</span>
                                    <span class="run_time">{{ run.created_at }}</span>
                                </div>
                            </button>
                            <button
                                type="button"
                                class="run_icon run_delete"
                                :title="t('explorer.delete_run')"
                                :aria-label="t('explorer.delete_run')"
                                @click="on_delete(run, $event)"
                            >
                                <svg viewBox="0 0 16 16" aria-hidden="true">
                                    <path fill="currentColor"
                                        d="M6.5 1a.5.5 0 0 0-.5.5V2H2.5a.5.5 0 0 0 0 1h.563l.703 9.146A1.5 1.5 0 0 0 5.26 13.5h5.48a1.5 1.5 0 0 0 1.494-1.354L13 3h.5a.5.5 0 0 0 0-1H10v-.5a.5.5 0 0 0-.5-.5h-3zM7 2h2v.5H7V2zm1 3.5a.5.5 0 0 1 .5.5v6a.5.5 0 0 1-1 0V6a.5.5 0 0 1 .5-.5zm-2.5.5a.5.5 0 0 1 1 0v6a.5.5 0 0 1-1 0V6zm5 0a.5.5 0 0 1 1 0v6a.5.5 0 0 1-1 0V6z" />
                                </svg>
                            </button>
                        </li>
                    </ul>
                    <ul v-else class="empty_list">
                        <li>{{ t("explorer.history_empty") }}</li>
                    </ul>
                </div>
            </ViewPane>
        </div>

        <footer class="panel_dock">
            <div class="dock_spacer" />
            <span
                class="sidecar_badge"
                :data-online="sidecar_online ? '1' : '0'"
                :title="sidecar_label"
                :aria-label="sidecar_label"
            >
                <span class="sidecar_dot" aria-hidden="true" />
                <span class="sidecar_text">Sidecar</span>
            </span>
            <IconButton :label="t('explorer.settings')" @click="emit('open_settings')">
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

.run_row {
    position: relative;
    margin-bottom: 0.25rem;
}

.run_item {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.4rem 1.7rem 0.4rem 0.45rem;
    border: 1px solid transparent;
    border-radius: var(--radius_sm);
    background: transparent;
    color: var(--color_body);
    text-align: left;
    cursor: pointer;
}

.run_item:disabled {
    cursor: default;
}

.run_row:hover .run_item,
.run_row.editing .run_item {
    border-color: var(--color_line);
    background: var(--color_surface_2);
}

.run_row.active .run_item {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
    background: color-mix(in srgb, var(--color_accent) 10%, var(--color_surface_2));
}

.run_title_row {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    min-width: 0;
}

.run_title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--color_ink);
}

.run_title_input {
    flex: 1;
    min-width: 0;
    width: 100%;
    border: 1px solid var(--color_accent);
    border-radius: 3px;
    padding: 0.15rem 0.3rem;
    font-size: 0.78rem;
    color: var(--color_ink);
    background: var(--color_surface);
}

.run_meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.68rem;
    color: var(--color_faint);
}

.run_id {
    color: var(--color_muted);
}

.run_status {
    color: var(--color_muted);
}

.run_status[data-status="ok"] {
    color: var(--color_accent_deep);
}

.run_status[data-status="failed"] {
    color: var(--color_brand);
}

.run_time {
    color: var(--color_faint);
}

.run_icon {
    display: inline-flex;
    width: 20px;
    height: 20px;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: 3px;
    background: transparent;
    color: var(--color_muted);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s ease, color 0.12s ease, background 0.12s ease;
}

.run_row:hover .run_icon,
.run_row.editing .run_icon,
.run_icon:focus-visible {
    opacity: 1;
    pointer-events: auto;
}

.run_icon:hover {
    color: var(--color_ink);
    background: color-mix(in srgb, var(--color_accent) 14%, transparent);
}

.run_icon svg {
    width: 13px;
    height: 13px;
}

.run_delete {
    position: absolute;
    top: 50%;
    right: 0.28rem;
    transform: translateY(-50%);
    z-index: 1;
}

.run_delete:hover {
    color: var(--color_brand);
    background: color-mix(in srgb, var(--color_brand) 14%, transparent);
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
    gap: 0.35rem;
    padding: 0.35rem 0.45rem;
    border-top: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface) 88%, var(--color_surface_2));
}

.dock_spacer {
    flex: 1;
}

.sidecar_badge {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    max-width: 7.5rem;
    padding: 0.18rem 0.45rem 0.18rem 0.35rem;
    border: 1px solid var(--color_line);
    border-radius: 999px;
    background: var(--color_surface_2);
    color: var(--color_muted);
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    user-select: none;
}

.sidecar_dot {
    width: 7px;
    height: 7px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: #c44b4b;
    box-shadow: 0 0 0 2px color-mix(in srgb, #c44b4b 22%, transparent);
}

.sidecar_badge[data-online="1"] .sidecar_dot {
    background: #2f9e6b;
    box-shadow: 0 0 0 2px color-mix(in srgb, #2f9e6b 22%, transparent);
}

.sidecar_badge[data-online="1"] {
    color: var(--color_ink);
}

.sidecar_text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
