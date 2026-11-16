fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/unified_inbox/v1/unified_inbox.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
