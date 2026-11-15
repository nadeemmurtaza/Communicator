fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/monitoring/v1/monitoring.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
