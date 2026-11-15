fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/help_center/v1/help_center.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
