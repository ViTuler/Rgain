<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import LeftPanel from "./components/panels/LeftPanel.vue";
import LogicStage from "./components/scene/LogicStage.vue";
import ExplainPanel from "./components/panels/ExplainPanel.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";
import TermExplainDialog from "./components/dialogs/TermExplainDialog.vue";
import ThemeSwitch from "./components/chrome/ThemeSwitch.vue";
import IconButton from "./components/chrome/IconButton.vue";
import WindowControls from "./components/chrome/WindowControls.vue";
import SelectionExplainFab from "./components/chrome/SelectionExplainFab.vue";
import { use_explain_hotkey } from "./composables/use_explain_hotkey";
import { use_settings } from "./composables/use_settings";
import { use_theme } from "./composables/use_theme";
import { set_locale, use_i18n } from "./i18n";
import type { AppSettings } from "./types/settings";
import type { GraphNode, GraphPayload } from "./types/graph";
import type { IngestResult, ProjectRow, RunRow } from "./types/project";
import type { ExplainTermResult } from "./types/term";

const { settings, load_settings, save_settings } = use_settings();
const { theme, toggle_theme, init_theme } = use_theme();
const { t } = use_i18n();

const show_settings = ref(false);
const show_explain = ref(false);
const show_term_dialog = ref(false);
const term_busy = ref(false);
const term_error = ref("");
const term_draft = ref({ term: "", explanation: "", context: "" });
const selection_term = ref("");
const selection_fab_pos = ref({ left: 0, top: 0 });
const busy = ref(false);
const settings_message = ref("");
const sidecar_status = ref("未检测");
const sidecar_message = ref("");
const llm_message = ref("");
const draft_settings = ref<AppSettings | null>(null);

const project = ref<ProjectRow | null>(null);
const runs = ref<RunRow[]>([]);
const active_run_id = ref<number | null>(null);
const ingest_busy = ref(false);
const ingest_message = ref("");
const current_graph = ref<GraphPayload | null>(null);
const selected_label = ref("");
const selected_node = ref<GraphNode | null>(null);

const graph_title = computed(() => current_graph.value?.meta.title || "");
const explain_hotkey = computed(() => settings.value.explain_hotkey || "Ctrl+Shift+E");
const selection_explain_enabled = computed(
    () => settings.value.selection_explain_enabled !== false,
);
const show_selection_fab = computed(
    () =>
        selection_explain_enabled.value &&
        !show_settings.value &&
        !show_term_dialog.value &&
        selection_term.value.length > 0,
);

const editing_settings = computed({
    get: () => draft_settings.value ?? settings.value,
    set: (value: AppSettings) => {
        draft_settings.value = value;
    },
});

const project_label = computed(() => project.value?.title || t("messages.project_none"));
const project_path = computed(() => project.value?.root_path || "");

use_explain_hotkey(explain_hotkey, () => {
    void trigger_term_explain();
});

function read_selection_text(): string {
    return window.getSelection()?.toString().trim().slice(0, 200) || "";
}

function read_selection_anchor(): { left: number; top: number } | null {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return null;
    const rect = sel.getRangeAt(0).getBoundingClientRect();
    if (rect.width <= 0 && rect.height <= 0) return null;
    return { left: rect.right, top: rect.top };
}

function refresh_selection_term() {
    if (!selection_explain_enabled.value || show_settings.value || show_term_dialog.value) {
        selection_term.value = "";
        return;
    }
    const text = read_selection_text();
    const anchor = read_selection_anchor();
    if (text && anchor) {
        selection_term.value = text;
        selection_fab_pos.value = anchor;
    } else {
        selection_term.value = "";
    }
}

function on_selection_mouse_up() {
    // Defer so the browser finishes the selection range.
    window.setTimeout(refresh_selection_term, 0);
}

function on_selection_scroll_or_resize() {
    if (!selection_term.value) return;
    const anchor = read_selection_anchor();
    if (anchor) selection_fab_pos.value = anchor;
    else selection_term.value = "";
}

onMounted(async () => {
    init_theme();
    await load_settings();
    draft_settings.value = { ...settings.value };
    await refresh_runs();
    await refresh_sidecar_status();
    document.addEventListener("mouseup", on_selection_mouse_up);
    document.addEventListener("keyup", refresh_selection_term);
    window.addEventListener("scroll", on_selection_scroll_or_resize, true);
    window.addEventListener("resize", on_selection_scroll_or_resize);
});

onBeforeUnmount(() => {
    document.removeEventListener("mouseup", on_selection_mouse_up);
    document.removeEventListener("keyup", refresh_selection_term);
    window.removeEventListener("scroll", on_selection_scroll_or_resize, true);
    window.removeEventListener("resize", on_selection_scroll_or_resize);
});

async function refresh_runs() {
    try {
        runs.value = await invoke<RunRow[]>("list_runs", { limit: 40 });
    } catch {
        runs.value = [];
    }
}

async function refresh_sidecar_status() {
    try {
        const result = await invoke<{ ok: boolean; detail: string }>("sidecar_health");
        sidecar_status.value = result.ok ? "online" : `offline · ${result.detail}`;
    } catch (err) {
        sidecar_status.value = `offline · ${String(err)}`;
    }
}

function open_settings() {
    draft_settings.value = { ...settings.value };
    settings_message.value = "";
    sidecar_message.value = "";
    llm_message.value = "";
    show_settings.value = true;
}

function close_settings() {
    draft_settings.value = { ...settings.value };
    set_locale(settings.value.ui_locale);
    show_settings.value = false;
}

function toggle_explain() {
    show_explain.value = !show_explain.value;
}

async function on_save_settings() {
    if (!draft_settings.value) return;
    busy.value = true;
    settings_message.value = "";
    try {
        await save_settings(draft_settings.value);
        draft_settings.value = { ...settings.value };
        settings_message.value = t("settings.done");
        await refresh_sidecar_status();
    } catch (err) {
        settings_message.value = t("settings.save_failed", { error: String(err) });
    } finally {
        busy.value = false;
    }
}

async function on_test_sidecar() {
    busy.value = true;
    sidecar_message.value = "";
    try {
        const result = await invoke<{ ok: boolean; detail: string }>("sidecar_echo", {
            payload: { ping: "rgain", ts: Date.now() },
        });
        sidecar_message.value = result.detail;
        sidecar_status.value = result.ok ? "online" : `offline · ${result.detail}`;
    } catch (err) {
        sidecar_message.value = String(err);
        sidecar_status.value = `offline · ${String(err)}`;
    } finally {
        busy.value = false;
    }
}

async function on_test_llm() {
    if (!draft_settings.value) return;
    busy.value = true;
    llm_message.value = "";
    try {
        const result = await invoke<{ ok: boolean; detail: string }>("test_llm", {
            args: {
                api_base_url: draft_settings.value.api_base_url,
                model: draft_settings.value.model,
                api_key: draft_settings.value.api_key,
            },
        });
        llm_message.value = result.ok
            ? result.detail
            : t("settings.llm_failed", { detail: result.detail });
    } catch (err) {
        llm_message.value = String(err);
    } finally {
        busy.value = false;
    }
}

async function on_pick_folder() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: t("messages.pick_folder_title"),
        });
        if (!selected || Array.isArray(selected)) return;
        const row = await invoke<ProjectRow>("open_project", {
            args: { root_path: selected, kind: "code" },
        });
        project.value = row;
        ingest_message.value = t("messages.opened", { path: row.root_path });
        current_graph.value = null;
        selected_label.value = "";
        selected_node.value = null;
        active_run_id.value = null;
    } catch (err) {
        ingest_message.value = t("messages.open_failed", { error: String(err) });
    }
}

function on_select_node(node: GraphNode) {
    selected_node.value = node;
    selected_label.value = node.label;
    show_explain.value = true;
}

function on_reset_prompt() {
    if (!draft_settings.value) return;
    draft_settings.value = { ...draft_settings.value, agent_prompt: "" };
    settings_message.value = t("settings.prompt_cleared");
}

async function on_run_ingest() {
    if (!project.value) {
        ingest_message.value = t("messages.need_folder");
        return;
    }
    ingest_busy.value = true;
    ingest_message.value = t("messages.ingesting");
    try {
        const result = await invoke<IngestResult>("run_ingest", {
            args: {
                project_id: project.value.id,
            },
        });
        await refresh_runs();
        active_run_id.value = result.run_id;
        if (result.status === "ok" && result.graph) {
            current_graph.value = result.graph as GraphPayload;
            selected_label.value = result.graph.meta?.title || "";
            selected_node.value = null;
            show_explain.value = true;
            const warn = result.warnings.length ? ` · warnings ${result.warnings.length}` : "";
            ingest_message.value = t("messages.ingest_ok", {
                id: result.run_id,
                files: result.files_processed,
                cached: result.files_cached,
                warn,
            });
        } else {
            current_graph.value = null;
            selected_node.value = null;
            const detail = result.error || result.warnings.join("; ") || "unknown";
            ingest_message.value = t("messages.ingest_fail", {
                id: result.run_id,
                detail,
            });
        }
    } catch (err) {
        ingest_message.value = t("messages.ingest_error", { error: String(err) });
    } finally {
        ingest_busy.value = false;
    }
}

async function on_open_run(run_id: number) {
    try {
        const graph = await invoke<GraphPayload | null>("get_run_graph", {
            runId: run_id,
        });
        if (!graph) {
            ingest_message.value = t("messages.no_graph", { id: run_id });
            return;
        }
        current_graph.value = graph;
        active_run_id.value = run_id;
        selected_label.value = graph.meta?.title || `run #${run_id}`;
        selected_node.value = null;
        show_explain.value = true;
        ingest_message.value = t("messages.loaded_run", { id: run_id });
    } catch (err) {
        ingest_message.value = t("messages.load_failed", { error: String(err) });
    }
}

async function on_rename_run(payload: { run_id: number; title: string }) {
    try {
        await invoke("update_run_title", {
            args: {
                run_id: payload.run_id,
                title: payload.title,
            },
        });
        await refresh_runs();
        if (active_run_id.value === payload.run_id) {
            selected_label.value = payload.title;
            if (current_graph.value?.meta) {
                current_graph.value = {
                    ...current_graph.value,
                    meta: {
                        ...current_graph.value.meta,
                        title: payload.title,
                    },
                };
            }
        }
        ingest_message.value = t("messages.renamed", { id: payload.run_id });
    } catch (err) {
        ingest_message.value = t("messages.rename_failed", { error: String(err) });
    }
}

async function on_delete_run(run_id: number) {
    const run = runs.value.find((r) => r.id === run_id);
    const label = run?.title?.trim() || `Run #${run_id}`;
    const ok = window.confirm(t("explorer.delete_confirm", { title: label }));
    if (!ok) return;
    try {
        await invoke("delete_run", {
            args: { run_id },
        });
        if (active_run_id.value === run_id) {
            active_run_id.value = null;
            current_graph.value = null;
            selected_node.value = null;
            selected_label.value = "";
        }
        await refresh_runs();
        ingest_message.value = t("messages.deleted", { id: run_id });
    } catch (err) {
        ingest_message.value = t("messages.delete_failed", { error: String(err) });
    }
}

function build_term_context(): string {
    if (selected_node.value) {
        return `node=${selected_node.value.label}\n${selected_node.value.explain}`.slice(0, 1200);
    }
    if (graph_title.value) {
        return `graph=${graph_title.value}`.slice(0, 1200);
    }
    return "";
}

function collect_term_payload(
    preferred_term?: string,
): { term: string; context: string } | null {
    const selection = (preferred_term || read_selection_text() || selection_term.value).trim();
    if (selection) {
        return { term: selection.slice(0, 200), context: build_term_context() };
    }
    if (selected_node.value) {
        return {
            term: selected_node.value.label,
            context: selected_node.value.explain.slice(0, 1200),
        };
    }
    return null;
}

async function trigger_term_explain(preferred_term?: string) {
    if (term_busy.value) return;
    let payload = collect_term_payload(preferred_term);
    if (!payload) {
        try {
            const clip = (await navigator.clipboard.readText()).trim();
            if (clip) {
                payload = {
                    term: clip.slice(0, 200),
                    context: selected_node.value?.explain.slice(0, 1200) || "",
                };
            }
        } catch {
            // clipboard may be denied
        }
    }
    if (!payload) {
        term_draft.value = { term: "", explanation: "", context: "" };
        term_error.value = t("term.need_selection");
        show_term_dialog.value = true;
        return;
    }

    selection_term.value = "";
    term_busy.value = true;
    term_error.value = "";
    term_draft.value = {
        term: payload.term,
        explanation: "",
        context: payload.context,
    };
    show_term_dialog.value = true;

    try {
        const result = await invoke<ExplainTermResult>("explain_term", {
            args: {
                term: payload.term,
                context: payload.context || null,
                run_id: active_run_id.value,
            },
        });
        term_draft.value = {
            term: result.term,
            explanation: result.explanation,
            context: result.context,
        };
    } catch (err) {
        term_error.value = String(err);
    } finally {
        term_busy.value = false;
    }
}

function on_selection_fab_explain() {
    void trigger_term_explain(selection_term.value);
}
</script>

<template>
    <div class="app_shell">
        <header class="top_bar">
            <div class="brand_block" data-tauri-drag-region>
                <span class="brand">Rgain</span>
                <span class="subtitle">{{ t("app.brand_sub") }}</span>
            </div>
            <div class="drag_fill" data-tauri-drag-region />
            <div class="top_actions">
                <ThemeSwitch :theme="theme" @toggle="toggle_theme" />
                <IconButton
                    class="explain_toggle"
                    :label="t('app.toggle_explain')"
                    :pressed="show_explain"
                    @click="toggle_explain"
                >
                    <svg viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="currentColor"
                            d="M2 1.5A1.5 1.5 0 0 0 .5 3v10A1.5 1.5 0 0 0 2 14.5h12a1.5 1.5 0 0 0 1.5-1.5V3A1.5 1.5 0 0 0 14 1.5H2zm0 1h8.5v11H2a.5.5 0 0 1-.5-.5V3a.5.5 0 0 1 .5-.5zm9.5 0H14a.5.5 0 0 1 .5.5v10a.5.5 0 0 1-.5.5h-2.5v-11z" />
                    </svg>
                </IconButton>
                <WindowControls />
            </div>
        </header>

        <SelectionExplainFab
            :visible="show_selection_fab"
            :left="selection_fab_pos.left"
            :top="selection_fab_pos.top"
            @explain="on_selection_fab_explain"
        />

        <div class="shell_body">
            <div class="workspace">
                <LeftPanel
                    :project_label="project_label"
                    :project_path="project_path"
                    :runs="runs"
                    :active_run_id="active_run_id"
                    :ingest_busy="ingest_busy"
                    :ingest_message="ingest_message"
                    :sidecar_status="sidecar_status"
                    @open_settings="open_settings"
                    @pick_folder="on_pick_folder"
                    @run_ingest="on_run_ingest"
                    @open_run="on_open_run"
                    @rename_run="on_rename_run"
                    @delete_run="on_delete_run"
                />
                <LogicStage
                    :graph="current_graph"
                    :run_id="active_run_id"
                    @select_node="on_select_node"
                />
            </div>

            <ExplainPanel
                :open="show_explain"
                :selected_label="selected_label"
                :selected_node="selected_node"
                :graph_title="graph_title"
            />
        </div>

        <SettingsPanel
            v-if="show_settings"
            v-model:model_value="editing_settings"
            :busy="busy"
            :message="settings_message"
            :sidecar_message="sidecar_message"
            :llm_message="llm_message"
            @save="on_save_settings"
            @test_sidecar="on_test_sidecar"
            @test_llm="on_test_llm"
            @reset_prompt="on_reset_prompt"
            @close="close_settings"
        />

        <TermExplainDialog
            v-if="show_term_dialog"
            :term="term_draft.term"
            :explanation="term_draft.explanation"
            :context="term_draft.context"
            :busy="term_busy"
            :error="term_error"
            :hotkey_hint="explain_hotkey"
            @close="show_term_dialog = false"
        />
    </div>
</template>

<style scoped>
.app_shell {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color_bg);
    color: var(--color_ink);
}

.top_bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    height: var(--titlebar_height);
    padding: 0 0.35rem 0 0.85rem;
    border-bottom: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface) 92%, var(--color_surface_2));
}

.brand_block {
    display: flex;
    flex-direction: row;
    align-items: baseline;
    gap: 0.5rem;
    min-width: 0;
    padding: 0.15rem 0;
}

.brand {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    color: var(--color_ink);
}

.subtitle {
    font-size: 0.68rem;
    color: var(--color_muted);
}

.drag_fill {
    flex: 1;
    align-self: stretch;
    min-width: 1rem;
}

.top_actions {
    display: flex;
    align-items: center;
    gap: 0.2rem;
}

/* Explain-sidebar toggle: dark + pressed — light icon for titlebar contrast. */
:global(html[data-theme="dark"]) .top_actions :deep(.explain_toggle.pressed) {
    color: #e4e4e4;
}

:global(html[data-theme="dark"]) .top_actions :deep(.explain_toggle.pressed:hover) {
    color: #e4e4e4;
}

.shell_body {
    position: relative;
    display: flex;
    flex: 1;
    min-height: 0;
}

.workspace {
    display: flex;
    flex: 1;
    min-width: 0;
}
</style>
