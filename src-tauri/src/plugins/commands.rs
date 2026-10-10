use base64::Engine;
use chrono::NaiveDate;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use super::manager::PluginManager;
use super::manifest::PluginManifest;
use super::statement::{ParseStatementInput, ParsedStatementOutput};
use crate::app_state::AppState;
use crate::shared::AppError;

const MAX_STATEMENT_BUFFER_BYTES: usize = 5 * 1024 * 1024; // 5 MB

fn get_plugin_search_directories(app: &AppHandle, state: &AppState) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // 1. App config directory (~/.config/finnca/plugins)
    if let Ok(base) = app.path().config_dir() {
        let app_plugins = base.join("finnca").join("plugins");
        if app_plugins.exists() {
            dirs.push(app_plugins);
        }
    }

    // 2. Active vault directory if unlocked (<vault_path>/plugins)
    if let Ok(session_guard) = state.session.lock() {
        if let Some(session) = session_guard.as_ref() {
            let vault_plugins = session.vault_path.join("plugins");
            if vault_plugins.exists() {
                dirs.push(vault_plugins);
            }
        }
    }

    dirs
}

#[tauri::command]
#[specta::specta]
pub fn list_plugins_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<PluginManifest>, AppError> {
    let search_dirs = get_plugin_search_directories(&app, &state);
    let mut manifests = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for dir in search_dirs {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_file = path.join("plugin.toml");
                if manifest_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_file) {
                        if let Ok(manifest) = PluginManifest::parse_from_toml(&content) {
                            if !seen_ids.contains(&manifest.plugin.id) {
                                seen_ids.insert(manifest.plugin.id.clone());
                                manifests.push(manifest);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(manifests)
}

#[tauri::command]
#[specta::specta]
pub fn execute_statement_parser_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    plugin_id: String,
    input: ParseStatementInput,
) -> Result<ParsedStatementOutput, AppError> {
    // 1. Validate payload byte size under 5 MB limit
    if !input.file_bytes_base64.trim().is_empty() {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(input.file_bytes_base64.trim())
            .map_err(|e| AppError::InvalidInput(format!("Invalid base64 statement buffer: {e}")))?;
        if decoded.len() > MAX_STATEMENT_BUFFER_BYTES {
            return Err(AppError::InvalidInput(format!(
                "Statement buffer ({} bytes) exceeds the 5 MB limit",
                decoded.len()
            )));
        }
    }

    // 2. Locate plugin across search directories
    let search_dirs = get_plugin_search_directories(&app, &state);
    let mut target_manager = None;

    for dir in search_dirs {
        let candidate = PluginManager::new(dir);
        if candidate.resolve_plugin_dir(&plugin_id).is_ok() {
            target_manager = Some(candidate);
            break;
        }
    }

    let manager = target_manager.ok_or_else(|| {
        AppError::NotFound(format!(
            "Plugin with ID '{plugin_id}' not found in plugins directories"
        ))
    })?;

    // 3. Load sandboxed plugin (verifies SHA-256 and configures memory caps)
    let (mut plugin, manifest) = manager.load_plugin(&plugin_id)?;

    // Verify plugin actually declares a statement parser
    let has_parser = manifest
        .extensions
        .statement_parsers
        .iter()
        .any(|p| p.id == input.parser_id || input.parser_id.is_empty());
    if !has_parser && !manifest.extensions.statement_parsers.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "Plugin '{}' does not provide statement parser '{}'",
            plugin_id, input.parser_id
        )));
    }

    // 4. Invoke Wasm export `parse_statement`
    let output: ParsedStatementOutput =
        manager.invoke_function(&mut plugin, "parse_statement", &input)?;

    // 5. Host-side verification: Date formats must be ISO-8601 (YYYY-MM-DD)
    for (idx, row) in output.rows.iter().enumerate() {
        if NaiveDate::parse_from_str(row.date.trim(), "%Y-%m-%d").is_err() {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_INVALID_DATE: Row [{idx}] has invalid ISO-8601 date '{}'. Expected 'YYYY-MM-DD'.",
                row.date
            )));
        }
    }

    if let Some(ref start) = output.start_date {
        if NaiveDate::parse_from_str(start.trim(), "%Y-%m-%d").is_err() {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_INVALID_DATE: Statement start_date has invalid ISO-8601 date '{start}'. Expected 'YYYY-MM-DD'."
            )));
        }
    }

    if let Some(ref end) = output.end_date {
        if NaiveDate::parse_from_str(end.trim(), "%Y-%m-%d").is_err() {
            return Err(AppError::InvalidInput(format!(
                "ERR_PLUGIN_INVALID_DATE: Statement end_date has invalid ISO-8601 date '{end}'. Expected 'YYYY-MM-DD'."
            )));
        }
    }

    Ok(output)
}
