fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/procedures/v1/procedures.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
