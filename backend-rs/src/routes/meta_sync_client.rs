//! Meta-sync library wrapper for backend-rs route files.
//!
//! Builds `newax_meta_sync::MetaSyncState` from `AppState` and delegates
//! to the library crate directly (no gRPC hop), matching the pattern used
//! by `alerts` and `reply-integration` library-only splits.

use crate::db::AppState;

fn to_ms_state(state: &AppState) -> newax_meta_sync::MetaSyncState {
    newax_meta_sync::MetaSyncState {
        db: state.db.clone(),
        http_client: state.http_client.clone(),
        meta_whatsapp_access_token: state.config.meta_whatsapp_access_token.clone(),
        meta_whatsapp_phone_number_id: state.config.meta_whatsapp_phone_number_id.clone(),
        meta_whatsapp_api_version: state.config.meta_whatsapp_api_version.clone(),
    }
}

pub async fn fetch_meta_business_profile(
    state: &AppState,
    access_token: &str,
    phone_number_id: &str,
    api_version: &str,
) -> newax_meta_sync::BusinessProfile {
    let ms = to_ms_state(state);
    newax_meta_sync::fetch_meta_business_profile(&ms, access_token, phone_number_id, api_version).await
}

pub async fn resolve_meta_credentials(state: &AppState) -> (String, String) {
    let ms = to_ms_state(state);
    newax_meta_sync::resolve_meta_credentials(&ms).await
}

pub async fn refresh_channel_info(state: &AppState) -> serde_json::Value {
    let ms = to_ms_state(state);
    newax_meta_sync::refresh_channel_info(&ms).await
}
