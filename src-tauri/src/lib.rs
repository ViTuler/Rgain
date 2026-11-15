mod db;
mod dev_port;
mod graph;
mod ingest;
mod llm;
mod logutil;
mod sidecar;

use db::{DbState, ProjectRow, PublicSettings, RunRow, TermExplanationRow};
use ingest::IngestResult;
use serde::Deserialize;
use serde_json::Value;
use sidecar::SidecarState;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, RunEvent, State};

#[derive(Debug, serde::Serialize)]
struct SidecarResult {
    ok: bool,
    detail: String,
}

#[derive(Debug, Deserialize)]
struct OpenProjectArgs {
    root_path: String,
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RunIngestArgs {
    project_id: i64,
}

#[tauri::command]
fn get_settings(db: State<'_, DbState>) -> Result<PublicSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(db::get_settings(&conn)?.to_public())
}

#[tauri::command]
fn save_settings(
    db: State<'_, DbState>,
    mut settings: PublicSettings,
) -> Result<PublicSettings, String> {
    settings.dev_port = dev_port::clamp_dev_port(settings.dev_port);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::save_public_settings(&conn, &settings)?;
    Ok(db::get_settings(&conn)?.to_public())
}

#[derive(Debug, Deserialize)]
struct TestLlmArgs {
    api_base_url: Option<String>,
    model: Option<String>,
    /// Optional draft key; empty / masked uses stored key.
    api_key: Option<String>,
}

#[tauri::command]
async fn test_llm(app: AppHandle, args: TestLlmArgs) -> Result<SidecarResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<DbState>();
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut settings = db::get_settings(&conn)?;
        drop(conn);

        if let Some(url) = args.api_base_url {
            if !url.trim().is_empty() {
                settings.api_base_url = url;
            }
        }
        if let Some(model) = args.model {
            if !model.trim().is_empty() {
                settings.model = model;
            }
        }
        if let Some(key) = args.api_key {
            if !db::is_keep_api_key(&key, &settings.api_key) {
                settings.api_key = key.trim().to_string();
            }
        }
        match llm::test_connection(&settings) {
            Ok(detail) => Ok(SidecarResult { ok: true, detail }),
            Err(detail) => Ok(SidecarResult { ok: false, detail }),
        }
    })
    .await
    .map_err(|e| format!("test_llm_join:{e}"))?
}

#[tauri::command]
fn default_agent_prompt() -> String {
    ingest::load_agent_prompt()
}

#[tauri::command]
fn count_runs(db: State<'_, DbState>) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::count_runs(&conn)
}

#[tauri::command]
fn list_runs(db: State<'_, DbState>, limit: Option<i64>) -> Result<Vec<RunRow>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::list_runs(&conn, limit.unwrap_or(50))
}

#[tauri::command]
fn get_run_graph(db: State<'_, DbState>, run_id: i64) -> Result<Option<Value>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match db::get_graph_for_run(&conn, run_id)? {
        Some(raw) => {
            let value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
struct GraphLayoutArgs {
    run_id: i64,
    view_key: Option<String>,
    layout_json: Option<String>,
}

fn resolve_view_key(view_key: &Option<String>) -> String {
    view_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(db::DEFAULT_LAYOUT_VIEW_KEY)
        .to_string()
}

#[tauri::command]
fn get_graph_layout(db: State<'_, DbState>, args: GraphLayoutArgs) -> Result<Option<Value>, String> {
    let view_key = resolve_view_key(&args.view_key);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match db::get_graph_layout(&conn, args.run_id, &view_key)? {
        Some(raw) => {
            let value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn save_graph_layout(db: State<'_, DbState>, args: GraphLayoutArgs) -> Result<(), String> {
    let view_key = resolve_view_key(&args.view_key);
    let layout_json = args
        .layout_json
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "layout_json_required".to_string())?;
    // Validate JSON before persist.
    let _: Value = serde_json::from_str(layout_json).map_err(|e| format!("layout_json:{e}"))?;
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::save_graph_layout(&conn, args.run_id, &view_key, layout_json)
}

#[tauri::command]
fn clear_graph_layout(db: State<'_, DbState>, args: GraphLayoutArgs) -> Result<(), String> {
    let view_key = resolve_view_key(&args.view_key);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::clear_graph_layout(&conn, args.run_id, &view_key)
}

#[derive(Debug, Deserialize)]
struct ExplainTermArgs {
    term: String,
    context: Option<String>,
    run_id: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
struct ExplainTermResult {
    id: i64,
    term: String,
    context: String,
    explanation: String,
}

#[tauri::command]
async fn explain_term(app: AppHandle, args: ExplainTermArgs) -> Result<ExplainTermResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<DbState>();
        let settings = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            db::get_settings(&conn)?
        };
        let context = args.context.unwrap_or_default();
        let explanation = llm::explain_term(&settings, &args.term, &context)?;
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let id = db::save_term_explanation(
            &conn,
            args.run_id,
            args.term.trim(),
            context.trim(),
            explanation.trim(),
        )?;
        Ok(ExplainTermResult {
            id,
            term: args.term.trim().to_string(),
            context: context.trim().to_string(),
            explanation: explanation.trim().to_string(),
        })
    })
    .await
    .map_err(|e| format!("explain_term_join:{e}"))?
}

#[tauri::command]
fn list_term_explanations(
    db: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<TermExplanationRow>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::list_term_explanations(&conn, limit.unwrap_or(30))
}

#[tauri::command]
fn sidecar_health(sidecar: State<'_, SidecarState>) -> Result<SidecarResult, String> {
    match sidecar::request(&sidecar, "health", None) {
        Ok(value) => {
            let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(SidecarResult {
                ok,
                detail: format!("{} · {}", sidecar::launch_description(), value),
            })
        }
        Err(err) => Ok(SidecarResult {
            ok: false,
            detail: err,
        }),
    }
}

#[tauri::command]
fn sidecar_echo(
    sidecar: State<'_, SidecarState>,
    payload: Value,
) -> Result<SidecarResult, String> {
    match sidecar::request(&sidecar, "echo", Some(payload)) {
        Ok(value) => {
            let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            Ok(SidecarResult {
                ok,
                detail: value.to_string(),
            })
        }
        Err(err) => Ok(SidecarResult {
            ok: false,
            detail: err,
        }),
    }
}

#[tauri::command]
fn open_project(db: State<'_, DbState>, args: OpenProjectArgs) -> Result<ProjectRow, String> {
    let root = PathBuf::from(&args.root_path);
    if !ingest::is_supported_project_root(&root) {
        return Err("root_path_invalid_need_folder_or_book".into());
    }
    let kind = args.kind.unwrap_or_else(|| {
        if root.is_file() {
            "book".into()
        } else {
            "code".into()
        }
    });
    let title = root
        .file_stem()
        .or_else(|| root.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = db::upsert_project(&conn, &root.to_string_lossy(), &kind, &title)?;
    db::get_project(&conn, id)
}

fn run_ingest_sync(
    db: &DbState,
    sidecar: &SidecarState,
    args: RunIngestArgs,
) -> Result<IngestResult, String> {
    logutil::info(&format!("ingest_begin project_id={}", args.project_id));
    let (settings, project, run_id, root) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let settings = db::get_settings(&conn)?;
        let project = db::get_project(&conn, args.project_id)?;
        let root = PathBuf::from(&project.root_path);
        if !ingest::is_supported_project_root(&root) {
            logutil::error(&format!(
                "ingest_bad_root project_id={} path={}",
                args.project_id,
                root.display()
            ));
            return Err("project_root_missing".into());
        }
        let run_id = db::create_run(&conn, project.id, "global_agent_prompt", &settings.model)?;
        logutil::info(&format!(
            "ingest_run_created run_id={run_id} project_id={} kind={} root={}",
            project.id,
            project.kind,
            root.display()
        ));
        (settings, project, run_id, root)
    };

    let extract = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        ingest::extract_project_files(&conn, sidecar, project.id, &root)
    };

    let (files, mut warnings) = match extract {
        Ok(v) => v,
        Err(err) => {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = db::finish_run(&conn, run_id, "failed");
            return Ok(IngestResult {
                project_id: project.id,
                run_id,
                status: "failed".into(),
                files_processed: 0,
                files_cached: 0,
                graph: None,
                error: Some(err),
                warnings: vec![],
            });
        }
    };

    let cached = files.iter().filter(|f| f.cached).count();
    let corpus = ingest::build_corpus(&files);
    if corpus.trim().is_empty() {
        let detail = if warnings.is_empty() {
            "no_extractable_content".to_string()
        } else {
            format!("no_extractable_content · {}", warnings.join("; "))
        };
        logutil::error(&format!(
            "ingest_empty run_id={run_id} project_id={} files={} cached={} detail={detail}",
            project.id,
            files.len(),
            cached
        ));
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = db::set_run_title(&conn, run_id, &format!("Empty #{run_id}"));
        let _ = db::finish_run(&conn, run_id, "failed");
        return Ok(IngestResult {
            project_id: project.id,
            run_id,
            status: "failed".into(),
            files_processed: files.len(),
            files_cached: cached,
            graph: None,
            error: Some(detail),
            warnings,
        });
    }

    // LLM stays off the DB lock so other commands stay responsive.
    match llm::generate_graph(&settings, &corpus) {
        Ok(graph) => {
            let payload = serde_json::to_string(&graph).map_err(|e| e.to_string())?;
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let title = graph
                .get("meta")
                .and_then(|m| m.get("title"))
                .and_then(|t| t.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Untitled")
                .to_string();
            db::save_graph_snapshot(&conn, run_id, &payload)?;
            db::set_run_title(&conn, run_id, &title)?;
            db::finish_run(&conn, run_id, "ok")?;
            Ok(IngestResult {
                project_id: project.id,
                run_id,
                status: "ok".into(),
                files_processed: files.len(),
                files_cached: cached,
                graph: Some(graph),
                error: None,
                warnings,
            })
        }
        Err(err) => {
            warnings.push(err.clone());
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = db::set_run_title(&conn, run_id, &format!("Failed #{run_id}"));
            db::finish_run(&conn, run_id, "failed")?;
            Ok(IngestResult {
                project_id: project.id,
                run_id,
                status: "failed".into(),
                files_processed: files.len(),
                files_cached: cached,
                graph: None,
                error: Some(err),
                warnings,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
struct RunTitleArgs {
    run_id: i64,
    title: String,
}

#[derive(Debug, Deserialize)]
struct RunIdArgs {
    run_id: i64,
}

#[tauri::command]
fn update_run_title(db: State<'_, DbState>, args: RunTitleArgs) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::set_run_title(&conn, args.run_id, &args.title)
}

#[tauri::command]
fn delete_run(db: State<'_, DbState>, args: RunIdArgs) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::delete_run(&conn, args.run_id)
}

#[tauri::command]
async fn run_ingest(app: AppHandle, args: RunIngestArgs) -> Result<IngestResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<DbState>();
        let sidecar = app.state::<SidecarState>();
        run_ingest_sync(&db, &sidecar, args)
    })
    .await
    .map_err(|e| format!("ingest_join:{e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("app_data_dir: {e}"))?;
            let _ = logutil::init(&app_data);
            logutil::info(&format!("app_start data_dir={}", app_data.display()));
            let conn = db::open_db(&db::db_path(&app_data))?;
            if let Ok(port) = dev_port::read_dev_port_file() {
                let _ = db::upsert_setting(&conn, "dev_port", &port.to_string());
            }
            app.manage(DbState {
                conn: Mutex::new(conn),
            });
            app.manage(SidecarState {
                child: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            test_llm,
            default_agent_prompt,
            count_runs,
            list_runs,
            update_run_title,
            delete_run,
            get_run_graph,
            get_graph_layout,
            save_graph_layout,
            clear_graph_layout,
            explain_term,
            list_term_explanations,
            sidecar_health,
            sidecar_echo,
            open_project,
            run_ingest
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle: &AppHandle, event: RunEvent| {
            if let RunEvent::Exit = event {
                if let Some(state) = app_handle.try_state::<SidecarState>() {
                    sidecar::shutdown(&state);
                }
            }
        });
}
