fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/compliance_dashboard/v1/compliance_dashboard.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
