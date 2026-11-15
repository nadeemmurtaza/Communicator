// Add each new module's .proto file here as it splits out of the
// monolith -- this crate is the one place generated client/server types
// live for every split service, so both the owning service and any
// caller depend on the same generated code instead of each regenerating
// their own copy.
const PROTOS: &[&str] = &[
    "../../proto/communicator/kb_documents/v1/kb_documents.proto",
    "../../proto/communicator/system_health/v1/system_health.proto",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile_protos(PROTOS, &["../../proto"])?;
    Ok(())
}
