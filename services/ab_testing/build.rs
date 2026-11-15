fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/ab_testing/v1/ab_testing.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
