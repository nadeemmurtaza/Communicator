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
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile_protos(PROTOS, &["../../proto"])?;
    Ok(())
}
