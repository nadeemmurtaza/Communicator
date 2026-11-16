// Add each new module's .proto file here as it splits out of the
// monolith -- this crate is the one place generated client/server types
// live for every split service, so both the owning service and any
// caller depend on the same generated code instead of each regenerating
// their own copy.
const PROTOS: &[&str] = &[
    "../../proto/communicator/kb_documents/v1/kb_documents.proto",
    "../../proto/communicator/system_health/v1/system_health.proto",
    "../../proto/communicator/embed_widget/v1/embed_widget.proto",
    "../../proto/communicator/help_center/v1/help_center.proto",
    "../../proto/communicator/scheduled_tasks/v1/scheduled_tasks.proto",
    "../../proto/communicator/monitoring/v1/monitoring.proto",
    "../../proto/communicator/conversation_search/v1/conversation_search.proto",
    "../../proto/communicator/reply_integration/v1/reply_integration.proto",
    "../../proto/communicator/cx_qa_scoring/v1/cx_qa_scoring.proto",
    "../../proto/communicator/tone_of_voice/v1/tone_of_voice.proto",
    "../../proto/communicator/ab_testing/v1/ab_testing.proto",
    "../../proto/communicator/compliance_dashboard/v1/compliance_dashboard.proto",
    "../../proto/communicator/unified_analytics/v1/unified_analytics.proto",
    "../../proto/communicator/procedures/v1/procedures.proto",
    "../../proto/communicator/unified_inbox/v1/unified_inbox.proto",
    "../../proto/communicator/notifications/v1/notifications.proto",
    "../../proto/communicator/suggestions/v1/suggestions.proto",
    "../../proto/communicator/crm_sync/v1/crm_sync.proto",
    "../../proto/communicator/crypto_mesh/v1/crypto_mesh.proto",
    "../../proto/communicator/canned_macros_sla/v1/canned_macros_sla.proto",
    "../../proto/communicator/agent_performance/v1/agent_performance.proto",
    "../../proto/communicator/topic_categorization/v1/topic_categorization.proto",
    "../../proto/communicator/workflow_builder/v1/workflow_builder.proto",
    "../../proto/communicator/brand_overrides/v1/brand_overrides.proto",
    "../../proto/communicator/autonomous_resolution/v1/autonomous_resolution.proto",
    "../../proto/communicator/smtp_email/v1/smtp_email.proto",
    "../../proto/communicator/access_control/v1/access_control.proto",
    "../../proto/communicator/copilot_reviewer/v1/copilot_reviewer.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile_protos(PROTOS, &["../../proto"])?;
    Ok(())
}
