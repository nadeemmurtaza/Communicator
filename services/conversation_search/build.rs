fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/conversation_search/v1/conversation_search.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
