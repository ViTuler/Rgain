mod db;
mod sidecar;

use db::{AppSettings, DbState};
use serde::Serialize;
use serde_json::Value;
use sidecar::SidecarState;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, RunEvent, State};

#[derive(Debug, Serialize)]
struct SidecarResult {
    ok: bool,
    detail: String,
}

#[tauri::command]
fn get_settings(db: State<'_, DbState>) -> Result<AppSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::get_settings(&conn)
}

#[tauri::command]
fn save_settings(
    db: State<'_, DbState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::save_settings(&conn, &settings)?;
    db::get_settings(&conn)
}

#[tauri::command]
fn count_runs(db: State<'_, DbState>) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::count_runs(&conn)
}

fn python_path_from_db(db: &DbState) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(db::get_settings(&conn)?.python_path)
}

#[tauri::command]
fn sidecar_health(
    db: State<'_, DbState>,
    sidecar: State<'_, SidecarState>,
) -> Result<SidecarResult, String> {
    let python_path = python_path_from_db(&db)?;
    match sidecar::request(&sidecar, &python_path, "health", None) {
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
fn sidecar_echo(
    db: State<'_, DbState>,
    sidecar: State<'_, SidecarState>,
    payload: Value,
) -> Result<SidecarResult, String> {
    let python_path = python_path_from_db(&db)?;
    match sidecar::request(&sidecar, &python_path, "echo", Some(payload)) {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("app_data_dir: {e}"))?;
            let conn = db::open_db(&db::db_path(&app_data))?;
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
            count_runs,
            sidecar_health,
            sidecar_echo
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
