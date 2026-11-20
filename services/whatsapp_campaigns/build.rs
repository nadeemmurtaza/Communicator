fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/communicator/whatsapp_campaigns/v1/whatsapp_campaigns.proto")?;
    Ok(())
}
