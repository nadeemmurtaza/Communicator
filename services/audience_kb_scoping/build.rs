fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/audience_kb_scoping/v1/audience_kb_scoping.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
