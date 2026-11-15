fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/notifications/v1/notifications.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
