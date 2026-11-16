fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../proto/communicator/crypto_mesh/v1/crypto_mesh.proto")?;
    Ok(())
}
