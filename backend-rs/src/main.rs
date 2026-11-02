#![recursion_limit = "512"]

mod config;
mod db;
mod models;
mod routes;
mod services;

use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response, StatusCode},
    routing::{get, post},
    Router,
};
use config::Config;
use db::{init_db, AppState};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend_rs=debug,tower_http=info".into()),
        )
        .init();

    info!("Starting Communicator Rust Backend (Desktop + Web Unified Engine)...");

    let mut config = Config::from_env();
    let vault_secrets = services::vault_loader::load_secrets_from_nid_vault(&config).await;
    services::vault_loader::apply_vault_secrets(&mut config, &vault_secrets);
    let config = Arc::new(config);

    let db = init_db(&config.database_url).await?;
    let http_client = reqwest::Client::builder().build()?;

    let state = AppState {
        db,
        config: Arc::clone(&config),
        http_client,
        nid: Arc::new(services::nid_client::NidClient::from_config(&config)),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Complete API Routes
    let api_routes = Router::new()
        // Health
        .route("/health", get(routes::health::health_check))

        // nID identity provider (Login with nID)
        .route("/auth/nid/start", get(routes::auth::login_with_nid_start))
        .route("/auth/nid/callback", get(routes::auth::login_with_nid_callback))
        .route("/auth/callback", get(routes::auth::login_with_nid_callback))
        .route("/auth/me", get(routes::auth::get_current_user_profile))
        
        // Business Profile
        .route("/business-profile", get(routes::business_profile::get_business_profile).post(routes::business_profile::update_business_profile))
        .route("/business-profile/sync-meta", post(routes::business_profile::sync_meta))

        // Conversations & Messages
        .route("/conversations", get(routes::conversations::list_conversations))
        .route("/conversations/{id}", get(routes::conversations::get_conversation))
        .route("/conversations/{id}/messages", post(routes::conversations::send_conversation_message))
        .route("/conversations/{id}/resolve", post(routes::conversations::resolve_conversation))
        .route("/conversations/{id}/priority", post(routes::conversations::update_priority))
        .route("/conversations/{id}/assign-team", post(routes::conversations::assign_team))
        .route("/conversations/{id}/assign-agent", post(routes::conversations::assign_agent))
        .route("/conversations/{id}/snooze", post(routes::conversations::snooze_conversation))
        .route("/conversations/{id}/custom-attributes", post(routes::conversations::update_custom_attributes))
        .route("/conversations/contacts/{contact_id}/notes", get(routes::conversations::list_contact_notes).post(routes::conversations::add_contact_note))
        .route("/conversations/{id}/enroll-sequence", post(routes::conversations::enroll_sequence))
        .route("/conversations/{id}/presence", get(routes::conversations::get_presence).post(routes::conversations::update_presence).delete(routes::conversations::update_presence))
        .route("/conversations/{id}/collision-check", get(routes::conversations::check_collision))
        .route("/conversations/{id}/auto-assign", post(routes::conversations::auto_assign))

        // Contacts
        .route("/contacts", get(routes::contacts::list_contacts))
        .route("/contacts/tags", get(routes::contacts::list_contact_tags))
        .route("/contacts/export", get(routes::contacts::export_contacts))
        .route("/contacts/sync-meta", post(routes::contacts::sync_meta_contacts))
        .route("/contacts/{id}", get(routes::contacts::get_contact))
        .route("/contacts/{id}/tags", post(routes::contacts::add_contact_tag))
        .route("/contacts/{id}/tags/{tag}", axum::routing::delete(routes::contacts::remove_contact_tag))
        .route("/contacts/{id}/toggle-consent", post(routes::contacts::toggle_contact_consent))
        .route("/contacts/{id}/sync-meta", post(routes::contacts::sync_contact_meta))
        .route("/contacts/{id}/conversation", post(routes::contacts::get_or_create_contact_conversation))
        .route("/contacts/{id}/memories", get(routes::contacts::list_contact_memories).post(routes::contacts::create_contact_memory))
        .route("/contacts/{id}/memories/{memory_id}", axum::routing::delete(routes::contacts::delete_contact_memory))

        // Inboxes & All 10 Channels
        .route("/inboxes", get(routes::inboxes::list_inboxes))
        .route("/channels/health", get(routes::inboxes::get_channels_health))
        .route("/inboxes/channels/whatsapp", get(routes::inboxes::get_whatsapp_channel).post(routes::inboxes::update_whatsapp_channel))
        .route("/inboxes/channels/whatsapp/verify", post(routes::inboxes::verify_whatsapp_connection))
        .route("/inboxes/channels/whatsapp/templates", get(routes::inboxes::list_channel_whatsapp_templates))
        .route("/inboxes/channels/whatsapp/sync-templates", post(routes::inboxes::sync_whatsapp_templates))
        .route("/inboxes/channels/whatsapp/test-message", post(routes::inboxes::send_whatsapp_test_message))
        .route("/inboxes/channels/whatsapp/events", get(routes::inboxes::list_whatsapp_events))
        .route("/inboxes/channels/whatsapp/profile", get(routes::inboxes::get_whatsapp_profile).post(routes::inboxes::update_whatsapp_profile))
        .route("/inboxes/channels/whatsapp/register", post(routes::inboxes::register_whatsapp_phone))

        .route("/inboxes/channels/live-chat", get(routes::inboxes::get_livechat_channel).post(routes::inboxes::update_livechat_channel))
        .route("/inboxes/channels/live-chat/regenerate-token", post(routes::inboxes::regenerate_livechat_token))
        .route("/inboxes/channels/live-chat/test-visitor", post(routes::inboxes::send_livechat_test_visitor))
        .route("/inboxes/channels/live-chat/events", get(routes::inboxes::list_livechat_events))
        .route("/inboxes/channels/live-chat/prechat", get(routes::inboxes::get_prechat_config).post(routes::inboxes::update_prechat_config))
        .route("/inboxes/channels/live-chat/prechat/reset", post(routes::inboxes::reset_prechat_config))
        .route("/inboxes/channels/live-chat/prechat/test", post(routes::inboxes::test_prechat_intake))

        .route("/inboxes/channels/sms", get(routes::inboxes::get_sms_channel).post(routes::inboxes::update_sms_channel))
        .route("/inboxes/channels/sms/verify", post(routes::inboxes::verify_sms_connection))
        .route("/inboxes/channels/sms/test-message", post(routes::inboxes::send_sms_test_message))
        .route("/inboxes/channels/sms/events", get(routes::inboxes::list_sms_events))

        .route("/inboxes/channels/facebook", get(routes::inboxes::get_facebook_channel).post(routes::inboxes::update_facebook_channel))
        .route("/inboxes/channels/facebook/verify", post(routes::inboxes::verify_facebook_connection))
        .route("/inboxes/channels/facebook/test-message", post(routes::inboxes::send_facebook_test_message))
        .route("/inboxes/channels/facebook/events", get(routes::inboxes::list_facebook_events))

        .route("/inboxes/channels/instagram", get(routes::inboxes::get_instagram_channel).post(routes::inboxes::update_instagram_channel))
        .route("/inboxes/channels/instagram/verify", post(routes::inboxes::verify_instagram_connection))
        .route("/inboxes/channels/instagram/test-message", post(routes::inboxes::send_instagram_test_message))
        .route("/inboxes/channels/instagram/events", get(routes::inboxes::list_instagram_events))

        .route("/inboxes/channels/telegram", get(routes::inboxes::get_telegram_channel).post(routes::inboxes::update_telegram_channel))
        .route("/inboxes/channels/telegram/verify", post(routes::inboxes::verify_telegram_connection))
        .route("/inboxes/channels/telegram/test-message", post(routes::inboxes::send_telegram_test_message))
        .route("/inboxes/channels/telegram/events", get(routes::inboxes::list_telegram_events))

        .route("/inboxes/channels/line", get(routes::inboxes::get_line_channel).post(routes::inboxes::update_line_channel))
        .route("/inboxes/channels/line/verify", post(routes::inboxes::verify_line_connection))
        .route("/inboxes/channels/line/test-message", post(routes::inboxes::send_line_test_message))
        .route("/inboxes/channels/line/events", get(routes::inboxes::list_line_events))

        .route("/inboxes/channels/voice", get(routes::inboxes::get_voice_channel).post(routes::inboxes::update_voice_channel))
        .route("/inboxes/channels/voice/verify", post(routes::inboxes::verify_voice_channel))
        .route("/inboxes/channels/voice/token", post(routes::inboxes::get_voice_webrtc_token))
        .route("/inboxes/channels/voice/call", post(routes::inboxes::initiate_voice_call))
        .route("/inboxes/channels/voice/hangup", post(routes::inboxes::hangup_voice_call))
        .route("/inboxes/channels/voice/call-logs", get(routes::inboxes::get_voice_call_logs))
        .route("/inboxes/channels/voice/events", get(routes::inboxes::list_voice_events))

        .route("/inboxes/channels/email", get(routes::inboxes::get_email_channel).post(routes::inboxes::update_email_channel))
        .route("/inboxes/channels/email/verify", post(routes::inboxes::verify_email_channel))
        .route("/inboxes/channels/email/test-message", post(routes::inboxes::send_email_test_message))
        .route("/inboxes/channels/email/sync-imap", post(routes::inboxes::sync_email_imap))
        .route("/inboxes/channels/email/events", get(routes::inboxes::list_email_events))

        .route("/inboxes/channels/tiktok", get(routes::inboxes::get_tiktok_channel).post(routes::inboxes::update_tiktok_channel))
        .route("/inboxes/channels/tiktok/verify", post(routes::inboxes::verify_tiktok_connection))
        .route("/inboxes/channels/tiktok/test-message", post(routes::inboxes::send_tiktok_test_message))
        .route("/inboxes/channels/tiktok/test-comment-reply", post(routes::inboxes::send_tiktok_test_comment_reply))
        .route("/inboxes/channels/tiktok/events", get(routes::inboxes::list_tiktok_events))

        // Drafts
        .route("/drafts", get(routes::drafts::list_drafts))
        .route("/drafts/{id}/approve", post(routes::drafts::approve_draft))
        .route("/drafts/{id}/reject", post(routes::drafts::reject_draft))
        .route("/drafts/{id}/edit", post(routes::drafts::edit_draft))
        .route("/drafts/generate/{conversation_id}", post(routes::drafts::generate_draft))

        // Broadcasts / Campaigns
        .route("/broadcasts", get(routes::broadcasts::list_broadcasts).post(routes::broadcasts::create_broadcast))
        .route("/broadcasts/audience-preview", get(routes::broadcasts::get_audience_preview))
        .route("/broadcasts/{id}/analytics", get(routes::broadcasts::get_campaign_analytics))
        .route("/broadcasts/{id}/reschedule", post(routes::broadcasts::reschedule_campaign))
        .route("/broadcasts/{id}/dispatch", post(routes::broadcasts::dispatch_broadcast))
        .route("/broadcasts/process-scheduled", post(routes::broadcasts::process_scheduled_campaigns))
        .route("/broadcasts/{id}/recipients", get(routes::broadcasts::list_broadcast_recipients))
        .route("/broadcasts/pricing-calculator", get(routes::broadcasts::get_whatsapp_pricing))
        .route("/broadcasts/seed-defaults", post(routes::broadcasts::seed_default_campaigns))

        // Services & Sequences
        .route("/services", get(routes::services_sequences::list_services).post(routes::services_sequences::create_service))
        .route("/sequences", get(routes::services_sequences::list_sequences).post(routes::services_sequences::create_sequence))
        .route("/sequences/{id}/variant-stats", get(routes::services_sequences::sequence_variant_stats))

        // Team & Routing
        .route("/team", get(routes::team_routing::list_team).post(routes::team_routing::create_team_member))
        .route("/team/{id}", axum::routing::delete(routes::team_routing::delete_team_member))
        .route("/team/conversations/{id}/assign", post(routes::team_routing::assign_conversation_team))
        .route("/team/conversations/{id}/takeover", post(routes::team_routing::set_human_takeover))
        .route("/teams", get(routes::team_routing::list_teams).post(routes::team_routing::create_team))
        .route("/teams/{id}", axum::routing::delete(routes::team_routing::delete_team))
        .route("/routing/agents", get(routes::team_routing::list_routing_agents))
        .route("/routing/agents/{id}/status", axum::routing::put(routes::team_routing::update_agent_status))
        .route("/routing/agents/{id}/capacity", axum::routing::put(routes::team_routing::update_agent_capacity))
        .route("/routing/capacity-overview", get(routes::team_routing::get_capacity_overview))
        .route("/routing/auto-assign-batch", post(routes::team_routing::batch_auto_assign))

        // Knowledge Base
        .route("/knowledge-base", get(routes::knowledge_base::list_knowledge_base).post(routes::knowledge_base::create_knowledge_base_entry))
        .route("/knowledge-base/{id}", axum::routing::delete(routes::knowledge_base::delete_knowledge_base_entry))
        .route("/knowledge-base/ingest-url", post(routes::knowledge_base::ingest_url))
        .route("/knowledge-base/hub", get(routes::knowledge_base::get_knowledge_hub_overview))
        .route("/knowledge-base/recommendations", get(routes::knowledge_base::list_knowledge_recommendations))
        .route("/knowledge-base/recommendations/{id}/apply", post(routes::knowledge_base::apply_knowledge_recommendation))
        .route("/knowledge-base/recommendations/{id}/dismiss", post(routes::knowledge_base::dismiss_knowledge_recommendation))
        .route("/knowledge-base/gaps", get(routes::knowledge_base::list_knowledge_gaps))
        .route("/knowledge-base/gaps/scan", post(routes::knowledge_base::scan_knowledge_gaps))
        .route("/knowledge-base/gaps/{id}/approve", post(routes::knowledge_base::approve_knowledge_gap))
        .route("/knowledge-base/gaps/{id}/dismiss", post(routes::knowledge_base::dismiss_knowledge_gap))

        // Automation Rules
        .route("/automation-rules", get(routes::automation::list_automation_rules).post(routes::automation::create_automation_rule))
        .route("/automation-rules/{id}", axum::routing::put(routes::automation::update_automation_rule).delete(routes::automation::delete_automation_rule))
        .route("/automation-rules/seed-defaults", post(routes::automation::seed_default_automations))
        .route("/automation-rules/test", post(routes::automation::test_automation_rule))
        .route("/automation-rules/stats", get(routes::automation::get_automation_stats))
        .route("/automation-rules/run-nudges", post(routes::automation::run_inactivity_nudges))

        // Canned Responses & Macros & SLA
        .route("/canned-responses", get(routes::canned_macros_sla::list_canned_responses).post(routes::canned_macros_sla::create_canned_response))
        .route("/canned-responses/{id}", axum::routing::put(routes::canned_macros_sla::update_canned_response).delete(routes::canned_macros_sla::delete_canned_response))
        .route("/canned-responses/expand", post(routes::canned_macros_sla::expand_canned_response))
        .route("/macros", get(routes::canned_macros_sla::list_macros).post(routes::canned_macros_sla::create_macro))
        .route("/macros/{id}", get(routes::canned_macros_sla::get_macro).put(routes::canned_macros_sla::update_macro).delete(routes::canned_macros_sla::delete_macro))
        .route("/conversations/{id}/macros/{macro_id}/execute", post(routes::canned_macros_sla::execute_macro))
        .route("/sla/policies", get(routes::canned_macros_sla::list_sla_policies).post(routes::canned_macros_sla::create_sla_policy))
        .route("/sla/policies/{id}", get(routes::canned_macros_sla::get_sla_policy).put(routes::canned_macros_sla::update_sla_policy).delete(routes::canned_macros_sla::delete_sla_policy))
        .route("/sla/metrics", get(routes::canned_macros_sla::get_sla_metrics))
        .route("/sla/breaches", get(routes::canned_macros_sla::get_sla_breaches))
        .route("/conversations/{id}/sla/evaluate", post(routes::canned_macros_sla::evaluate_conversation_sla))

        // Tickets & Issue Trackers
        .route("/tickets", get(routes::tickets::list_tickets).post(routes::tickets::create_ticket))
        .route("/tickets/stats", get(routes::tickets::get_ticket_stats))
        .route("/tickets/trackers", get(routes::tickets::get_trackers))
        .route("/tickets/{id}", get(routes::tickets::get_ticket).patch(routes::tickets::update_ticket))
        .route("/tickets/{id}/activities", post(routes::tickets::add_ticket_activity))
        .route("/tickets/{id}/link-tracker", post(routes::tickets::link_ticket_to_tracker))
        .route("/tickets/{id}/broadcast", post(routes::tickets::broadcast_tracker_update))
        .route("/integrations/trackers/configs", get(routes::tickets::list_tracker_configs))
        .route("/integrations/trackers/configs/{provider}", get(routes::tickets::get_tracker_config).post(routes::tickets::update_tracker_config))
        .route("/integrations/trackers/test/{provider}", post(routes::tickets::test_tracker_connection))
        .route("/integrations/trackers/linked", get(routes::tickets::list_linked_issues))
        .route("/integrations/trackers/create-issue", post(routes::tickets::create_linked_issue))
        .route("/integrations/trackers/linked/{id}", axum::routing::delete(routes::tickets::unlink_issue))
        .route("/integrations/trackers/linked/{id}/sync", post(routes::tickets::sync_issue_status))
        .route("/integrations/trackers/slack/sync", post(routes::tickets::sync_conversation_to_slack))
        .route("/integrations/trackers/slack/status/{id}", get(routes::tickets::get_slack_sync_status))

        // Copilot AI & Simulations
        .route("/copilot/approvals", get(routes::copilot_ai::list_copilot_approvals).post(routes::copilot_ai::create_copilot_approval))
        .route("/copilot/approvals/{id}", get(routes::copilot_ai::get_copilot_approval))
        .route("/copilot/approvals/{id}/approve", post(routes::copilot_ai::approve_copilot_request))
        .route("/copilot/approvals/{id}/reject", post(routes::copilot_ai::reject_copilot_request))
        .route("/copilot/approvals/{id}/provide-info", post(routes::copilot_ai::provide_copilot_info))
        .route("/copilot/approvals/evaluate-trigger", post(routes::copilot_ai::evaluate_copilot_trigger))
        .route("/simulations/policies", get(routes::copilot_ai::list_agent_policies).post(routes::copilot_ai::create_agent_policy))
        .route("/simulations/scenarios", get(routes::copilot_ai::list_simulation_scenarios))
        .route("/simulations/run", post(routes::copilot_ai::run_simulation))
        .route("/simulations/batch", post(routes::copilot_ai::run_batch_simulations))
        .route("/simulations/sandbox/chat", post(routes::copilot_ai::sandbox_chat))
        .route("/simulations/stats", get(routes::copilot_ai::get_simulation_stats))
        .route("/self-improving/insights", get(routes::copilot_ai::list_self_improving_insights))
        .route("/self-improving/insights/{id}/apply", post(routes::copilot_ai::apply_insight_retraining))
        .route("/self-improving/insights/{id}/dismiss", post(routes::copilot_ai::dismiss_insight))
        .route("/self-improving/retraining-history", get(routes::copilot_ai::list_retraining_history))
        .route("/self-improving/scan", post(routes::copilot_ai::trigger_auto_learning_scan))
        .route("/self-improving/stats", get(routes::copilot_ai::get_self_improving_stats))
        .route("/ai/connections", get(routes::copilot_ai::list_ai_connections).post(routes::copilot_ai::create_ai_connection))
        .route("/ai/connections/{id}", axum::routing::delete(routes::copilot_ai::delete_ai_connection))
        .route("/ai/connections/{id}/test", post(routes::copilot_ai::test_ai_connection))
        .route("/ai/jobs/mapping", get(routes::copilot_ai::get_job_model_mapping).post(routes::copilot_ai::assign_model_to_job))
        .route("/ai/policies/{job_type}", get(routes::copilot_ai::get_custom_policy).put(routes::copilot_ai::update_custom_policy))
        .route("/ai/prompts/{job_type}", get(routes::copilot_ai::get_effective_prompt))
        .route("/conversations/{id}/ai-copilot/suggested-reply", post(routes::copilot_ai::get_captain_suggested_reply))
        .route("/conversations/{id}/ai-copilot/summarize", post(routes::copilot_ai::summarize_conversation_thread))
        .route("/conversations/{id}/ai-copilot/triage", post(routes::copilot_ai::triage_conversation))
        .route("/ai-copilot/rewrite", post(routes::copilot_ai::rewrite_draft_text))
        .route("/captain/monitors", get(routes::copilot_ai::get_captain_monitors).post(routes::copilot_ai::create_captain_monitor))

        // Gateways, Outgoing Webhooks & Stripe Billing
        .route("/gateways", get(routes::gateways_webhooks_stripe::list_gateways).post(routes::gateways_webhooks_stripe::create_gateway))
        .route("/gateways/providers/supported", get(routes::gateways_webhooks_stripe::get_supported_gateway_providers))
        .route("/gateways/{id}", axum::routing::get(routes::gateways_webhooks_stripe::get_single_gateway).put(routes::gateways_webhooks_stripe::update_gateway).delete(routes::gateways_webhooks_stripe::delete_gateway))
        .route("/gateways/{id}/test", post(routes::gateways_webhooks_stripe::test_gateway_connection))
        .route("/gateways/{id}/set-default", post(routes::gateways_webhooks_stripe::set_default_gateway))
        .route("/gateways/{id}/ping", post(routes::gateways_webhooks_stripe::send_gateway_ping))
        .route("/outgoing-webhooks", get(routes::gateways_webhooks_stripe::list_webhook_endpoints).post(routes::gateways_webhooks_stripe::create_webhook_endpoint))
        .route("/outgoing-webhooks/events/catalog", get(routes::gateways_webhooks_stripe::get_webhook_catalog))
        .route("/outgoing-webhooks/deliveries", get(routes::gateways_webhooks_stripe::get_webhook_deliveries))
        .route("/outgoing-webhooks/{id}/deliveries", get(routes::gateways_webhooks_stripe::get_endpoint_deliveries))
        .route("/outgoing-webhooks/{id}", axum::routing::get(routes::gateways_webhooks_stripe::get_single_webhook_endpoint).put(routes::gateways_webhooks_stripe::update_webhook_endpoint).delete(routes::gateways_webhooks_stripe::delete_webhook_endpoint))
        .route("/outgoing-webhooks/{id}/test", post(routes::gateways_webhooks_stripe::test_webhook_ping))
        .route("/outgoing-webhooks/deliveries/{id}/resend", post(routes::gateways_webhooks_stripe::resend_webhook_delivery))
        .route("/integrations/stripe/billing/{identifier}", get(routes::gateways_webhooks_stripe::get_stripe_billing))
        .route("/integrations/stripe/invoices/{identifier}", get(routes::gateways_webhooks_stripe::get_stripe_invoices))
        .route("/integrations/stripe/refund", post(routes::gateways_webhooks_stripe::process_stripe_refund))
        .route("/integrations/stripe/portal-session", post(routes::gateways_webhooks_stripe::create_stripe_portal_session))
        .route("/integrations/stripe/logs/{identifier}", get(routes::gateways_webhooks_stripe::get_stripe_billing_logs))
        .route("/integrations/stripe/config", get(routes::gateways_webhooks_stripe::get_stripe_config).post(routes::gateways_webhooks_stripe::update_stripe_config))
        .route("/integrations/stripe/test", post(routes::gateways_webhooks_stripe::test_stripe_connection))

        // Admin, Customizer, Audit, RBAC, SSO, Custom Attributes, Organizations, Segments, Settings
        .route("/widget/customizer", get(routes::admin_system::get_widget_customizer).post(routes::admin_system::update_widget_customizer))
        .route("/widget/customizer/reset", post(routes::admin_system::reset_widget_customizer))
        .route("/widget/customizer/embed-code", get(routes::admin_system::get_widget_embed_code))
        .route("/audit-logs", get(routes::admin_system::get_audit_logs).post(routes::admin_system::record_audit_event))
        .route("/audit-logs/stats", get(routes::admin_system::get_audit_stats))
        .route("/audit-logs/export", get(routes::admin_system::export_audit_logs))
        .route("/roles", get(routes::admin_system::list_roles).post(routes::admin_system::create_role))
        .route("/roles/permissions", get(routes::admin_system::get_permissions_catalog))
        .route("/roles/{id}", get(routes::admin_system::get_role_detail).put(routes::admin_system::update_role).delete(routes::admin_system::delete_role))
        .route("/roles/members/{id}/assign", post(routes::admin_system::assign_member_role))
        .route("/roles/members/{id}/check", get(routes::admin_system::check_member_permission))
        .route("/sso/config", get(routes::admin_system::get_sso_config).put(routes::admin_system::update_sso_config))
        .route("/sso/test", post(routes::admin_system::test_sso_connection))
        .route("/sso/check", post(routes::admin_system::check_sso_email))
        .route("/translation/languages", get(routes::admin_system::get_translation_languages))
        .route("/translation/settings", get(routes::admin_system::get_translation_settings).put(routes::admin_system::update_translation_settings))
        .route("/translation/detect", post(routes::admin_system::detect_language))
        .route("/translation/messages/{id}/translate", post(routes::admin_system::translate_message))
        .route("/translation/conversations/{id}/prepare-outbound", post(routes::admin_system::prepare_outbound_translation))
        .route("/custom-attributes", get(routes::admin_system::get_custom_attributes).post(routes::admin_system::create_custom_attribute))
        .route("/custom-attributes/{id}", axum::routing::put(routes::admin_system::update_custom_attribute).delete(routes::admin_system::delete_custom_attribute))
        .route("/custom-attributes/contacts/{id}/values", post(routes::admin_system::update_contact_attribute_values))
        .route("/custom-attributes/conversations/{id}/values", post(routes::admin_system::update_conv_attribute_values))
        .route("/organizations", get(routes::admin_system::list_organizations).post(routes::admin_system::create_organization))
        .route("/segments", get(routes::admin_system::list_segments).post(routes::admin_system::create_segment))
        .route("/segments/{id}", get(routes::admin_system::get_segment).put(routes::admin_system::update_segment).delete(routes::admin_system::delete_segment))
        .route("/segments/{id}/export", get(routes::admin_system::export_segment))
        .route("/scheduler/run-all", post(routes::admin_system::run_all_schedulers))
        .route("/settings", get(routes::admin_system::get_integration_settings).put(routes::admin_system::update_integration_settings))
        .route("/settings/whatsapp/verify", post(routes::admin_system::verify_whatsapp_settings))
        .route("/settings/whatsapp/templates", get(routes::admin_system::list_whatsapp_templates).post(routes::admin_system::create_whatsapp_template))
        .route("/settings/whatsapp/templates/{name}", axum::routing::delete(routes::admin_system::delete_whatsapp_template))
        .route("/reports/live", get(routes::admin_system::get_live_report))
        .route("/reports/conversations", get(routes::admin_system::get_conversation_reports))
        .route("/reports/agents", get(routes::admin_system::get_agent_reports))
        .route("/reports/teams", get(routes::admin_system::get_team_reports))
        .route("/reports/csat", get(routes::admin_system::get_csat_report))

        // Webhooks
        .route(
            "/webhooks/whatsapp",
            get(routes::webhooks::verify_whatsapp_webhook)
                .post(routes::webhooks::handle_whatsapp_webhook),
        );

    // Static assets & SPA fallback
    let static_dir = PathBuf::from(&config.static_dir);
    let assets_dir = static_dir.join("assets");
    let index_file = static_dir.join("index.html");

    info!("Static directory configured at: {:?}", static_dir);
    info!("Index file exists: {}", index_file.exists());

    let mut app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(cors)
        .with_state(state);

    if assets_dir.exists() {
        info!("Mounting static assets from {:?}", assets_dir);
        app = app.nest_service("/assets", ServeDir::new(&assets_dir));
    }

    // Serve static files in root like favicon.svg, icons.svg and SPA fallback
    let static_dir_clone = static_dir.clone();
    let index_path = index_file.clone();

    app = app.fallback(move |req: Request<Body>| {
        let uri_path = req.uri().path().trim_start_matches('/').to_string();
        let static_dir = static_dir_clone.clone();
        let fallback_index = index_path.clone();

        async move {
            let requested_file = static_dir.join(&uri_path);
            if !uri_path.is_empty() && requested_file.is_file() {
                if let Ok(bytes) = tokio::fs::read(&requested_file).await {
                    let mut res = Response::new(Body::from(bytes));
                    let content_type = if uri_path.ends_with(".svg") {
                        "image/svg+xml"
                    } else if uri_path.ends_with(".js") {
                        "application/javascript"
                    } else if uri_path.ends_with(".css") {
                        "text/css"
                    } else if uri_path.ends_with(".json") {
                        "application/json"
                    } else if uri_path.ends_with(".png") {
                        "image/png"
                    } else if uri_path.ends_with(".ico") {
                        "image/x-icon"
                    } else {
                        "application/octet-stream"
                    };
                    res.headers_mut().insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(content_type),
                    );
                    return res;
                }
            }

            match tokio::fs::read(&fallback_index).await {
                Ok(bytes) => {
                    let mut res = Response::new(Body::from(bytes));
                    res.headers_mut().insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/html; charset=utf-8"),
                    );
                    res.headers_mut().insert(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("no-store, must-revalidate"),
                    );
                    res
                }
                Err(_) => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found - Frontend build not found"))
                    .unwrap(),
            }
        }
    });

    let addr = SocketAddr::new(config.host.parse()?, config.port);
    info!("Rust backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
