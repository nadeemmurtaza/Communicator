//! KB Documents Service

use crate::db::AppState;

pub async fn list_documents(_state: &AppState, _brand_id: Option<&str>, _query: Option<&str>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

pub async fn get_document(_state: &AppState, _doc_id: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn create_document(_state: &AppState, _brand_id: &str, _title: &str, _content: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn update_document(_state: &AppState, _doc_id: &str, _title: Option<&str>, _content: Option<&str>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn delete_document(_state: &AppState, _doc_id: &str) -> Result<(), String> {
    Ok(())
}

pub async fn search_documents(_state: &AppState, _query: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

pub async fn upload_document(_state: &AppState, _brand_id: &str, _filename: &str, _content_type: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn get_document_embeddings(_state: &AppState, _doc_id: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

pub async fn regenerate_embeddings(_state: &AppState, _doc_id: &str) -> Result<(), String> {
    Ok(())
}

pub async fn get_document_stats(_state: &AppState, _doc_id: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}
