use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AuditEntry {
    pub id: String,
    pub actor: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub detail: Option<String>,
    pub created_at: i64,
    pub prev_hash: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AuditIntegrityReport {
    pub is_valid: bool,
    pub total_verified: i64,
    pub broken_index: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    CreateAccount,
    UpdateAccount,
    DeleteAccount,
    PostJournalEntry,
    UpdateJournalEntry,
    DeleteJournalEntry,
    UpsertBudget,
    DeleteBudget,
    CreatePlan,
    UpdatePlan,
    DeletePlan,
    ReconcileAccount,
    VaultUnlocked,
    VaultLocked,
    UpdateClosingDate,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditAction::CreateAccount => "CREATE_ACCOUNT",
            AuditAction::UpdateAccount => "UPDATE_ACCOUNT",
            AuditAction::DeleteAccount => "DELETE_ACCOUNT",
            AuditAction::PostJournalEntry => "POST_JOURNAL_ENTRY",
            AuditAction::UpdateJournalEntry => "UPDATE_JOURNAL_ENTRY",
            AuditAction::DeleteJournalEntry => "DELETE_JOURNAL_ENTRY",
            AuditAction::UpsertBudget => "UPSERT_BUDGET",
            AuditAction::DeleteBudget => "DELETE_BUDGET",
            AuditAction::CreatePlan => "CREATE_PLAN",
            AuditAction::UpdatePlan => "UPDATE_PLAN",
            AuditAction::DeletePlan => "DELETE_PLAN",
            AuditAction::ReconcileAccount => "RECONCILE_ACCOUNT",
            AuditAction::VaultUnlocked => "VAULT_UNLOCKED",
            AuditAction::VaultLocked => "VAULT_LOCKED",
            AuditAction::UpdateClosingDate => "UPDATE_CLOSING_DATE",
        }
    }
}
