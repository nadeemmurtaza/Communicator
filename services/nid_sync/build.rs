fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/nid_sync/v1/nid_sync.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
