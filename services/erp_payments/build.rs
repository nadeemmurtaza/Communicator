fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/erp_payments/v1/erp_payments.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
