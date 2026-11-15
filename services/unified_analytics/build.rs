fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/unified_analytics/v1/unified_analytics.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
