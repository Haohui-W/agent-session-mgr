pub mod claude;
pub mod utils;

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    pub provider_id: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedMessages {
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub messages: Vec<SessionMessage>,
}

pub fn scan_sessions() -> Vec<SessionMeta> {
    claude::scan_sessions()
}

pub fn load_messages_paginated(
    source_path: &str,
    offset: usize,
    limit: usize,
) -> Result<PaginatedMessages, String> {
    let path = Path::new(source_path);
    let all = claude::load_messages(path)?;
    let total = all.len();
    let start = offset.min(total);
    let end = (offset + limit).min(total);
    // Reverse so newest messages come first
    let reversed: Vec<_> = all.into_iter().rev().collect();
    Ok(PaginatedMessages {
        total,
        offset: start,
        limit,
        messages: reversed[start..end].to_vec(),
    })
}

pub fn delete_session(session_id: &str, source_path: &str) -> Result<bool, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let root = home.join(".claude").join("projects");
    let path = Path::new(source_path);
    let validated_source = canonicalize_existing_path(path, "session source")?;
    let validated_root = canonicalize_existing_path(&root, "session root")?;

    if !validated_source.starts_with(&validated_root) {
        return Err(format!(
            "Session source path is outside Claude projects root: {}",
            path.display()
        ));
    }

    claude::delete_session(&validated_root, &validated_source, session_id)
}

fn canonicalize_existing_path(path: &Path, label: &str) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!("{label} not found: {}", path.display()));
    }
    path.canonicalize()
        .map_err(|e| format!("Failed to resolve {label} {}: {e}", path.display()))
}
