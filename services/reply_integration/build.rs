fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/reply_integration/v1/reply_integration.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
