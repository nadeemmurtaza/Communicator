fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/communicator/crm_sync/v1/crm_sync.proto")?;
    Ok(())
}
