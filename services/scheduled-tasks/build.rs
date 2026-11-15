fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/scheduled_tasks/v1/scheduled_tasks.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
