#![recursion_limit = "512"]

mod config;
mod db;
mod models;
mod routes;
mod services;

use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post, put},
    Router,
};
use config::Config;
use db::{init_db, AppState};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // RUST_LOG is optional, but a rejected value must be visible rather than
    // silently swallowed by a compiled-in filter.
    let raw_filter = std::env::var("RUST_LOG").ok();
    let env_filter = match raw_filter.as_deref() {
        Some(raw) => tracing_subscriber::EnvFilter::try_new(raw).map_err(|e| {
            format!("RUST_LOG={raw:?} is not a valid log filter: {e}")
        })?,
        None => {
            info!("RUST_LOG unset; using default log filter \"info,backend_rs=debug,tower_http=info\"");
            tracing_subscriber::EnvFilter::try_new("info,backend_rs=debug,tower_http=info")?
        }
    };
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    info!("Starting Communicator Rust Backend (Desktop + Web Unified Engine)...");

    let mut config = Config::from_env();
    let vault_secrets = services::vault_loader::load_secrets_from_nid_vault(&config).await;
    services::vault_loader::apply_vault_secrets(&mut config, &vault_secrets);
    let config = Arc::new(config);

    let db = init_db(&config.database_url).await?;
    let http_client = reqwest::Client::builder().build()?;

    let state = AppState {
        db: db.clone(),
        config: Arc::clone(&config),
        http_client,
        nid: Arc::new(services::nid_client::NidClient::from_config(&db, &config)),
    };

    // CORS configuration: restrict origins based on environment.
    // In development, allow common local origins. In production, require
    // PUBLIC_URL to be set and restrict to that origin.
    // Always allow the Cloudflare tunnel domain (communicator.newax.co).
    let tunnel_origin = "https://communicator.newax.co"
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("https://communicator.newax.co"));

    let cors = if config.app_env == "development" || config.public_url.is_empty() {
        CorsLayer::new()
            .allow_origin([
                "http://localhost:5173".parse::<HeaderValue>().unwrap_or_else(|_| HeaderValue::from_static("http://localhost:5173")),
                "http://localhost:3000".parse::<HeaderValue>().unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3000")),
                "http://localhost:8000".parse::<HeaderValue>().unwrap_or_else(|_| HeaderValue::from_static("http://localhost:8000")),
                tunnel_origin,
            ])
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let public_origin: HeaderValue = config.public_url.parse().unwrap_or_else(|_| {
            warn!("PUBLIC_URL={:?} is not a valid origin; falling back to tunnel origin", config.public_url);
            tunnel_origin.clone()
        });
        CorsLayer::new()
            .allow_origin([public_origin, tunnel_origin])
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Complete API Routes
    let api_routes = Router::new()
        // Health
        .route("/health", get(routes::health::health_check))

        // nID identity provider (Login with nID + Sign Up)
        .route("/auth/nid/start", get(routes::auth::login_with_nid_start))
        .route("/auth/nid/callback", get(routes::auth::login_with_nid_callback))
        .route("/auth/callback", get(routes::auth::login_with_nid_callback))
        .route("/auth/sign-up/start", post(routes::auth::sign_up_start))
        .route("/auth/sign-up/verify", post(routes::auth::sign_up_verify))
        .route("/auth/instagram/start", get(routes::auth::instagram_oauth_start))
        .route("/auth/instagram/callback", get(routes::auth::instagram_oauth_callback))
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
        .route("/messages/{message_id}/feedback", post(routes::conversations::submit_message_feedback))
        .route("/feedback/summary", get(routes::conversations::get_feedback_summary))

        // Contacts
        .route("/contacts", get(routes::contacts::list_contacts).post(routes::contacts::create_contact))
        .route("/contacts/import", post(routes::contacts::import_contacts))
        .route("/contacts/import/preview", post(routes::contacts::preview_import))
        .route("/contacts/tags", get(routes::contacts::list_contact_tags))
        .route("/contacts/export", get(routes::contacts::export_contacts))
        .route("/contacts/sync-meta", post(routes::contacts::sync_meta_contacts))
        .route("/contacts/{id}", get(routes::contacts::get_contact))
        .route("/contacts/{id}/link-customer", post(routes::contacts::link_customer))
        .route("/contacts/{id}/relationship-profile", get(routes::contacts::relationship_profile))
        .route("/contacts/{id}/tags", post(routes::contacts::add_contact_tag))
        .route("/contacts/{id}/tags/{tag}", delete(routes::contacts::remove_contact_tag))
        .route("/contacts/{id}/toggle-consent", post(routes::contacts::toggle_contact_consent))
        .route("/contacts/{id}/sync-meta", post(routes::contacts::sync_contact_meta))
        .route("/contacts/{id}/conversation", post(routes::contacts::get_or_create_contact_conversation))
        .route("/contacts/{id}/memories", get(routes::contacts::list_contact_memories).post(routes::contacts::create_contact_memory))
        .route("/contacts/{id}/memories/{memory_id}", delete(routes::contacts::delete_contact_memory))

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

        .route("/inboxes/channels/discord", get(routes::inboxes::get_discord_channel).post(routes::inboxes::update_discord_channel))
        .route("/inboxes/channels/discord/verify", post(routes::inboxes::verify_discord_connection))
        .route("/inboxes/channels/discord/test-message", post(routes::inboxes::send_discord_test_message))
        .route("/inboxes/channels/discord/events", get(routes::inboxes::list_discord_events))

        .route("/inboxes/channels/slack", get(routes::inboxes::get_slack_channel).post(routes::inboxes::update_slack_channel))
        .route("/inboxes/channels/slack/verify", post(routes::inboxes::verify_slack_connection))
        .route("/inboxes/channels/slack/test-message", post(routes::inboxes::send_slack_test_message))
        .route("/inboxes/channels/slack/events", get(routes::inboxes::list_slack_events))

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
        .route("/drafts/dispatch-auto-approved", post(routes::drafts::dispatch_auto_approved_drafts))

        // WhatsApp Broadcasts / Campaigns (legacy — use /broadcasts for omni-channel)
        .route("/whatsapp-broadcasts", get(routes::broadcasts::list_broadcasts).post(routes::broadcasts::create_broadcast))
        .route("/whatsapp-broadcasts/audience-preview", get(routes::broadcasts::get_audience_preview))
        .route("/whatsapp-broadcasts/{id}/analytics", get(routes::broadcasts::get_campaign_analytics))
        .route("/whatsapp-broadcasts/{id}/reschedule", post(routes::broadcasts::reschedule_campaign))
        .route("/whatsapp-broadcasts/{id}/dispatch", post(routes::broadcasts::dispatch_broadcast))
        .route("/whatsapp-broadcasts/process-scheduled", post(routes::broadcasts::process_scheduled_campaigns))
        .route("/whatsapp-broadcasts/{id}/recipients", get(routes::broadcasts::list_broadcast_recipients))
        .route("/whatsapp-broadcasts/pricing-calculator", get(routes::broadcasts::get_whatsapp_pricing))

        // Commerce Engine
        .route("/commerce/categories", get(routes::commerce::list_categories).post(routes::commerce::create_category))
        .route("/commerce/categories/{id}", get(routes::commerce::get_category).put(routes::commerce::update_category).delete(routes::commerce::delete_category))
        .route("/commerce/products", get(routes::commerce::list_products).post(routes::commerce::create_product))
        .route("/commerce/products/{id}", get(routes::commerce::get_product).put(routes::commerce::update_product).delete(routes::commerce::delete_product))
        .route("/commerce/products/{id}/inventory", put(routes::commerce::update_inventory))
        .route("/commerce/products/{id}/visibility", get(routes::commerce::get_channel_visibility).put(routes::commerce::set_channel_visibility))
        .route("/commerce/carts", get(routes::commerce::list_carts).post(routes::commerce::create_cart))
        .route("/commerce/carts/get-or-create", get(routes::commerce::get_or_create_cart))
        .route("/commerce/carts/{id}", get(routes::commerce::get_cart).delete(routes::commerce::delete_cart))
        .route("/commerce/carts/{id}/clear", post(routes::commerce::clear_cart))
        .route("/commerce/carts/{cartId}/items", post(routes::commerce::add_cart_item))
        .route("/commerce/carts/{cartId}/items/{itemId}", put(routes::commerce::update_cart_item).delete(routes::commerce::remove_cart_item))
        .route("/commerce/carts/{cartId}/checkout", post(routes::commerce::checkout_cart))

        // Commerce Orders
        .route("/commerce/orders", get(routes::commerce::list_orders))
        .route("/commerce/orders/{id}", get(routes::commerce::get_order))
        .route("/commerce/orders/{id}/update-status", post(routes::commerce::update_order_status))
        .route("/commerce/orders/stats", get(routes::commerce::get_order_stats))

        // Commerce Payments
        .route("/commerce/payments", get(routes::commerce::list_payments).post(routes::commerce::record_payment))
        .route("/commerce/payments/{id}", get(routes::commerce::get_payment).put(routes::commerce::update_payment_status))
        .route("/commerce/payments/{id}/refund", post(routes::commerce::refund_payment))
        .route("/commerce/orders/{orderId}/payments", get(routes::commerce::list_order_payments))
        .route("/commerce/payments/stats", get(routes::commerce::get_payment_stats))

        // Commerce Sync
        .route("/commerce/sync/connections", get(routes::commerce::list_sync_connections).post(routes::commerce::create_sync_connection))
        .route("/commerce/sync/connections/{id}", get(routes::commerce::get_sync_connection).put(routes::commerce::update_sync_connection).delete(routes::commerce::delete_sync_connection))
        .route("/commerce/sync/connections/{id}/sync", post(routes::commerce::trigger_sync))
        .route("/commerce/sync/connections/{id}/info", get(routes::commerce::get_platform_info))
        .route("/commerce/sync/connections/{id}/mappings", get(routes::commerce::list_product_mappings))
        .route("/commerce/sync/logs", get(routes::commerce::list_sync_logs))
        .route("/commerce/sync/all", post(routes::commerce::sync_all_platforms))
        .route("/commerce/sync/queue/process", post(routes::commerce::process_sync_queue))

        // Commerce Webhooks — receive real-time events from external platforms
        .route("/commerce/webhooks/shopify", post(routes::commerce::webhook_shopify))
        .route("/commerce/webhooks/woocommerce", post(routes::commerce::webhook_woocommerce))

        // Commerce Phase 2: Coupons
        .route("/commerce/coupons", get(routes::commerce::list_coupons).post(routes::commerce::create_coupon))
        .route("/commerce/coupons/{id}", get(routes::commerce::get_coupon).put(routes::commerce::update_coupon).delete(routes::commerce::delete_coupon))
        .route("/commerce/coupons/validate", post(routes::commerce::validate_coupon))
        .route("/commerce/coupons/{id}/usage", get(routes::commerce::coupon_usage_history))

        // Commerce Phase 2: Memberships
        .route("/commerce/memberships", get(routes::commerce::list_membership_tiers).post(routes::commerce::create_membership_tier))
        .route("/commerce/memberships/{id}", get(routes::commerce::get_membership_tier).put(routes::commerce::update_membership_tier).delete(routes::commerce::delete_membership_tier))
        .route("/commerce/memberships/assign", post(routes::commerce::assign_membership))
        .route("/commerce/memberships/customer", get(routes::commerce::list_customer_memberships))

        // Commerce Phase 2: Analytics
        .route("/commerce/analytics/overview", get(routes::commerce::analytics_overview))
        .route("/commerce/analytics/revenue", get(routes::commerce::analytics_revenue_trend))
        .route("/commerce/analytics/top-products", get(routes::commerce::analytics_top_products))
        .route("/commerce/analytics/top-customers", get(routes::commerce::analytics_top_customers))
        .route("/commerce/analytics/channels", get(routes::commerce::analytics_channels))
        .route("/commerce/analytics/daily-stats", get(routes::commerce::analytics_daily_stats))
        .route("/commerce/analytics/update-stats", post(routes::commerce::analytics_update_daily_stats))
        .route("/commerce/analytics/events", post(routes::commerce::record_analytics_event))

        // Commerce Phase 2: Shipping
        .route("/commerce/shipping-methods", get(routes::commerce::list_shipping_methods).post(routes::commerce::create_shipping_method))
        .route("/commerce/shipping-methods/{id}", get(routes::commerce::get_shipping_method).put(routes::commerce::update_shipping_method).delete(routes::commerce::delete_shipping_method))
        .route("/commerce/shipments", get(routes::commerce::list_shipments).post(routes::commerce::create_shipment))
        .route("/commerce/shipments/{id}", get(routes::commerce::get_shipment).put(routes::commerce::update_shipment))
        .route("/commerce/shipments/{shipmentId}/events", get(routes::commerce::list_delivery_events).post(routes::commerce::add_delivery_event))

        // Commerce Phase 2: Storefront
        .route("/commerce/storefronts", get(routes::commerce::list_storefronts).post(routes::commerce::create_storefront))
        .route("/commerce/storefronts/{id}", get(routes::commerce::get_storefront).put(routes::commerce::update_storefront).delete(routes::commerce::delete_storefront))
        .route("/commerce/storefronts/{storefrontId}/sections", get(routes::commerce::list_storefront_sections).post(routes::commerce::create_storefront_section))
        .route("/commerce/storefronts/sections/{id}", put(routes::commerce::update_storefront_section).delete(routes::commerce::delete_storefront_section))
        .route("/storefront/{slug}", get(routes::commerce::get_public_storefront))

        // Commerce Phase 2: Inventory
        .route("/commerce/warehouses", get(routes::commerce::list_warehouses).post(routes::commerce::create_warehouse))
        .route("/commerce/warehouses/{id}", get(routes::commerce::get_warehouse).put(routes::commerce::update_warehouse).delete(routes::commerce::delete_warehouse))
        .route("/commerce/inventory/{productId}", get(routes::commerce::get_inventory))
        .route("/commerce/inventory/{productId}/summary", get(routes::commerce::get_inventory_summary))
        .route("/commerce/inventory", put(routes::commerce::set_inventory))
        .route("/commerce/inventory/transfer", post(routes::commerce::transfer_inventory))
        .route("/commerce/inventory/alerts", get(routes::commerce::list_low_stock_alerts))
        .route("/commerce/inventory/alerts/{id}/acknowledge", post(routes::commerce::acknowledge_low_stock_alert))
        .route("/commerce/inventory/back-in-stock", get(routes::commerce::list_back_in_stock_requests).post(routes::commerce::create_back_in_stock_request))
        .route("/commerce/inventory/{productId}/transactions", get(routes::commerce::transaction_history))

        // Commerce Gaps: Product Options
        .route("/commerce/products/{productId}/options", get(routes::commerce::list_product_options).post(routes::commerce::create_product_option))
        .route("/commerce/options/{id}", get(routes::commerce::get_product_option).put(routes::commerce::update_product_option).delete(routes::commerce::delete_product_option))
        .route("/commerce/options/{optionId}/values", post(routes::commerce::create_product_option_value))
        .route("/commerce/options/values/{id}", delete(routes::commerce::delete_product_option_value))

        // Commerce Gaps: Delivery Fee Rules
        .route("/commerce/delivery-fee-rules", get(routes::commerce::list_delivery_fee_rules).post(routes::commerce::create_delivery_fee_rule))
        .route("/commerce/delivery-fee-rules/{id}", put(routes::commerce::update_delivery_fee_rule).delete(routes::commerce::delete_delivery_fee_rule))

        // Commerce Gaps: Payment Proofs
        .route("/commerce/payment-proofs", get(routes::commerce::list_payment_proofs).post(routes::commerce::upload_payment_proof))
        .route("/commerce/payment-proofs/{id}/verify", post(routes::commerce::verify_payment_proof))

        // Commerce Gaps: Order Notification Templates
        .route("/commerce/notification-templates", get(routes::commerce::list_notification_templates).post(routes::commerce::create_notification_template))
        .route("/commerce/notification-templates/{id}", delete(routes::commerce::delete_notification_template))

        // Commerce Gaps: Payment Gateways
        .route("/commerce/payment-gateways", get(routes::commerce::list_payment_gateways).post(routes::commerce::create_payment_gateway))
        .route("/commerce/payment-gateways/{id}", delete(routes::commerce::delete_payment_gateway))

        // Commerce Gaps: Service Types
        .route("/commerce/service-types", get(routes::commerce::list_service_types).post(routes::commerce::create_service_type))
        .route("/commerce/service-types/{id}", delete(routes::commerce::delete_service_type))

        // Commerce Gaps: Delivery Schedules & Time Slots
        .route("/commerce/delivery-schedules", get(routes::commerce::list_delivery_schedules).post(routes::commerce::create_delivery_schedule))
        .route("/commerce/delivery-schedules/{id}", delete(routes::commerce::delete_delivery_schedule))
        .route("/commerce/time-slots", get(routes::commerce::list_time_slots).post(routes::commerce::create_time_slot))
        .route("/commerce/time-slots/{id}", delete(routes::commerce::delete_time_slot))

        // Commerce Gaps: Invoices
        .route("/commerce/orders/{orderId}/invoice", post(routes::commerce::generate_invoice))

        // Commerce Gaps: CSV Import
        .route("/commerce/csv/import", post(routes::commerce::import_csv))
        .route("/commerce/csv/jobs", get(routes::commerce::list_csv_import_jobs))

        // Commerce Gaps: Customer Accounts
        .route("/commerce/customers/{customerId}/account", post(routes::commerce::create_customer_account))
        .route("/commerce/customer/login", post(routes::commerce::customer_login))

        // Commerce Gaps: Customer Reviews
        .route("/commerce/reviews", get(routes::commerce::list_reviews).post(routes::commerce::create_review))

        // Commerce Gaps: Loyalty Programs
        .route("/commerce/loyalty", get(routes::commerce::list_loyalty_programs).post(routes::commerce::create_loyalty_program))
        .route("/commerce/loyalty/points", get(routes::commerce::get_customer_points))

        // Commerce Gaps: Staff Management
        .route("/commerce/staff", get(routes::commerce::list_staff_members).post(routes::commerce::create_staff_member))
        .route("/commerce/staff/{id}", delete(routes::commerce::delete_staff_member))

        // Commerce Gaps: POS Sessions
        .route("/commerce/pos/sessions", get(routes::commerce::list_pos_sessions).post(routes::commerce::create_pos_session))
        .route("/commerce/pos/sessions/{id}/end", post(routes::commerce::end_pos_session))

        // Commerce Gaps: Receipts
        .route("/commerce/orders/{orderId}/receipt", post(routes::commerce::generate_receipt))

        // Commerce Gaps: Wholesale Pricing
        .route("/commerce/products/{productId}/wholesale", get(routes::commerce::list_wholesale_pricing).post(routes::commerce::create_wholesale_pricing))
        .route("/commerce/wholesale/{id}", delete(routes::commerce::delete_wholesale_pricing))

        // Commerce Gaps: Automated Follow-ups
        .route("/commerce/followups", get(routes::commerce::list_followups).post(routes::commerce::create_followup))
        .route("/commerce/followups/{id}", delete(routes::commerce::delete_followup))

        // Commerce Gaps: Custom Domains
        .route("/commerce/custom-domains", get(routes::commerce::list_custom_domains).post(routes::commerce::create_custom_domain))
        .route("/commerce/custom-domains/{id}", delete(routes::commerce::delete_custom_domain))
        .route("/commerce/custom-domains/{id}/verify", post(routes::commerce::verify_custom_domain))

        // Commerce Gaps: Storefront Pages
        .route("/commerce/storefronts/{storefrontId}/pages", get(routes::commerce::list_storefront_pages).post(routes::commerce::create_storefront_page))
        .route("/commerce/storefronts/pages/{id}", delete(routes::commerce::delete_storefront_page))

        // Commerce Gaps: Meta Pixels
        .route("/commerce/meta-pixels", get(routes::commerce::list_meta_pixels).post(routes::commerce::create_meta_pixel))
        .route("/commerce/meta-pixels/{id}", delete(routes::commerce::delete_meta_pixel))

        // Commerce Gaps: Checkout Questions
        .route("/commerce/checkout-questions", get(routes::commerce::list_checkout_questions).post(routes::commerce::create_checkout_question))
        .route("/commerce/checkout-questions/{id}", delete(routes::commerce::delete_checkout_question))
        .route("/commerce/checkout/answers", post(routes::commerce::submit_checkout_answers))

        // Commerce Gaps: Delivery Proofs
        .route("/commerce/delivery-proofs", post(routes::commerce::upload_delivery_proof))
        .route("/commerce/orders/{orderId}/delivery-proofs", get(routes::commerce::list_delivery_proofs))

        // ERP Catalog: Categories
        .route("/erp/catalog/categories", get(routes::erp_catalog::list_categories).post(routes::erp_catalog::create_category))
        .route("/erp/catalog/categories/{id}", get(routes::erp_catalog::get_category).put(routes::erp_catalog::update_category).delete(routes::erp_catalog::delete_category))

        // ERP Catalog: Products
        .route("/erp/catalog/products", get(routes::erp_catalog::list_products).post(routes::erp_catalog::create_product))
        .route("/erp/catalog/products/{id}", get(routes::erp_catalog::get_product).put(routes::erp_catalog::update_product).delete(routes::erp_catalog::delete_product))
        .route("/erp/catalog/products/slug/{slug}", get(routes::erp_catalog::get_product_by_slug))

        // ERP Catalog: Product Translations
        .route("/erp/catalog/products/{productId}/translations", get(routes::erp_catalog::list_product_translations))
        .route("/erp/catalog/products/{productId}/translations/{locale}", post(routes::erp_catalog::upsert_product_translation))

        // ERP Catalog: Variants
        .route("/erp/catalog/products/{productId}/variants", get(routes::erp_catalog::list_variants).post(routes::erp_catalog::create_variant))
        .route("/erp/catalog/variants/{id}", get(routes::erp_catalog::get_variant).put(routes::erp_catalog::update_variant).delete(routes::erp_catalog::delete_variant))

        // ERP Catalog: Images
        .route("/erp/catalog/products/{productId}/images", get(routes::erp_catalog::list_images).post(routes::erp_catalog::create_image))
        .route("/erp/catalog/images/{id}", delete(routes::erp_catalog::delete_image))

        // ERP Catalog: Options
        .route("/erp/catalog/products/{productId}/options", get(routes::erp_catalog::list_product_options).post(routes::erp_catalog::create_product_option))
        .route("/erp/catalog/options/{id}", delete(routes::erp_catalog::delete_product_option))
        .route("/erp/catalog/options/{optionId}/values", get(routes::erp_catalog::list_option_values).post(routes::erp_catalog::create_option_value))
        .route("/erp/catalog/option-values/{id}", delete(routes::erp_catalog::delete_option_value))

        // ERP Catalog: Collections
        .route("/erp/catalog/collections", get(routes::erp_catalog::list_collections).post(routes::erp_catalog::create_collection))
        .route("/erp/catalog/collections/{id}", get(routes::erp_catalog::get_collection))
        .route("/erp/catalog/collections/{collectionId}/products", post(routes::erp_catalog::add_product_to_collection).get(routes::erp_catalog::list_collection_products))
        .route("/erp/catalog/collections/{collectionId}/products/{productId}", delete(routes::erp_catalog::remove_product_from_collection))

        // ERP Catalog: Stock Adjustments
        .route("/erp/catalog/stock/adjust", post(routes::erp_catalog::adjust_stock))
        .route("/erp/catalog/variants/{variantId}/adjustments", get(routes::erp_catalog::list_stock_adjustments))

        // ERP Catalog: Cross-sells
        .route("/erp/catalog/products/{sourceId}/cross-sells", post(routes::erp_catalog::add_cross_sell).get(routes::erp_catalog::list_cross_sells))

        // ERP Catalog: AI Suggestions
        .route("/erp/catalog/products/{productId}/ai-suggestions", get(routes::erp_catalog::list_ai_suggestions))
        .route("/erp/catalog/ai-suggestions/{id}/approve", post(routes::erp_catalog::approve_ai_suggestion))
        .route("/erp/catalog/ai-suggestions/{id}/reject", post(routes::erp_catalog::reject_ai_suggestion))

        // ERP Catalog: Redirects
        .route("/erp/catalog/redirects", get(routes::erp_catalog::list_redirects).post(routes::erp_catalog::create_redirect))

        // ERP Orders
        .route("/erp/orders", get(routes::erp_orders::list_orders).post(routes::erp_orders::create_order))
        .route("/erp/orders/stats", get(routes::erp_orders::get_order_stats))
        .route("/erp/orders/{id}", get(routes::erp_orders::get_order))
        .route("/erp/orders/{id}/items", get(routes::erp_orders::get_order_items))
        .route("/erp/orders/{id}/events", get(routes::erp_orders::get_order_status_events))
        .route("/erp/orders/{id}/status", post(routes::erp_orders::update_order_status))
        .route("/erp/orders/{id}/cancel", post(routes::erp_orders::cancel_order))
        .route("/erp/orders/number/{number}", get(routes::erp_orders::get_order_by_number))

        // ERP Carts
        .route("/erp/carts", get(routes::erp_orders::list_carts).post(routes::erp_orders::create_cart))
        .route("/erp/carts/{id}", get(routes::erp_orders::get_cart).delete(routes::erp_orders::delete_cart))
        .route("/erp/carts/{id}/items", get(routes::erp_orders::get_cart_items).post(routes::erp_orders::add_cart_item))
        .route("/erp/carts/items/{itemId}", put(routes::erp_orders::update_cart_item).delete(routes::erp_orders::remove_cart_item))
        .route("/erp/carts/checkout", post(routes::erp_orders::checkout_cart))

        // ERP Shipments
        .route("/erp/shipments", get(routes::erp_orders::list_shipments).post(routes::erp_orders::create_shipment))
        .route("/erp/shipments/{id}", get(routes::erp_orders::get_shipment))
        .route("/erp/shipments/{id}/status", post(routes::erp_orders::update_shipment_status))
        .route("/erp/shipments/{id}/events", get(routes::erp_orders::get_shipment_events).post(routes::erp_orders::create_shipment_event))

        // ERP Back in Stock
        .route("/erp/back-in-stock", get(routes::erp_orders::list_back_in_stock_requests).post(routes::erp_orders::create_back_in_stock_request))
        .route("/erp/back-in-stock/{productId}/notify", post(routes::erp_orders::notify_back_in_stock))

        // ERP Payments
        .route("/erp/payments", get(routes::erp_payments::list_payments).post(routes::erp_payments::create_payment))
        .route("/erp/payments/stats", get(routes::erp_payments::get_payment_stats))
        .route("/erp/payments/approvals", get(routes::erp_payments::get_pending_approvals))
        .route("/erp/payments/{id}", get(routes::erp_payments::get_payment))
        .route("/erp/payments/{id}/status", post(routes::erp_payments::update_payment_status))
        .route("/erp/payments/{id}/mark-paid", post(routes::erp_payments::mark_as_paid))
        .route("/erp/payments/{id}/mark-failed", post(routes::erp_payments::mark_as_failed))
        .route("/erp/payments/{id}/refund", post(routes::erp_payments::refund_payment))
        .route("/erp/payments/{id}/receipt", post(routes::erp_payments::upload_receipt))
        .route("/erp/payments/{id}/approve", post(routes::erp_payments::approve_payment))
        .route("/erp/payments/{id}/reject", post(routes::erp_payments::reject_payment))
        .route("/erp/payments/order/{orderId}", get(routes::erp_payments::get_payments_by_order))
        .route("/erp/payments/ref/{reference}", get(routes::erp_payments::get_payment_by_reference))
        .route("/erp/payment-methods", get(routes::erp_payments::list_payment_methods).post(routes::erp_payments::create_payment_method))
        .route("/erp/payment-methods/{id}", get(routes::erp_payments::get_payment_method).put(routes::erp_payments::update_payment_method).delete(routes::erp_payments::delete_payment_method))
        .route("/erp/payment-methods/code/{code}", get(routes::erp_payments::get_payment_method_by_code))

        // ERP Inventory
        .route("/erp/inventory/locations", get(routes::erp_inventory::list_locations).post(routes::erp_inventory::create_location))
        .route("/erp/inventory/locations/{id}", get(routes::erp_inventory::get_location).put(routes::erp_inventory::update_location))
        .route("/erp/inventory/stock/location/{location_id}", get(routes::erp_inventory::get_stock_by_location))
        .route("/erp/inventory/stock/variant/{variant_id}", get(routes::erp_inventory::get_stock_by_variant))
        .route("/erp/inventory/stock/{variant_id}/{location_id}", put(routes::erp_inventory::update_variant_stock))
        .route("/erp/inventory/stock/{variant_id}/total", get(routes::erp_inventory::get_total_stock))
        .route("/erp/inventory/movements", get(routes::erp_inventory::list_stock_movements).post(routes::erp_inventory::create_stock_movement))
        .route("/erp/inventory/movements/{id}", get(routes::erp_inventory::get_stock_movement))
        .route("/erp/inventory/movements/variant/{variant_id}", get(routes::erp_inventory::get_movements_by_variant))
        .route("/erp/inventory/variances", get(routes::erp_inventory::list_stock_variances).post(routes::erp_inventory::create_stock_variance))
        .route("/erp/inventory/variances/{id}", get(routes::erp_inventory::get_stock_variance))
        .route("/erp/inventory/variances/{id}/resolve", put(routes::erp_inventory::resolve_variance))
        .route("/erp/inventory/spot-check-policies", get(routes::erp_inventory::list_spot_check_policies).post(routes::erp_inventory::create_spot_check_policy))
        .route("/erp/inventory/spot-checks", get(routes::erp_inventory::list_spot_checks).post(routes::erp_inventory::create_spot_check))
        .route("/erp/inventory/spot-checks/{id}", get(routes::erp_inventory::get_spot_check))
        .route("/erp/inventory/spot-checks/{id}/start", put(routes::erp_inventory::start_spot_check))
        .route("/erp/inventory/spot-checks/{id}/complete", put(routes::erp_inventory::complete_spot_check))
        .route("/erp/inventory/spot-check-lines/{spot_check_id}", get(routes::erp_inventory::list_spot_check_lines))
        .route("/erp/inventory/spot-check-lines", post(routes::erp_inventory::create_spot_check_line))
        .route("/erp/inventory/spot-check-lines/{id}", put(routes::erp_inventory::update_spot_check_line))
        .route("/erp/inventory/sales-stats/{variant_id}", get(routes::erp_inventory::get_variant_sales_stat))
        .route("/erp/inventory/slow-movers", get(routes::erp_inventory::list_slow_movers))
        .route("/erp/inventory/forecasts/{variant_id}", get(routes::erp_inventory::get_variant_forecast))
        .route("/erp/inventory/forecasts", post(routes::erp_inventory::create_variant_forecast).get(routes::erp_inventory::list_forecasts))
        .route("/erp/inventory/markdown-suggestions", get(routes::erp_inventory::list_markdown_suggestions).post(routes::erp_inventory::create_markdown_suggestion))
        .route("/erp/inventory/markdown-suggestions/{id}/status", put(routes::erp_inventory::update_markdown_suggestion_status))
        .route("/erp/inventory/lots", get(routes::erp_inventory::list_lots).post(routes::erp_inventory::create_lot))
        .route("/erp/inventory/lots/{id}", get(routes::erp_inventory::get_lot))
        .route("/erp/inventory/lots/expiring", get(routes::erp_inventory::get_expiring_lots))
        .route("/erp/inventory/lot-stock/{lot_id}", get(routes::erp_inventory::list_lot_stock))
        .route("/erp/inventory/lot-stock", post(routes::erp_inventory::create_lot_stock))
        .route("/erp/inventory/lot-allocations/{lot_id}", get(routes::erp_inventory::list_lot_allocations))
        .route("/erp/inventory/lot-allocations", post(routes::erp_inventory::create_lot_allocation))
        .route("/erp/inventory/serialized-units", get(routes::erp_inventory::list_serialized_units).post(routes::erp_inventory::create_serialized_unit))
        .route("/erp/inventory/serialized-units/{id}", get(routes::erp_inventory::get_serialized_unit))
        .route("/erp/inventory/serialized-units/serial/{serial_number}", get(routes::erp_inventory::get_serialized_unit_by_serial))
        .route("/erp/inventory/serialized-units/{id}/status", put(routes::erp_inventory::update_serialized_unit_status))
        .route("/erp/inventory/stock-counts", get(routes::erp_inventory::list_stock_counts).post(routes::erp_inventory::create_stock_count))
        .route("/erp/inventory/stock-counts/{id}", get(routes::erp_inventory::get_stock_count))
        .route("/erp/inventory/stock-counts/{id}/start", put(routes::erp_inventory::start_stock_count))
        .route("/erp/inventory/stock-counts/{id}/complete", put(routes::erp_inventory::complete_stock_count))
        .route("/erp/inventory/stock-counts/{id}/cancel", put(routes::erp_inventory::cancel_stock_count))
        .route("/erp/inventory/stock-count-lines/{stock_count_id}", get(routes::erp_inventory::list_stock_count_lines))
        .route("/erp/inventory/stock-count-lines", post(routes::erp_inventory::create_stock_count_line))
        .route("/erp/inventory/stock-count-lines/{id}", put(routes::erp_inventory::update_stock_count_line))
        .route("/erp/inventory/stats", get(routes::erp_inventory::get_inventory_stats))

        // ERP Fulfilment: Summary
        .route("/erp/fulfilment/summary", get(routes::erp_fulfilment::get_summary))
        // ERP Fulfilment: Shipping Zones
        .route("/erp/fulfilment/shipping-zones", get(routes::erp_fulfilment::list_shipping_zones).post(routes::erp_fulfilment::create_shipping_zone))
        .route("/erp/fulfilment/shipping-zones/{id}", get(routes::erp_fulfilment::get_shipping_zone).put(routes::erp_fulfilment::update_shipping_zone).delete(routes::erp_fulfilment::delete_shipping_zone))
        // ERP Fulfilment: Shipping Methods
        .route("/erp/fulfilment/shipping-methods", get(routes::erp_fulfilment::list_shipping_methods).post(routes::erp_fulfilment::create_shipping_method))
        .route("/erp/fulfilment/shipping-methods/{id}", get(routes::erp_fulfilment::get_shipping_method).put(routes::erp_fulfilment::update_shipping_method).delete(routes::erp_fulfilment::delete_shipping_method))
        // ERP Fulfilment: Shipping Rates
        .route("/erp/fulfilment/shipping-rates", get(routes::erp_fulfilment::list_shipping_rates).post(routes::erp_fulfilment::create_shipping_rate))
        .route("/erp/fulfilment/shipping-rates/{id}", get(routes::erp_fulfilment::get_shipping_rate).put(routes::erp_fulfilment::update_shipping_rate).delete(routes::erp_fulfilment::delete_shipping_rate))
        // ERP Fulfilment: Return Reasons
        .route("/erp/fulfilment/return-reasons", get(routes::erp_fulfilment::list_return_reasons))
        // ERP Fulfilment: Return Requests (RMA)
        .route("/erp/fulfilment/returns", get(routes::erp_fulfilment::list_return_requests).post(routes::erp_fulfilment::create_return_request))
        .route("/erp/fulfilment/returns/{id}", get(routes::erp_fulfilment::get_return_request))
        .route("/erp/fulfilment/returns/{id}/status", put(routes::erp_fulfilment::update_return_status))
        .route("/erp/fulfilment/returns/inspect", put(routes::erp_fulfilment::inspect_return_item))
        .route("/erp/fulfilment/returns/stats", get(routes::erp_fulfilment::get_return_stats))
        // ERP Fulfilment: Manifests
        .route("/erp/fulfilment/manifests", get(routes::erp_fulfilment::list_manifests))
        .route("/erp/fulfilment/manifests/{id}/dispatch", post(routes::erp_fulfilment::dispatch_manifest))

        // Services & Sequences
        .route("/services", get(routes::services_sequences::list_services).post(routes::services_sequences::create_service))
        .route("/services/{id}", get(routes::services_sequences::get_service))
        .route("/sequences", get(routes::services_sequences::list_sequences).post(routes::services_sequences::create_sequence))
        .route("/sequences/{id}/variant-stats", get(routes::services_sequences::sequence_variant_stats))

        // Team & Routing
        .route("/team", get(routes::team_routing::list_team).post(routes::team_routing::create_team_member))
        .route("/team/{id}", delete(routes::team_routing::delete_team_member))
        .route("/team/conversations/{id}/assign", post(routes::team_routing::assign_conversation_team))
        .route("/team/conversations/{id}/takeover", post(routes::team_routing::set_human_takeover))
        .route("/teams", get(routes::team_routing::list_teams).post(routes::team_routing::create_team))
        .route("/teams/{id}", delete(routes::team_routing::delete_team))
        .route("/teams/{id}/members", post(routes::team_routing::add_member_to_team))
        .route(
            "/teams/{id}/members/{member_id}",
            delete(routes::team_routing::remove_member_from_team),
        )
        .route("/routing/agents", get(routes::team_routing::list_routing_agents))
        .route("/routing/agents/{id}/status", put(routes::team_routing::update_agent_status))
        .route("/routing/agents/{id}/capacity", put(routes::team_routing::update_agent_capacity))
        .route("/routing/capacity-overview", get(routes::team_routing::get_capacity_overview))
        .route("/routing/auto-assign-batch", post(routes::team_routing::batch_auto_assign))

        // Knowledge Base
        .route("/knowledge-base", get(routes::knowledge_base::list_knowledge_base).post(routes::knowledge_base::create_knowledge_base_entry))
        .route("/knowledge-base/{id}", delete(routes::knowledge_base::delete_knowledge_base_entry))
        .route("/knowledge-base/ingest-url", post(routes::knowledge_base::ingest_url))
        .route("/knowledge-base/upload", post(routes::knowledge_base::upload_knowledge_document))
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
        .route("/automation-rules/{id}", put(routes::automation::update_automation_rule).delete(routes::automation::delete_automation_rule))
        .route("/automation-rules/test", post(routes::automation::test_automation_rule))
        .route("/automation-rules/stats", get(routes::automation::get_automation_stats))
        .route("/automation-rules/run-nudges", post(routes::automation::run_inactivity_nudges))
        .route("/automation-rules/apply", post(routes::automation::apply_automation_rules))

        // Canned Responses & Macros & SLA
        .route("/canned-responses", get(routes::canned_macros_sla::list_canned_responses).post(routes::canned_macros_sla::create_canned_response))
        .route("/canned-responses/{id}", put(routes::canned_macros_sla::update_canned_response).delete(routes::canned_macros_sla::delete_canned_response))
        .route("/canned-responses/expand", post(routes::canned_macros_sla::expand_canned_response))
        .route("/macros", get(routes::canned_macros_sla::list_macros).post(routes::canned_macros_sla::create_macro))
        .route("/macros/{id}", get(routes::canned_macros_sla::get_macro).put(routes::canned_macros_sla::update_macro).delete(routes::canned_macros_sla::delete_macro))
        .route("/conversations/{id}/macros/{macro_id}/execute", post(routes::canned_macros_sla::execute_macro))
        .route("/macros/{macro_id}/execute", post(routes::canned_macros_sla::execute_macro_direct))
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
        .route("/integrations/trackers/configs", get(routes::integrations_parity::list_tracker_configs))
        .route(
            "/integrations/trackers/configs/{provider}",
            get(routes::integrations_parity::get_tracker_config)
                .post(routes::integrations_parity::update_tracker_config)
                .put(routes::integrations_parity::update_tracker_config),
        )
        .route("/integrations/trackers/test/{provider}", post(routes::integrations_parity::test_tracker_connection))
        .route("/integrations/trackers/linked", get(routes::tickets::list_linked_issues))
        .route("/integrations/trackers/create-issue", post(routes::integrations_parity::create_linked_issue))
        .route("/integrations/trackers/linked/{id}", delete(routes::tickets::unlink_issue))
        .route("/integrations/trackers/linked/{id}/sync", post(routes::integrations_parity::sync_issue_status))
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
        .route("/ai/connections/{id}", delete(routes::copilot_ai::delete_ai_connection))
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
        .route("/gateways/{id}", get(routes::gateways_webhooks_stripe::get_single_gateway).put(routes::gateways_webhooks_stripe::update_gateway).delete(routes::gateways_webhooks_stripe::delete_gateway))
        .route("/gateways/{id}/test", post(routes::gateways_webhooks_stripe::test_gateway_connection))
        .route("/gateways/{id}/set-default", post(routes::gateways_webhooks_stripe::set_default_gateway))
        .route("/gateways/{id}/ping", post(routes::gateways_webhooks_stripe::send_gateway_ping))
        .route("/outgoing-webhooks", get(routes::gateways_webhooks_stripe::list_webhook_endpoints).post(routes::gateways_webhooks_stripe::create_webhook_endpoint))
        .route("/outgoing-webhooks/events/catalog", get(routes::gateways_webhooks_stripe::get_webhook_catalog))
        .route("/outgoing-webhooks/deliveries", get(routes::gateways_webhooks_stripe::get_webhook_deliveries))
        .route("/outgoing-webhooks/{id}/deliveries", get(routes::gateways_webhooks_stripe::get_endpoint_deliveries))
        .route("/outgoing-webhooks/{id}", get(routes::gateways_webhooks_stripe::get_single_webhook_endpoint).put(routes::gateways_webhooks_stripe::update_webhook_endpoint).delete(routes::gateways_webhooks_stripe::delete_webhook_endpoint))
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
        .route("/widget/preview-page", get(routes::admin_system::get_widget_preview_page))
        .route("/audit-logs", get(routes::admin_system::get_audit_logs).post(routes::admin_system::record_audit_event))
        .route("/audit-logs/stats", get(routes::admin_system::get_audit_stats))
        .route("/audit-logs/export", get(routes::admin_system::export_audit_logs))
        .route("/roles", get(routes::admin_system::list_roles).post(routes::admin_system::create_role))
        .route("/roles/permissions", get(routes::admin_system::get_permissions_catalog))
        .route("/roles/{id}", get(routes::admin_system::get_role_detail).put(routes::admin_system::update_role).delete(routes::admin_system::delete_role))
        .route("/roles/members/{id}/assign", post(routes::admin_system::assign_member_role))
        .route("/roles/members/{id}/check", get(routes::admin_system::check_member_permission))
        .route("/sso/config", get(routes::sso::get_config).put(routes::sso::update_config))
        .route("/sso/test", post(routes::sso::test_connection))
        .route("/sso/check", post(routes::sso::check_email))
        .route("/sso/login", get(routes::sso::initiate_login))
        .route("/sso/metadata", get(routes::sso::sp_metadata))
        .route("/sso/metadata.xml", get(routes::sso::sp_metadata))
        .route("/sso/saml/acs", post(routes::sso::assertion_consumer_service))
        .route("/custom-attributes", get(routes::admin_system::get_custom_attributes).post(routes::admin_system::create_custom_attribute))
        .route("/custom-attributes/{id}", get(routes::admin_system::get_custom_attribute).put(routes::admin_system::update_custom_attribute).delete(routes::admin_system::delete_custom_attribute))
        .route("/custom-attributes/contacts/{id}/values", post(routes::admin_system::update_contact_attribute_values))
        .route("/custom-attributes/conversations/{id}/values", post(routes::admin_system::update_conv_attribute_values))
        .route("/organizations", get(routes::admin_system::list_organizations).post(routes::admin_system::create_organization))
        .route("/segments", get(routes::admin_system::list_segments).post(routes::admin_system::create_segment))
        .route("/segments/{id}", get(routes::admin_system::get_segment).put(routes::admin_system::update_segment).delete(routes::admin_system::delete_segment))
        .route("/segments/{id}/export", get(routes::admin_system::export_segment))
        .route("/scheduler/run-followups", post(routes::scheduler::run_followups))
        .route("/scheduler/dispatch-auto-approved", post(routes::scheduler::dispatch_auto_approved))
        .route("/scheduler/run-nudges", post(routes::scheduler::run_nudges))
        .route("/scheduler/process-campaigns", post(routes::scheduler::process_campaigns))
        .route("/scheduler/run-all", post(routes::scheduler::run_all_schedulers))
        .route("/scheduler/jobs", get(routes::scheduler::list_jobs))
        .route("/scheduler/status", get(routes::scheduler::scheduler_status))
        .route("/scheduler/jobs/{key}", get(routes::scheduler::get_job).put(routes::scheduler::update_job))
        .route("/settings", get(routes::integrations_parity::get_integration_settings).put(routes::integrations_parity::update_integration_settings))
        .route("/settings/whatsapp/verify", post(routes::integrations_parity::verify_whatsapp_settings))
        .route("/settings/whatsapp/templates", get(routes::integrations_parity::list_whatsapp_templates).post(routes::integrations_parity::create_whatsapp_template))
        .route("/settings/whatsapp/templates/{name}", delete(routes::integrations_parity::delete_whatsapp_template))

        // Organization Profile & BYOK
        .route("/organization/profile", get(routes::organization::get_profile).put(routes::organization::update_profile))
        .route("/organization/settings", get(routes::organization::get_settings).put(routes::organization::update_settings))
        .route("/organization/stats", get(routes::organization::get_stats))
        .route("/organization/channels/{channel}/verify", post(routes::organization::verify_channel))

        // Brands, Organizations, BYOK, AI Config, Knowledge Base (Brand-level)
        .merge(routes::brands::router())

        // Client Profiles - Unified Customer IDs, Self-Learning, Transactions, RAG
        .merge(routes::client_profiles::router())

        // Discovery funnel
        .route("/discovery/templates", get(routes::discovery::list_discovery_templates))
        .route("/discovery/inquiries", get(routes::discovery::list_inquiries))
        .route("/discovery/inquiries/{id}", get(routes::discovery::get_inquiry))
        .route("/discovery/campaign-templates", get(routes::discovery::list_campaign_templates))
        .route("/discovery/campaign-templates/sync-status", post(routes::discovery::sync_campaign_template_status))
        .route("/reports/live", get(routes::admin_system::get_live_report))
        .route("/reports/conversations", get(routes::admin_system::get_conversation_reports))
        .route("/reports/agents", get(routes::admin_system::get_agent_reports))
        .route("/reports/teams", get(routes::admin_system::get_team_reports))
        .route("/reports/csat", get(routes::admin_system::get_csat_report))

        // Webhooks -- inbound from channel providers. Paths must match the
        // callback URLs already configured in those provider dashboards.
        .route(
            "/webhooks/meta/whatsapp",
            get(routes::webhooks::meta_whatsapp_verify)
                .post(routes::webhooks::meta_whatsapp_inbound),
        )
        .route(
            "/webhooks/whatsapp",
            get(routes::webhooks::meta_whatsapp_verify)
                .post(routes::webhooks::meta_whatsapp_inbound),
        )
        .route("/webhooks/twilio/sms", post(routes::webhooks::twilio_sms_inbound))
        .route("/webhooks/twilio/whatsapp", post(routes::webhooks::twilio_whatsapp_inbound))
        .route(
            "/webhooks/meta/facebook",
            get(routes::webhooks::meta_facebook_verify)
                .post(routes::webhooks::meta_facebook_inbound),
        )
        .route(
            "/webhooks/meta/instagram",
            get(routes::webhooks::meta_instagram_verify)
                .post(routes::webhooks::meta_instagram_inbound),
        )
        .route("/webhooks/telegram", post(routes::webhooks::telegram_inbound))
        .route("/webhooks/line", post(routes::webhooks::line_inbound))
        .route(
            "/webhooks/tiktok",
            get(routes::webhooks::tiktok_webhook_verify)
                .post(routes::webhooks::tiktok_webhook_inbound),
        )
        .route("/webhooks/email/inbound", post(routes::webhooks::email_inbound))
        .route("/webhooks/postmark/inbound", post(routes::webhooks::postmark_inbound))
        .route("/webhooks/voice/inbound", post(routes::webhooks::voice_inbound))
        .route("/webhooks/voice/status", post(routes::webhooks::voice_status))
        .route("/webhooks/voice/recording", post(routes::webhooks::voice_recording))
        .route(
            "/webhooks/voice/transcription",
            post(routes::webhooks::voice_transcription),
        )
        .route(
            "/webhooks/ecosystem/events",
            post(routes::webhooks::ecosystem_event_inbound),
        )
        .route(
            "/live-proof/sessions",
            post(routes::live_proof::create_session),
        )
        .route(
            "/live-proof/sessions/{session_id}",
            get(routes::live_proof::get_session_details),
        )
        .route(
            "/live-proof/sessions/{session_id}/decision",
            post(routes::live_proof::record_decision),
        )
        .route(
            "/live-proof/sessions/{session_id}/evidence",
            post(routes::live_proof::upload_evidence),
        )
        .route(
            "/live-proof/sessions/{session_id}/evidence/{evidence_id}/media",
            get(routes::live_proof::evidence_media),
        )
        .route(
            "/translation/languages",
            get(routes::translation::list_supported_languages),
        )
        .route(
            "/translation/settings",
            get(routes::translation::get_translation_settings)
                .put(routes::translation::update_translation_settings),
        )
        .route(
            "/translation/detect",
            post(routes::translation::detect_language_endpoint),
        )
        .route(
            "/translation/translate",
            post(routes::translation::translate_text),
        )
        .route(
            "/translation/messages/{message_id}/translate",
            post(routes::translation::translate_message),
        )
        .route(
            "/translation/conversations/{conversation_id}/prepare-outbound",
            post(routes::translation::prepare_outbound),
        )
        .route(
            "/crypto-mesh/ingress",
            post(routes::crypto_mesh::ingress),
        )
        .route(
            "/crypto-mesh/egress",
            post(routes::crypto_mesh::egress),
        )
        .route("/crypto-mesh/outbox", get(routes::crypto_mesh::outbox))
        .route(
            "/crypto-mesh/outbox/flush",
            post(routes::crypto_mesh::flush_outbox),
        )
        .route("/widget/config", get(routes::widget::get_widget_config))
        .route("/widget/prechat", get(routes::widget::get_prechat_form))
        .route(
            "/widget/prechat/submit",
            post(routes::widget::submit_prechat_form),
        )
        .route("/widget/contact", post(routes::widget::register_contact))
        .route("/widget/messages", post(routes::widget::send_message))
        .route(
            "/widget/conversations/{conversation_id}/messages",
            get(routes::widget::list_messages),
        )
        .route(
            "/widget/conversations/{conversation_id}/messages/since",
            get(routes::widget::poll_messages),
        )
        .route("/widget/read", post(routes::widget::mark_read))
        .route(
            "/widget/presence",
            get(routes::widget::get_presence).post(routes::widget::update_presence),
        )
        .route("/widget/typing", post(routes::widget::set_typing))
        .route("/widget/session", get(routes::widget::session_state))
        .route("/widget/session/close", post(routes::widget::close_session))
        .route("/widget/session/leave", post(routes::widget::session_leave))
        .route("/widget/transcript", get(routes::widget::get_transcript))
        .route("/widget/rate", post(routes::widget::rate_conversation))

        // Monitoring & Alerting (Phase 8)
        .route("/monitoring/health", get(routes::monitoring::health_check_extended))
        .route("/monitoring/dashboard", get(routes::monitoring::get_dashboard_summary))
        .route("/monitoring/metrics", get(routes::monitoring::get_metrics))
        .route("/monitoring/metrics/summary", get(routes::monitoring::get_metrics_summary))
        .route("/monitoring/metrics/collect", post(routes::monitoring::collect_metrics))
        .route("/monitoring/alerts", get(routes::monitoring::get_alert_events))
        .route("/monitoring/alerts/evaluate", post(routes::monitoring::evaluate_alerts))
        .route(
            "/monitoring/alerts/{id}/acknowledge",
            post(routes::monitoring::acknowledge_alert),
        )
        .route(
            "/monitoring/alerts/{id}/resolve",
            post(routes::monitoring::resolve_alert),
        )
        .route("/monitoring/alert-rules", get(routes::monitoring::get_alert_rules).post(routes::monitoring::create_alert_rule))
        .route("/monitoring/alert-rules/{id}", get(routes::monitoring::get_alert_rule).put(routes::monitoring::update_alert_rule).delete(routes::monitoring::delete_alert_rule))

        // AI-Powered Suggestions (Phase 9)
        .route("/suggestions", get(routes::suggestions::list_suggestions))
        .route("/suggestions/summary", get(routes::suggestions::get_suggestion_summary))
        .route("/suggestions/{id}/approve", post(routes::suggestions::approve_suggestion))
        .route("/suggestions/{id}/dismiss", post(routes::suggestions::dismiss_suggestion))
        .route("/suggestions/batch-analyze", post(routes::suggestions::trigger_batch_analysis))
        .route("/suggestions/analyze-feedback/{feedback_id}", post(routes::suggestions::analyze_feedback))

        // Topic Categorization (Phase 11)
        .route("/topics/summary", get(routes::topic_categorization::get_topic_summary))
        .route("/topics/{conversation_id}", get(routes::topic_categorization::get_conversation_topic)
            .post(routes::topic_categorization::categorize_conversation)
            .put(routes::topic_categorization::override_conversation_topic))
        .route("/topics/batch-categorize", post(routes::topic_categorization::batch_categorize))

        // CX Scoring & Automated QA (Phase 16)
        .route("/cx-qa/summary", get(routes::cx_qa_scoring::get_cx_summary))
        .route("/cx-qa/{conversation_id}", get(routes::cx_qa_scoring::get_qa_score))
        .route("/cx-qa/batch-score", post(routes::cx_qa_scoring::batch_score))
        .route("/cx-qa/rules", get(routes::cx_qa_scoring::list_qa_rules))
        .route("/cx-qa/rules/{id}", put(routes::cx_qa_scoring::update_qa_rule))

        // CSAT Surveys (Phase 15)
        .route("/csat/templates", get(routes::csat_surveys::list_templates)
            .post(routes::csat_surveys::create_template))
        .route("/csat/templates/{id}", get(routes::csat_surveys::get_template)
            .put(routes::csat_surveys::update_template)
            .delete(routes::csat_surveys::delete_template))
        .route("/csat/surveys", get(routes::csat_surveys::list_surveys)
            .post(routes::csat_surveys::create_survey))
        .route("/csat/surveys/{id}/respond", post(routes::csat_surveys::respond_to_survey))
        .route("/csat/surveys/{id}/send", post(routes::csat_surveys::send_survey))
        .route("/csat/summary", get(routes::csat_surveys::get_csat_summary))
        .route("/csat/snapshots", get(routes::csat_surveys::list_snapshots)
            .post(routes::csat_surveys::generate_snapshot))

        // Copilot for Human Reviewers (Phase 10)
        .route("/copilot-reviewer/drafts/{draft_id}/context", get(routes::copilot_reviewer::get_draft_context))
        .route("/copilot-reviewer/pending-contexts", get(routes::copilot_reviewer::get_pending_contexts))
        .route("/copilot-reviewer/drafts/{draft_id}/refresh", post(routes::copilot_reviewer::refresh_draft_context))

        // Fin Operator — Meta-agent for autonomous self-improvement (Phase 17)
        .route("/operator/summary", get(routes::fin_operator::get_summary))
        .route("/operator/config", get(routes::fin_operator::get_config).put(routes::fin_operator::update_config))
        .route("/operator/scan", post(routes::fin_operator::run_scan))
        .route("/operator/proposals", get(routes::fin_operator::list_proposals))
        .route("/operator/proposals/{id}", get(routes::fin_operator::get_proposal))
        .route("/operator/proposals/{id}/approve", post(routes::fin_operator::approve_proposal))
        .route("/operator/proposals/{id}/dismiss", post(routes::fin_operator::dismiss_proposal))
        .route("/operator/logs", get(routes::fin_operator::list_logs))
        .route("/operator/snapshots", get(routes::fin_operator::list_snapshots))

        // AI Procedures & Actions (Phase 18)
        .route("/procedures/summary", get(routes::procedures::get_summary))
        .route("/procedures", get(routes::procedures::list_procedures).post(routes::procedures::create_procedure))
        .route("/procedures/match", post(routes::procedures::match_procedures))
        .route("/procedures/{id}", get(routes::procedures::get_procedure).put(routes::procedures::update_procedure).delete(routes::procedures::delete_procedure))
        .route("/procedures/{id}/execute", post(routes::procedures::start_execution))
        .route("/procedures/executions/{id}", get(routes::procedures::get_execution))
        .route("/procedures/executions/conversation/{conversation_id}", get(routes::procedures::get_running_execution))
        .route("/procedures/executions/{id}/step", post(routes::procedures::execute_step))
        .route("/procedures/executions/{id}/complete", post(routes::procedures::complete_execution))
        .route("/procedures/executions/{id}/fail", post(routes::procedures::fail_execution))
        .route("/actions", get(routes::procedures::list_actions).post(routes::procedures::create_action))
        .route("/actions/{id}", get(routes::procedures::get_action).put(routes::procedures::update_action).delete(routes::procedures::delete_action))
        .route("/actions/{id}/test", post(routes::procedures::test_action))

        // Unified Inbox (Phase 19)
        .route("/unified-inbox", get(routes::unified_inbox::list_unified_inbox))
        .route("/unified-inbox/resolve", post(routes::unified_inbox::resolve_customer))
        .route("/unified-inbox/mapping", post(routes::unified_inbox::create_customer_mapping))
        .route("/unified-inbox/settings", get(routes::unified_inbox::get_unified_inbox_settings).put(routes::unified_inbox::update_unified_inbox_settings))
        .route("/unified-inbox/{customer_id}", get(routes::unified_inbox::get_unified_customer))

        // Visual Workflow Builder (Phase 20)
        .route("/workflows/summary", get(routes::workflow_builder::workflow_summary))
        .route("/workflows", get(routes::workflow_builder::list_workflows).post(routes::workflow_builder::create_workflow))
        .route("/workflows/available-triggers", get(routes::workflow_builder::available_triggers))
        .route("/workflows/executions", get(routes::workflow_builder::list_executions))
        .route("/workflows/{id}", get(routes::workflow_builder::get_workflow).put(routes::workflow_builder::update_workflow).delete(routes::workflow_builder::delete_workflow))
        .route("/workflows/{id}/steps", post(routes::workflow_builder::add_step))
        .route("/workflows/{id}/conditions", post(routes::workflow_builder::add_condition))
        .route("/workflows/{id}/triggers", post(routes::workflow_builder::add_trigger))
        .route("/workflows/{id}/execute", post(routes::workflow_builder::trigger_execution))
        .route("/workflows/{id}/evaluate-conditions", post(routes::workflow_builder::evaluate_conditions))

        // Public Help Center (Phase 21)
        .route("/help-center/summary", get(routes::help_center::get_summary))
        .route("/help-center/public", get(routes::help_center::get_public))
        .route("/help-center/categories", get(routes::help_center::list_categories).post(routes::help_center::create_category))
        .route("/help-center/articles", get(routes::help_center::list_articles).post(routes::help_center::create_article))
        .route("/help-center/articles/search", get(routes::help_center::search))
        .route("/help-center/articles/slug/{slug}", get(routes::help_center::get_article_by_slug))
        .route("/help-center/articles/{id}", put(routes::help_center::update_article).delete(routes::help_center::delete_article))
        .route("/help-center/articles/{id}/feedback", post(routes::help_center::record_feedback))
        .route("/help-center/settings", get(routes::help_center::get_settings).put(routes::help_center::update_settings))

        // CRM Sync (Phase 22)
        .route("/crm-sync/summary", get(routes::crm_sync::get_crm_sync_summary))
        .route("/crm-sync/providers", get(routes::crm_sync::get_crm_providers))
        .route("/crm-sync/connections", get(routes::crm_sync::list_crm_connections).post(routes::crm_sync::create_crm_connection))
        .route("/crm-sync/connections/{id}", put(routes::crm_sync::update_crm_connection).delete(routes::crm_sync::delete_crm_connection))
        .route("/crm-sync/connections/{id}/sync", post(routes::crm_sync::trigger_sync))
        .route("/crm-sync/logs", get(routes::crm_sync::list_sync_logs))
        .route("/crm-sync/mappings", get(routes::crm_sync::list_field_mappings))
        .route("/crm-sync/refs", get(routes::crm_sync::get_external_refs).post(routes::crm_sync::create_external_ref))

        // KB Document Upload & URL Crawling (Phase 23)
        .route("/kb-import/summary", get(routes::kb_documents::get_summary))
        .route("/kb-import/upload", post(routes::kb_documents::upload_document_handler))
        .route("/kb-import/documents", get(routes::kb_documents::list_documents_handler))
        .route("/kb-import/documents/{id}/chunks", get(routes::kb_documents::get_document_chunks_handler))
        .route("/kb-import/documents/{id}", delete(routes::kb_documents::delete_document_handler))
        .route("/kb-import/urls", get(routes::kb_documents::list_url_sources_handler).post(routes::kb_documents::add_url_source_handler))
        .route("/kb-import/urls/{id}/crawl", post(routes::kb_documents::crawl_url_handler))
        .route("/kb-import/urls/{id}", delete(routes::kb_documents::delete_url_source_handler))
        .route("/kb-import/settings", get(routes::kb_documents::get_settings_handler).put(routes::kb_documents::update_settings_handler))

        // nID Sync System (Phase 32) - shared PostgreSQL with nID
        .route("/nid-sync/launchpad", get(routes::nid_sync::get_launchpad_overview))
        .route("/nid-sync/personal-profile/{account_id}", get(routes::nid_sync::get_personal_profile))
        .route("/nid-sync/work-profile/{org_id}", get(routes::nid_sync::get_work_profile))
        .route("/nid-sync/organizations", get(routes::nid_sync::get_user_organizations))
        .route("/nid-sync/memberships", get(routes::nid_sync::get_user_memberships))
        .route("/nid-sync/invoices", get(routes::nid_sync::get_invoices))
        .route("/nid-sync/payments", get(routes::nid_sync::get_payments))
        .route("/nid-sync/subscriptions", get(routes::nid_sync::get_subscriptions))
        .route("/nid-sync/order-platform", post(routes::nid_sync::set_order_platform_info))
        .route("/nid-sync/tenants", get(routes::nid_sync::get_tenants))

        // Fin Vision - Image Understanding (Phase 25)
        .route("/vision/summary", get(routes::fin_vision::get_summary))
        .route("/vision/upload", post(routes::fin_vision::upload_image))
        .route("/vision/analyze/{image_id}", post(routes::fin_vision::analyze_image))
        .route("/vision/images", get(routes::fin_vision::list_images))
        .route("/vision/images/{image_id}", get(routes::fin_vision::get_image).delete(routes::fin_vision::delete_image))
        .route("/vision/settings", get(routes::fin_vision::get_settings).put(routes::fin_vision::update_settings))

        // Audience-Based Knowledge Scoping (Phase 26)
        .route("/audience-kb/summary", get(routes::audience_kb_scoping::get_summary))
        .route("/audience-kb/settings", get(routes::audience_kb_scoping::get_settings).put(routes::audience_kb_scoping::update_settings))
        .route("/audience-kb/links", get(routes::audience_kb_scoping::list_links).post(routes::audience_kb_scoping::link_kb))
        .route("/audience-kb/links/{segment_id}/{kb_entry_id}", delete(routes::audience_kb_scoping::unlink_kb))
        .route("/audience-kb/scoped", get(routes::audience_kb_scoping::get_scoped_kb))
        .route("/audience-kb/memberships", get(routes::audience_kb_scoping::list_memberships).post(routes::audience_kb_scoping::add_membership))
        .route("/audience-kb/memberships/{segment_id}/{contact_id}", delete(routes::audience_kb_scoping::remove_membership))
        .route("/audience-kb/usage-log", post(routes::audience_kb_scoping::log_usage))
        .route("/audience-kb/usage/{segment_id}", get(routes::audience_kb_scoping::get_usage_logs))

        // Brand Overrides in Workflows (Phase 27)
        .route("/brand-overrides/summary", get(routes::brand_overrides::summary))
        .route("/brand-overrides/rules", get(routes::brand_overrides::list_rules).post(routes::brand_overrides::create_rule))
        .route("/brand-overrides/rules/{rule_id}", put(routes::brand_overrides::update_rule).delete(routes::brand_overrides::delete_rule))
        .route("/brand-overrides/switch", post(routes::brand_overrides::switch_brand))
        .route("/brand-overrides/switch-log/{conversation_id}", get(routes::brand_overrides::switch_log))
        .route("/brand-overrides/settings", get(routes::brand_overrides::get_settings).put(routes::brand_overrides::update_settings))
        .route("/brand-overrides/hierarchy", get(routes::brand_overrides::list_hierarchy).post(routes::brand_overrides::add_hierarchy))
        .route("/brand-overrides/hierarchy/remove", delete(routes::brand_overrides::remove_hierarchy))
        .route("/brand-overrides/matching", get(routes::brand_overrides::find_matching_rules))
        .route("/brand-overrides/sub-brands/{parent_brand_id}", get(routes::brand_overrides::get_sub_brands))

        // MCP Protocol Support (Phase 28)
        .route("/mcp/summary", get(routes::mcp_protocol::get_summary))
        .route("/mcp/servers", get(routes::mcp_protocol::get_servers).post(routes::mcp_protocol::create_server))
        .route("/mcp/servers/{id}", put(routes::mcp_protocol::update_server).delete(routes::mcp_protocol::delete_server))
        .route("/mcp/servers/{id}/discover", post(routes::mcp_protocol::discover_tools))
        .route("/mcp/servers/{id}/call", post(routes::mcp_protocol::call_tool))
        .route("/mcp/tools", get(routes::mcp_protocol::get_tools))
        .route("/mcp/templates", get(routes::mcp_protocol::get_templates))
        .route("/mcp/settings", get(routes::mcp_protocol::get_settings).put(routes::mcp_protocol::update_settings))
        .route("/mcp/calls", get(routes::mcp_protocol::get_call_logs))

        // Autonomous Resolution (Phase 29)
        .route("/autonomous-resolution/summary", get(routes::autonomous_resolution::get_summary))
        .route("/autonomous-resolution/config", get(routes::autonomous_resolution::get_config).put(routes::autonomous_resolution::update_config))
        .route("/autonomous-resolution/escalation-rules", get(routes::autonomous_resolution::list_escalation_rules).post(routes::autonomous_resolution::create_escalation_rule))
        .route("/autonomous-resolution/escalation-rules/{id}", put(routes::autonomous_resolution::update_escalation_rule).delete(routes::autonomous_resolution::delete_escalation_rule))
        .route("/autonomous-resolution/policies", get(routes::autonomous_resolution::list_policies))
        .route("/autonomous-resolution/evaluate", post(routes::autonomous_resolution::evaluate_decision))
        .route("/autonomous-resolution/log-attempt", post(routes::autonomous_resolution::log_attempt))
        .route("/autonomous-resolution/logs", get(routes::autonomous_resolution::list_logs))
        .route("/autonomous-resolution/snapshot", post(routes::autonomous_resolution::generate_snapshot))

        // Agent Performance Analytics (Phase 30)
        .route("/agent-performance/summary", get(routes::agent_performance::get_summary))
        .route("/agent-performance/snapshots", get(routes::agent_performance::list_snapshots).post(routes::agent_performance::create_snapshot))
        .route("/agent-performance/insights", get(routes::agent_performance::list_insights).post(routes::agent_performance::create_insight))
        .route("/agent-performance/insights/{id}/address", put(routes::agent_performance::address_insight))
        .route("/agent-performance/comparison", get(routes::agent_performance::get_comparison))
        .route("/agent-performance/comparison/generate", post(routes::agent_performance::generate_comparison))
        .route("/agent-performance/settings", get(routes::agent_performance::get_settings).put(routes::agent_performance::update_settings))

        // Real-time Notifications & WebSockets (Phase 31)
        .route("/notifications/summary", get(routes::notifications::get_summary))
        .route("/notifications", get(routes::notifications::list))
        .route("/notifications/queue", post(routes::notifications::queue))
        .route("/notifications/broadcast", post(routes::notifications::broadcast))
        .route("/notifications/{id}/read", put(routes::notifications::mark_read))
        .route("/notifications/{id}/dismiss", put(routes::notifications::dismiss))
        .route("/notifications/preferences/{user_id}", get(routes::notifications::get_preferences).put(routes::notifications::update_preferences))
        .route("/notifications/settings", get(routes::notifications::get_settings).put(routes::notifications::update_settings))
        .route("/notifications/connections", get(routes::notifications::list_connections))
        .route("/notifications/cleanup", post(routes::notifications::cleanup))

        // In-App Messaging & Proactive Triggers (Phase 32)
        .route("/inapp/summary", get(routes::in_app_messaging::get_summary))
        .route("/inapp/messages", get(routes::in_app_messaging::list_messages).post(routes::in_app_messaging::create_message))
        .route("/inapp/messages/{id}", get(routes::in_app_messaging::get_message).put(routes::in_app_messaging::update_message).delete(routes::in_app_messaging::delete_message))
        .route("/inapp/triggers", get(routes::in_app_messaging::list_triggers).post(routes::in_app_messaging::create_trigger))
        .route("/inapp/triggers/{id}", put(routes::in_app_messaging::update_trigger).delete(routes::in_app_messaging::delete_trigger))
        .route("/inapp/events/behavioral", post(routes::in_app_messaging::record_behavioral_event))
        .route("/inapp/events/behavioral/list", get(routes::in_app_messaging::list_behavioral_events))
        .route("/inapp/events/message", post(routes::in_app_messaging::record_message_event))
        .route("/inapp/triggers/evaluate", post(routes::in_app_messaging::evaluate_triggers))
        .route("/inapp/settings", get(routes::in_app_messaging::get_settings).put(routes::in_app_messaging::update_settings))

        // Tone of Voice & Guidance (Phase 33)
        .route("/tone-voice/summary", get(routes::tone_of_voice::get_summary))
        .route("/tone-voice/config", get(routes::tone_of_voice::get_config).put(routes::tone_of_voice::update_config))
        .route("/tone-voice/apply-preset", post(routes::tone_of_voice::apply_preset))
        .route("/tone-voice/guidance", get(routes::tone_of_voice::list_guidance).post(routes::tone_of_voice::create_guidance))
        .route("/tone-voice/guidance/{id}", put(routes::tone_of_voice::update_guidance).delete(routes::tone_of_voice::delete_guidance))
        .route("/tone-voice/prompt-additions", get(routes::tone_of_voice::get_prompt_additions))
        .route("/tone-voice/analytics", post(routes::tone_of_voice::create_analytics))

        // Embed Widget (Phase 35)
        .route("/embed-widget/summary", get(routes::embed_widget::get_summary))
        .route("/embed-widget/configs", get(routes::embed_widget::list_configs).post(routes::embed_widget::create_config))
        .route("/embed-widget/configs/{id}", get(routes::embed_widget::get_config).put(routes::embed_widget::update_config).delete(routes::embed_widget::delete_config))
        .route("/embed-widget/settings", get(routes::embed_widget::get_settings).put(routes::embed_widget::update_settings))
        .route("/embed-widget/events", get(routes::embed_widget::get_events).post(routes::embed_widget::record_event))
        .route("/embed-widget/public/{config_id}", get(routes::embed_widget::get_public_widget_data))
        .route("/embed-widget/script/{config_id}", get(routes::embed_widget::get_embed_script))

        // Unified Analytics (Phase 36)
        .route("/analytics/summary", get(routes::unified_analytics::get_summary))
        .route("/analytics/reports", get(routes::unified_analytics::list_saved_reports).post(routes::unified_analytics::create_saved_report))
        .route("/analytics/reports/{id}", put(routes::unified_analytics::update_saved_report).delete(routes::unified_analytics::delete_saved_report))
        .route("/analytics/reports/{id}/run", post(routes::unified_analytics::run_report))
        .route("/analytics/scheduled", get(routes::unified_analytics::list_scheduled_reports).post(routes::unified_analytics::create_scheduled_report))
        .route("/analytics/scheduled/{id}", put(routes::unified_analytics::update_scheduled_report).delete(routes::unified_analytics::delete_scheduled_report))
        .route("/analytics/templates", get(routes::unified_analytics::list_templates).post(routes::unified_analytics::create_template))
        .route("/analytics/widgets", get(routes::unified_analytics::list_widgets).post(routes::unified_analytics::create_widget))
        .route("/analytics/widgets/{id}", put(routes::unified_analytics::update_widget).delete(routes::unified_analytics::delete_widget))
        .route("/analytics/snapshots", get(routes::unified_analytics::list_snapshots).post(routes::unified_analytics::create_snapshot))
        .route("/analytics/settings", get(routes::unified_analytics::get_settings).put(routes::unified_analytics::update_settings))
        .route("/analytics/export", get(routes::unified_analytics::export_report))

        // Reply Engine Integration (Phase 37)
        .route("/reply-integration/summary", get(routes::reply_integration::get_summary))
        .route("/reply-integration/settings", get(routes::reply_integration::get_settings).put(routes::reply_integration::update_settings))
        .route("/reply-integration/build-context", post(routes::reply_integration::build_context))
        .route("/reply-integration/sources", get(routes::reply_integration::get_knowledge_sources))

        // A/B Testing & Phased Deployments (Phase 38)
        .route("/ab-testing/summary", get(routes::ab_testing::get_summary))
        .route("/ab-testing/experiments", get(routes::ab_testing::list_experiments).post(routes::ab_testing::create_experiment))
        .route("/ab-testing/experiments/{id}", get(routes::ab_testing::get_experiment).delete(routes::ab_testing::delete_experiment))
        .route("/ab-testing/experiments/{id}/status", put(routes::ab_testing::update_experiment_status))
        .route("/ab-testing/experiments/{id}/variants", get(routes::ab_testing::get_variants))
        .route("/ab-testing/experiments/{id}/assign", post(routes::ab_testing::assign_conversation))
        .route("/ab-testing/experiments/{id}/results", post(routes::ab_testing::record_result).get(routes::ab_testing::get_results))
        .route("/ab-testing/experiments/{id}/summary", post(routes::ab_testing::generate_summary))
        .route("/ab-testing/settings", get(routes::ab_testing::get_settings).put(routes::ab_testing::update_settings))

        // Scheduled Tasks & Cron Jobs (Phase 40)
        .route("/scheduled-tasks/summary", get(routes::scheduled_tasks::get_summary))
        .route("/scheduled-tasks", get(routes::scheduled_tasks::list_tasks).post(routes::scheduled_tasks::create_task))
        .route("/scheduled-tasks/{id}", get(routes::scheduled_tasks::get_task).put(routes::scheduled_tasks::update_task).delete(routes::scheduled_tasks::delete_task))
        .route("/scheduled-tasks/{id}/trigger", post(routes::scheduled_tasks::trigger_task))
        .route("/scheduled-tasks/process-queue", post(routes::scheduled_tasks::process_queue))
        .route("/scheduled-tasks/runs", get(routes::scheduled_tasks::list_runs))
        .route("/scheduled-tasks/queue", get(routes::scheduled_tasks::list_queue))
        .route("/scheduled-tasks/settings", get(routes::scheduled_tasks::get_settings).put(routes::scheduled_tasks::update_settings))
        .route("/scheduled-tasks/cleanup", post(routes::scheduled_tasks::cleanup_old_logs))
        .route("/scheduled-tasks/task-types", get(routes::scheduled_tasks::get_task_types))

        // Compliance & Data Privacy Dashboard (Phase 41)
        .route("/compliance/summary", get(routes::compliance_dashboard::get_summary))
        .route("/compliance/suppressions", get(routes::compliance_dashboard::list_suppressions).post(routes::compliance_dashboard::add_suppression))
        .route("/compliance/suppressions/{id}", get(routes::compliance_dashboard::get_suppression).delete(routes::compliance_dashboard::remove_suppression))
        .route("/compliance/suppressions/check", get(routes::compliance_dashboard::check_suppressed))
        .route("/compliance/gdpr-requests", get(routes::compliance_dashboard::list_gdpr_requests).post(routes::compliance_dashboard::create_gdpr_request))
        .route("/compliance/gdpr-requests/{id}", get(routes::compliance_dashboard::get_gdpr_request).put(routes::compliance_dashboard::update_gdpr_request))
        .route("/compliance/policies", get(routes::compliance_dashboard::list_policies).post(routes::compliance_dashboard::create_policy))
        .route("/compliance/policies/{id}", get(routes::compliance_dashboard::get_policy).put(routes::compliance_dashboard::update_policy).delete(routes::compliance_dashboard::delete_policy))
        .route("/compliance/audit-logs", get(routes::compliance_dashboard::list_audit_logs))
        .route("/compliance/check", get(routes::compliance_dashboard::check_compliance))
        .route("/compliance/settings", get(routes::compliance_dashboard::get_settings).put(routes::compliance_dashboard::update_settings))

        // Conversation Search & Discovery (Phase 42)
        .route("/search/summary", get(routes::conversation_search::get_summary))
        .route("/search", get(routes::conversation_search::search))
        .route("/search/saved", get(routes::conversation_search::list_saved_searches).post(routes::conversation_search::create_saved_search))
        .route("/search/saved/{id}", get(routes::conversation_search::get_saved_search).delete(routes::conversation_search::delete_saved_search))
        .route("/search/saved/{id}/pin", post(routes::conversation_search::pin_saved_search))
        .route("/search/saved/{id}/use", post(routes::conversation_search::use_saved_search))
        .route("/search/history", get(routes::conversation_search::get_search_history).delete(routes::conversation_search::clear_search_history))
        .route("/search/analytics", get(routes::conversation_search::get_analytics))
        .route("/search/analytics/snapshot", post(routes::conversation_search::generate_analytics_snapshot))
        .route("/search/settings", get(routes::conversation_search::get_settings).put(routes::conversation_search::update_settings))

        // System Health Check & Onboarding Wizard (Phase 44)
        .route("/system-health/summary", get(routes::system_health::get_summary))
        .route("/system-health/steps", get(routes::system_health::get_steps))
        .route("/system-health/progress", get(routes::system_health::get_progress))
        .route("/system-health/complete-step", post(routes::system_health::complete_step))
        .route("/system-health/health-check", post(routes::system_health::run_health_check))
        .route("/system-health/health-summary", get(routes::system_health::get_health_summary))
        .route("/system-health/settings", get(routes::system_health::get_settings).put(routes::system_health::update_settings))

        // SMTP & Email Templates (Phase 52)
        .route("/smtp/summary", get(routes::smtp_email::get_summary))
        .route("/smtp", get(routes::smtp_email::list_smtp_settings).post(routes::smtp_email::create_smtp_settings))
        .route("/smtp/{id}", get(routes::smtp_email::get_smtp_settings).put(routes::smtp_email::update_smtp_settings).delete(routes::smtp_email::delete_smtp_settings))
        .route("/smtp/{id}/test", post(routes::smtp_email::test_smtp_connection))
        .route("/email-templates", get(routes::smtp_email::list_email_templates).post(routes::smtp_email::create_email_template))
        .route("/email-templates/{id}", get(routes::smtp_email::get_email_template).put(routes::smtp_email::update_email_template).delete(routes::smtp_email::delete_email_template))
        .route("/email/send", post(routes::smtp_email::send_email))
        .route("/email/logs", get(routes::smtp_email::get_send_logs))

        // Advanced OTP Engine (Phase 53) — shared across all channels with per-brand toggles
        .route("/otp/generate/{brand_id}", post(routes::otp::generate_otp))
        .route("/otp/verify", post(routes::otp::verify_otp))
        .route("/otp/config/{brand_id}", get(routes::otp::get_channel_config).put(routes::otp::update_channel_config))
        .route("/otp/records/{brand_id}", get(routes::otp::list_otp_records))
        .route("/otp/audit/{brand_id}", get(routes::otp::get_audit_log))
        .route("/otp/stats/{brand_id}", get(routes::otp::get_otp_stats))

        // External OTP Integration (Phase 55) — for Shopify, custom stores, etc.
        .route("/external-otp/integrations", post(routes::external_otp::create_integration))
        .route("/external-otp/integrations/{brand_id}", get(routes::external_otp::list_integrations))
        .route("/external-otp/integrations/{integration_id}", delete(routes::external_otp::deactivate_integration))
        .route("/external-otp/generate", post(routes::external_otp::external_generate_otp))
        .route("/external-otp/verify", post(routes::external_otp::external_verify_otp))
        .route("/external-otp/info", get(routes::external_otp::get_integration_info))

        // Shopify Store OTP Integration (Phase 56) — connects Shopify storefronts to OTP engine
        .route("/shopify/connect", post(routes::shopify_store::create_shopify_connection))
        .route("/shopify/connections/{brand_id}", get(routes::shopify_store::list_shopify_connections))
        .route("/shopify/{connection_id}/channels", get(routes::shopify_store::get_channel_preferences))
        .route("/shopify/{connection_id}/channels", put(routes::shopify_store::update_channel_preferences))
        .route("/shopify/{store_domain}/account/create", post(routes::shopify_store::create_shopify_account))
        .route("/shopify/account/verify", post(routes::shopify_store::verify_shopify_account))
        .route("/shopify/account/resend", post(routes::shopify_store::resend_shopify_otp))
        .route("/shopify/webhook/{connection_id}", post(routes::shopify_store::receive_shopify_webhook))

        // Platform-Agnostic Store Connection Management (Phase 57)
        .route("/store/connect", post(routes::store_connection::create_connection))
        .route("/store/connections", get(routes::store_connection::list_connections))
        .route("/store/{connection_id}", get(routes::store_connection::get_connection).put(routes::store_connection::update_connection))
        .route("/store/{connection_id}/deactivate", post(routes::store_connection::deactivate_connection))
        .route("/store/{connection_id}/status", get(routes::store_connection::get_connection_status))
        .route("/store/{connection_id}/health-check", post(routes::store_connection::run_health_check))
        .route("/store/{connection_id}/ui-compatibility", get(routes::store_connection::get_ui_compatibility))
        .route("/store/{connection_id}/account/create", post(routes::store_connection::initiate_account_creation))
        .route("/store/account/verify", post(routes::store_connection::verify_account_creation))
        .route("/store/account/resend/{pending_id}", post(routes::store_connection::resend_otp))
        .route("/store/ui-compatibility/{brand_id}", post(routes::store_connection::submit_ui_compatibility_report))
        .route("/store/brand-isolation/{brand_id}", get(routes::store_connection::get_brand_isolation).put(routes::store_connection::update_brand_isolation))

        // Omni-Channel Broadcast System (Phase 46)
        .route("/broadcasts/summary", get(routes::omni_broadcast::get_summary))
        .route("/broadcasts", get(routes::omni_broadcast::list_broadcasts).post(routes::omni_broadcast::create_broadcast))
        .route("/broadcasts/{id}", get(routes::omni_broadcast::get_broadcast).put(routes::omni_broadcast::update_broadcast).delete(routes::omni_broadcast::delete_broadcast))
        .route("/broadcasts/{id}/status/{status}", put(routes::omni_broadcast::update_broadcast_status))
        .route("/broadcasts/{id}/send", post(routes::omni_broadcast::send_broadcast_handler))
        .route("/broadcasts/{id}/format-preview", get(routes::omni_broadcast::get_format_preview_handler))
        .route("/broadcasts/channels", get(routes::omni_broadcast::list_available_channels))
        .route("/broadcasts/media", post(routes::omni_broadcast::upload_media))
        .route("/broadcasts/{broadcast_id}/media", get(routes::omni_broadcast::list_media))
        .route("/broadcasts/media/{id}", delete(routes::omni_broadcast::delete_media))
        .route("/broadcasts/{broadcast_id}/channels", get(routes::omni_broadcast::list_broadcast_channels))
        .route("/broadcasts/{broadcast_id}/recipients", get(routes::omni_broadcast::list_broadcast_recipients))
        .route("/broadcasts/{broadcast_id}/logs", get(routes::omni_broadcast::list_broadcast_logs))
        .route("/broadcasts/settings", get(routes::omni_broadcast::get_settings).put(routes::omni_broadcast::update_settings))
        .route("/broadcasts/html-preview", post(routes::omni_broadcast::preview_html_for_channels))
        // nID Tenant Sync (Phase 58)
        .route("/nid/webhook", post(routes::nid_tenant_sync::nid_webhook))
        .route("/nid/tenants", get(routes::nid_tenant_sync::list_nid_tenants))
        .route("/nid/tenants/{tenant_id}", get(routes::nid_tenant_sync::get_nid_tenant))
        .route("/nid/connection", get(routes::nid_tenant_sync::get_nid_connection).put(routes::nid_tenant_sync::update_nid_connection))
        .route("/nid/redirect/create", post(routes::nid_tenant_sync::create_redirect_session))
        .route("/nid/redirect/callback", get(routes::nid_tenant_sync::redirect_callback))
        .route("/nid/redirect/{state}", get(routes::nid_tenant_sync::get_redirect_session))
        .route("/nid/sync-log", get(routes::nid_tenant_sync::get_sync_log))
        // nID Organization Sync (Phase 59)
        .route("/nid/org/webhook", post(routes::nid_organization_sync::webhook_handler))
        .route("/nid/org/organizations", get(routes::nid_organization_sync::list_organizations_handler))
        .route("/nid/org/organizations/{id}", get(routes::nid_organization_sync::get_organization_handler))
        .route("/nid/org/connection", get(routes::nid_organization_sync::get_connection_handler).put(routes::nid_organization_sync::update_connection_handler))
        .route("/nid/org/health", get(routes::nid_organization_sync::get_health_handler))
        .route("/nid/org/sync-log", get(routes::nid_organization_sync::get_sync_log_handler))
        .route("/nid/org/redirect/create", post(routes::nid_organization_sync::create_redirect_handler))
        .route("/nid/org/redirect/callback", get(routes::nid_organization_sync::redirect_callback_handler))
         // nID Brand Sync (Phase 60)
         .route("/nid/brand/webhook", post(routes::nid_brand_sync::webhook_handler))
         .route("/nid/brand/brands", get(routes::nid_brand_sync::list_brands_handler))
         .route("/nid/brand/brands/{id}", get(routes::nid_brand_sync::get_brand_handler))
         .route("/nid/brand/connection", get(routes::nid_brand_sync::get_connection_handler).put(routes::nid_brand_sync::update_connection_handler))
         .route("/nid/brand/health", get(routes::nid_brand_sync::health_handler))
         .route("/nid/brand/sync-log", get(routes::nid_brand_sync::get_sync_log_handler))
         .route("/nid/brand/redirect/create", post(routes::nid_brand_sync::create_redirect_handler))
         .route("/nid/brand/redirect/callback", get(routes::nid_brand_sync::redirect_callback_handler))
         // Three-Layer Access Control (Phase 61)
         .merge(routes::access_control::router());

    // Static assets & SPA fallback
    let static_dir = PathBuf::from(&config.static_dir);
    let assets_dir = static_dir.join("assets");
    let index_file = static_dir.join("index.html");

    info!("Static directory configured at: {:?}", static_dir);
    if !index_file.exists() {
        warn!(
            "FRONTEND_DIST={:?} contains no index.html (looked at {:?}); every non-API path will answer 404 until the frontend is built",
            config.static_dir, index_file
        );
    }

    let mut app = Router::new()
        .nest("/api/v1", api_routes)
        .route(
            "/verify/proof/{session_id}",
            get(routes::live_proof::capture_page),
        )
        .layer(cors)
        .with_state(state);

    if assets_dir.exists() {
        info!("Mounting static assets from {:?}", assets_dir);
        app = app.nest_service("/assets", ServeDir::new(&assets_dir));
    } else {
        warn!(
            "{:?} does not exist, so /assets is not served at all; the SPA shell will load without its bundles",
            assets_dir
        );
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
                match tokio::fs::read(&requested_file).await {
                    Ok(bytes) => {
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
                    // A file that exists but cannot be read is a real failure,
                    // not "no such asset": reporting it keeps the fallback from
                    // quietly serving the SPA shell in its place.
                    Err(e) => {
                        warn!("Static asset {:?} exists but could not be read: {e}", requested_file);
                    }
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
                Err(e) => {
                    warn!("SPA index {:?} could not be read: {e}", fallback_index);
                    (
                        StatusCode::NOT_FOUND,
                        format!(
                            "404 Not Found - no static file at \"/{uri_path}\" and no readable SPA index at {}",
                            fallback_index.display()
                        ),
                    )
                        .into_response()
                }
            }
        }
    });

    let host: std::net::IpAddr = config.host.parse().map_err(|e| {
        format!(
            "HOST={:?} is not a valid IP address ({e}); set HOST to an address this machine can bind",
            config.host
        )
    })?;
    let addr = SocketAddr::new(host, config.port);
    info!("Rust backend running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
