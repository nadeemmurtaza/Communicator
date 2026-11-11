//! Unified Inbox Service

use crate::db::AppState;

pub async fn list_messages(_state: &AppState, _brand_id: Option<&str>, _channel: Option<&str>, _status: Option<&str>, _limit: i64, _offset: i64) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!([]))
}

pub async fn get_message(_state: &AppState, _message_id: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn reply_to_message(_state: &AppState, _message_id: &str, _text: &str, _channel: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn assign_message(_state: &AppState, _message_id: &str, _agent_id: &str) -> Result<(), String> {
    Ok(())
}

pub async fn update_message_status(_state: &AppState, _message_id: &str, _status: &str) -> Result<(), String> {
    Ok(())
}

pub async fn add_note_to_message(_state: &AppState, _message_id: &str, _note: &str) -> Result<(), String> {
    Ok(())
}

pub async fn get_inbox_stats(_state: &AppState, _brand_id: &str) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

pub async fn get_unread_count(_state: &AppState, _brand_id: &str) -> Result<i64, String> {
    Ok(0)
}
