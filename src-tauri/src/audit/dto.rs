use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AuditPage {
    pub entries: Vec<super::models::AuditEntry>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
