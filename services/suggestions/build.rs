fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/communicator/suggestions/v1/suggestions.proto")?;
    Ok(())
}
