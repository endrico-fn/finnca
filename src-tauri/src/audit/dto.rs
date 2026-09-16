use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuditPage {
    pub entries: Vec<super::models::AuditEntry>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
