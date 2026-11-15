fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/system_health/v1/system_health.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
