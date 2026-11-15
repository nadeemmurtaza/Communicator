fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/kb_documents/v1/kb_documents.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
