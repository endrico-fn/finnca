use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::shared::AppError;

pub const DEFAULT_MAX_MEMORY_MB: u64 = 32;
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_FUEL_BUDGET: u64 = 10_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub min_finnca_version: Option<String>,
    #[serde(default)]
    pub target_abi_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginBinary {
    pub entrypoint: String,
    pub sha256: String,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginLimits {
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_fuel_budget")]
    pub fuel_budget: u64,
}

fn default_max_memory_mb() -> u64 {
    DEFAULT_MAX_MEMORY_MB
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_fuel_budget() -> u64 {
    DEFAULT_FUEL_BUDGET
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: DEFAULT_MAX_MEMORY_MB,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            fuel_budget: DEFAULT_FUEL_BUDGET,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginPermissions {
    #[serde(default)]
    pub filesystem: Vec<String>,
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default = "default_ledger_permission")]
    pub ledger: String,
    #[serde(default)]
    pub notifications: bool,
}

fn default_ledger_permission() -> String {
    "none".to_string()
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            filesystem: Vec::new(),
            network: Vec::new(),
            ledger: default_ledger_permission(),
            notifications: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StatementParserExtension {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub file_extensions: Vec<String>,
    #[serde(default)]
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct PluginExtensions {
    #[serde(default)]
    pub statement_parsers: Vec<StatementParserExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct PluginManifest {
    pub manifest_version: u32,
    pub plugin: PluginMeta,
    pub binary: PluginBinary,
    #[serde(default)]
    pub limits: PluginLimits,
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub extensions: PluginExtensions,
}

impl PluginManifest {
    pub fn parse_from_toml(raw: &str) -> Result<Self, AppError> {
        let manifest: Self = toml::from_str(raw)
            .map_err(|e| AppError::InvalidInput(format!("Invalid plugin manifest TOML: {e}")))?;

        if manifest.manifest_version != 1 {
            return Err(AppError::InvalidInput(format!(
                "Unsupported manifest_version: {}. Expected 1.",
                manifest.manifest_version
            )));
        }
        if manifest.plugin.id.trim().is_empty() {
            return Err(AppError::InvalidInput("Plugin ID cannot be empty".into()));
        }
        let entrypoint = manifest.binary.entrypoint.trim();
        if entrypoint.is_empty() {
            return Err(AppError::InvalidInput(
                "Plugin binary entrypoint cannot be empty".into(),
            ));
        }
        let entrypoint_path = std::path::Path::new(entrypoint);
        if entrypoint_path.is_absolute()
            || entrypoint.contains("..")
            || entrypoint.starts_with('/')
            || entrypoint.starts_with('\\')
        {
            return Err(AppError::InvalidInput(
                "ERR_PLUGIN_PATH_TRAVERSAL: Plugin binary entrypoint cannot be absolute or contain path traversal ('..')".into(),
            ));
        }

        let sha = manifest.binary.sha256.trim();
        if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::InvalidInput(
                "Plugin binary sha256 must be a valid 64-character hex string".into(),
            ));
        }

        Ok(manifest)
    }

    pub fn verify_binary_sha256(&self, wasm_bytes: &[u8]) -> Result<(), AppError> {
        let calculated = format!("{:x}", Sha256::digest(wasm_bytes));
        let expected = self.binary.sha256.trim().to_lowercase();
        if calculated.to_lowercase() != expected {
            return Err(AppError::Crypto(format!(
                "Integritas plugin gagal: checksum hash tidak cocok! Harapan: {}, Terkalkulasi: {}",
                expected, calculated
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parsing_and_sha256_verification() {
        let toml_str = r#"
manifest_version = 1

[plugin]
id = "org.finnca.parser.test"
name = "Test Statement Parser"
version = "1.0.0"
author = "Finnca Team"
description = "Test parser description"

[binary]
entrypoint = "test.wasm"
sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

[limits]
max_memory_mb = 16
timeout_ms = 3000
fuel_budget = 5000000

[permissions]
filesystem = ["scoped_read"]
network = []
ledger = "none"
notifications = true

[extensions]
statement_parsers = [
  { id = "test_csv", name = "Test CSV Parser", file_extensions = ["csv"], mime_types = ["text/csv"] }
]
"#;

        let manifest =
            PluginManifest::parse_from_toml(toml_str).expect("Valid manifest should parse");
        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.plugin.id, "org.finnca.parser.test");
        assert_eq!(manifest.limits.max_memory_mb, 16);
        assert_eq!(manifest.limits.timeout_ms, 3000);
        assert_eq!(manifest.limits.fuel_budget, 5000000);
        assert_eq!(manifest.extensions.statement_parsers.len(), 1);
        assert_eq!(manifest.extensions.statement_parsers[0].id, "test_csv");

        // "abc" has sha256 ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let valid_bytes = b"abc";
        assert!(manifest.verify_binary_sha256(valid_bytes).is_ok());

        let invalid_bytes = b"corrupted bytes";
        assert!(manifest.verify_binary_sha256(invalid_bytes).is_err());
    }

    #[test]
    fn test_manifest_parsing_rejects_path_traversal() {
        let toml_traversal = r#"
manifest_version = 1
[plugin]
id = "org.finnca.malicious"
name = "Malicious"
version = "1.0.0"
[binary]
entrypoint = "../../etc/shadow"
sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
"#;
        let err = PluginManifest::parse_from_toml(toml_traversal).unwrap_err();
        assert!(err.to_string().contains("ERR_PLUGIN_PATH_TRAVERSAL"));

        let toml_absolute = r#"
manifest_version = 1
[plugin]
id = "org.finnca.malicious"
name = "Malicious"
version = "1.0.0"
[binary]
entrypoint = "/usr/lib/plugin.wasm"
sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
"#;
        let err2 = PluginManifest::parse_from_toml(toml_absolute).unwrap_err();
        assert!(err2.to_string().contains("ERR_PLUGIN_PATH_TRAVERSAL"));
    }

    #[test]
    fn test_manifest_parsing_rejects_invalid_sha256() {
        let toml_bad_sha = r#"
manifest_version = 1
[plugin]
id = "org.finnca.bad"
name = "Bad"
version = "1.0.0"
[binary]
entrypoint = "plugin.wasm"
sha256 = "not_a_valid_sha256"
"#;
        let err = PluginManifest::parse_from_toml(toml_bad_sha).unwrap_err();
        assert!(err.to_string().contains("64-character hex"));
    }
}
