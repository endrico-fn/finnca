use extism::{Manifest, Plugin, PluginBuilder, Wasm};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::gateway::validate_no_raw_floats;
use super::manifest::PluginManifest;
use crate::shared::AppError;

pub struct PluginManager {
    plugins_directory: PathBuf,
}

impl PluginManager {
    pub fn new(plugins_directory: PathBuf) -> Self {
        Self { plugins_directory }
    }

    pub fn plugins_directory(&self) -> &Path {
        &self.plugins_directory
    }

    /// Resolves the plugin directory by checking direct folder name or scanning manifest IDs.
    pub fn resolve_plugin_dir(&self, plugin_id: &str) -> Result<PathBuf, AppError> {
        let direct_dir = self.plugins_directory.join(plugin_id);
        if direct_dir.join("plugin.toml").exists() {
            return Ok(direct_dir);
        }

        // Search subdirectories to find manifest where manifest.plugin.id == plugin_id
        if let Ok(entries) = std::fs::read_dir(&self.plugins_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_file = path.join("plugin.toml");
                    if manifest_file.exists() {
                        if let Ok(raw) = std::fs::read_to_string(&manifest_file) {
                            if let Ok(manifest) = PluginManifest::parse_from_toml(&raw) {
                                if manifest.plugin.id == plugin_id {
                                    return Ok(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(AppError::NotFound(format!(
            "Plugin with ID '{plugin_id}' not found in {:?}",
            self.plugins_directory
        )))
    }

    /// Load and instantiate a sandboxed Extism Wasm plugin from disk.
    pub fn load_plugin(&self, plugin_id: &str) -> Result<(Plugin, PluginManifest), AppError> {
        let plugin_dir = self.resolve_plugin_dir(plugin_id)?;
        let manifest_path = plugin_dir.join("plugin.toml");

        if !manifest_path.exists() {
            return Err(AppError::NotFound(format!(
                "Plugin manifest not found at {:?}",
                manifest_path
            )));
        }

        let manifest_raw = std::fs::read_to_string(&manifest_path)?;
        let manifest = PluginManifest::parse_from_toml(&manifest_raw)?;

        let canonical_plugin_dir = plugin_dir
            .canonicalize()
            .map_err(|e| AppError::NotFound(format!("Plugin directory not accessible: {e}")))?;

        let wasm_file = plugin_dir.join(&manifest.binary.entrypoint);
        let canonical_wasm = wasm_file
            .canonicalize()
            .map_err(|e| AppError::NotFound(format!("Plugin binary not found: {e}")))?;

        if !canonical_wasm.starts_with(&canonical_plugin_dir) {
            return Err(AppError::InvalidInput(
                "ERR_PLUGIN_PATH_TRAVERSAL: Plugin binary is outside plugin directory".into(),
            ));
        }

        let wasm_bytes = std::fs::read(&canonical_wasm)?;

        // 1. Verify SHA-256 checksum
        manifest.verify_binary_sha256(&wasm_bytes)?;

        // 2. Configure Extism Manifest with linear memory cap & timeout
        let max_pages = (manifest.limits.max_memory_mb * 1024 * 1024 / 65536) as u32;
        let mut extism_manifest =
            Manifest::new([Wasm::data(wasm_bytes)]).with_memory_max(max_pages);

        if manifest.limits.timeout_ms > 0 {
            extism_manifest = extism_manifest
                .with_timeout(std::time::Duration::from_millis(manifest.limits.timeout_ms));
        }

        // 3. Build sandboxed plugin with fuel budget bounds
        let mut builder = PluginBuilder::new(extism_manifest).with_wasi(true);
        if manifest.limits.fuel_budget > 0 {
            builder = builder.with_fuel_limit(manifest.limits.fuel_budget);
        }

        let plugin = builder.build().map_err(|e| {
            AppError::InvalidInput(format!("Failed to instantiate Wasm sandbox: {e}"))
        })?;

        Ok((plugin, manifest))
    }

    /// Call an exported function on a plugin instance with input/output validation.
    pub fn invoke_function<I: Serialize, O: for<'de> Deserialize<'de>>(
        &self,
        plugin: &mut Plugin,
        function_name: &str,
        input: &I,
    ) -> Result<O, AppError> {
        let input_bytes = serde_json::to_vec(input).map_err(|e| {
            AppError::InvalidInput(format!("Failed to serialize plugin input: {e}"))
        })?;

        // Zero-Float Gateway: reject IEEE-754 floats on outbound host payload
        validate_no_raw_floats(&input_bytes)?;

        let output_bytes = plugin
            .call::<&[u8], &[u8]>(function_name, &input_bytes)
            .map_err(|e| {
                let err_str = e.to_string();
                let lower = err_str.to_lowercase();
                if lower.contains("fuel") {
                    AppError::InvalidInput(format!("ERR_PLUGIN_OUT_OF_FUEL: {err_str}"))
                } else if lower.contains("timeout") || lower.contains("deadline") {
                    AppError::InvalidInput(format!("ERR_PLUGIN_TIMEOUT: {err_str}"))
                } else if lower.contains("memory") || lower.contains("out of memory") {
                    AppError::InvalidInput(format!("ERR_PLUGIN_OOM: {err_str}"))
                } else {
                    AppError::InvalidInput(format!("Plugin execution failed: {err_str}"))
                }
            })?;

        // Zero-Float Gateway: unconditionally reject IEEE-754 floats in inbound plugin payload
        validate_no_raw_floats(output_bytes)?;

        let output: O = serde_json::from_slice(output_bytes).map_err(|e| {
            AppError::InvalidInput(format!("Failed to deserialize plugin output: {e}"))
        })?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_real_wasm_fuel_exhaustion() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("fuel_test_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let wat_src = r#"
            (module
                (func (export "parse_statement") (result i32)
                    (loop (br 0))
                    i32.const 0
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_src).unwrap();
        let wasm_hash = format!("{:x}", Sha256::digest(&wasm_bytes));

        fs::write(plugin_dir.join("plugin.wasm"), &wasm_bytes).unwrap();

        let manifest_toml = format!(
            r#"
manifest_version = 1
[plugin]
id = "org.finnca.test.fuel"
name = "Fuel Test"
version = "1.0.0"
[binary]
entrypoint = "plugin.wasm"
sha256 = "{wasm_hash}"
[limits]
max_memory_mb = 16
timeout_ms = 5000
fuel_budget = 500
"#
        );
        fs::write(plugin_dir.join("plugin.toml"), manifest_toml).unwrap();

        let manager = PluginManager::new(dir.path().to_path_buf());
        let (mut plugin, _manifest) = manager.load_plugin("org.finnca.test.fuel").unwrap();

        let dummy_input = serde_json::json!({"test": 123});
        let res: Result<serde_json::Value, AppError> =
            manager.invoke_function(&mut plugin, "parse_statement", &dummy_input);

        let err = res.unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_OUT_OF_FUEL"));
    }

    #[test]
    fn test_real_wasm_timeout() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("timeout_test_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let wat_src = r#"
            (module
                (func (export "parse_statement") (result i32)
                    (loop (br 0))
                    i32.const 0
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_src).unwrap();
        let wasm_hash = format!("{:x}", Sha256::digest(&wasm_bytes));

        fs::write(plugin_dir.join("plugin.wasm"), &wasm_bytes).unwrap();

        let manifest_toml = format!(
            r#"
manifest_version = 1
[plugin]
id = "org.finnca.test.timeout"
name = "Timeout Test"
version = "1.0.0"
[binary]
entrypoint = "plugin.wasm"
sha256 = "{wasm_hash}"
[limits]
max_memory_mb = 16
timeout_ms = 50
fuel_budget = 0
"#
        );
        fs::write(plugin_dir.join("plugin.toml"), manifest_toml).unwrap();

        let manager = PluginManager::new(dir.path().to_path_buf());
        let (mut plugin, _manifest) = manager.load_plugin("org.finnca.test.timeout").unwrap();

        let dummy_input = serde_json::json!({"test": 123});
        let res: Result<serde_json::Value, AppError> =
            manager.invoke_function(&mut plugin, "parse_statement", &dummy_input);

        let err = res.unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_TIMEOUT"));
    }

    #[test]
    fn test_real_wasm_zero_float_violation() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("float_test_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let wat_src = r#"
            (module
                (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
                (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
                (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))

                (func (export "parse_statement") (result i32)
                    (local $ptr i64)
                    ;; allocate 17 bytes for `{"amount": 100.5}`
                    (local.set $ptr (call $alloc (i64.const 17)))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 0)) (i32.const 123))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 1)) (i32.const 34))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 2)) (i32.const 97))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 3)) (i32.const 109))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 4)) (i32.const 111))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 5)) (i32.const 117))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 6)) (i32.const 110))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 7)) (i32.const 116))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 8)) (i32.const 34))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 9)) (i32.const 58))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 10)) (i32.const 32))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 11)) (i32.const 49))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 12)) (i32.const 48))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 13)) (i32.const 48))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 14)) (i32.const 46))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 15)) (i32.const 53))
                    (call $store_u8 (i64.add (local.get $ptr) (i64.const 16)) (i32.const 125))

                    (call $output_set (local.get $ptr) (i64.const 17))
                    i32.const 0
                )
            )
        "#;
        let wasm_bytes = wat::parse_str(wat_src).unwrap();
        let wasm_hash = format!("{:x}", Sha256::digest(&wasm_bytes));

        fs::write(plugin_dir.join("plugin.wasm"), &wasm_bytes).unwrap();

        let manifest_toml = format!(
            r#"
manifest_version = 1
[plugin]
id = "org.finnca.test.float"
name = "Float Test"
version = "1.0.0"
[binary]
entrypoint = "plugin.wasm"
sha256 = "{wasm_hash}"
[limits]
max_memory_mb = 16
timeout_ms = 5000
fuel_budget = 10000000
"#
        );
        fs::write(plugin_dir.join("plugin.toml"), manifest_toml).unwrap();

        let manager = PluginManager::new(dir.path().to_path_buf());
        let (mut plugin, _manifest) = manager.load_plugin("org.finnca.test.float").unwrap();

        let dummy_input = serde_json::json!({"test": 123});
        let res: Result<crate::plugins::statement::ParsedStatementOutput, AppError> =
            manager.invoke_function(&mut plugin, "parse_statement", &dummy_input);

        let err = res.unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_FLOAT_VIOLATION"));
    }

    #[test]
    fn test_real_wasm_statement_parser_success() {
        let dir = tempdir().unwrap();
        let plugin_dir = dir.path().join("valid_parser");
        fs::create_dir_all(&plugin_dir).unwrap();

        let json_str = r#"{"rows":[{"date":"2026-10-06","amount":150000}]}"#;
        let bytes = json_str.as_bytes();

        let mut store_instructions = String::new();
        for (i, b) in bytes.iter().enumerate() {
            store_instructions.push_str(&format!(
                "(call $store_u8 (i64.add (local.get $ptr) (i64.const {i})) (i32.const {b}))\n"
            ));
        }

        let wat_src = format!(
            r#"
            (module
                (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
                (import "extism:host/env" "output_set" (func $output_set (param i64 i64)))
                (import "extism:host/env" "store_u8" (func $store_u8 (param i64 i32)))

                (func (export "parse_statement") (result i32)
                    (local $ptr i64)
                    (local.set $ptr (call $alloc (i64.const {})))
                    {}
                    (call $output_set (local.get $ptr) (i64.const {}))
                    i32.const 0
                )
            )
        "#,
            bytes.len(),
            store_instructions,
            bytes.len()
        );

        let wasm_bytes = wat::parse_str(&wat_src).unwrap();
        let wasm_hash = format!("{:x}", Sha256::digest(&wasm_bytes));

        fs::write(plugin_dir.join("plugin.wasm"), &wasm_bytes).unwrap();

        let manifest_toml = format!(
            r#"
manifest_version = 1
[plugin]
id = "org.finnca.parser.valid"
name = "Valid Parser"
version = "1.0.0"
[binary]
entrypoint = "plugin.wasm"
sha256 = "{wasm_hash}"
[limits]
max_memory_mb = 16
timeout_ms = 5000
fuel_budget = 10000000
[extensions]
statement_parsers = [
  {{ id = "valid_csv", name = "Valid CSV" }}
]
"#
        );
        fs::write(plugin_dir.join("plugin.toml"), manifest_toml).unwrap();

        let manager = PluginManager::new(dir.path().to_path_buf());
        let (mut plugin, manifest) = manager.load_plugin("org.finnca.parser.valid").unwrap();
        assert_eq!(manifest.plugin.id, "org.finnca.parser.valid");

        let input = crate::plugins::statement::ParseStatementInput {
            parser_id: "valid_csv".into(),
            file_name: "test.csv".into(),
            file_bytes_base64: "".into(),
            account_currency: "IDR".into(),
            account_is_debit_normal: true,
            options: std::collections::HashMap::new(),
        };

        let output: crate::plugins::statement::ParsedStatementOutput = manager
            .invoke_function(&mut plugin, "parse_statement", &input)
            .unwrap();

        assert_eq!(output.rows.len(), 1);
        assert_eq!(output.rows[0].date, "2026-10-06");
        assert_eq!(output.rows[0].amount, 150000);
    }
}
