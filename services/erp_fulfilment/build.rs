fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/erp_fulfilment/v1/erp_fulfilment.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
