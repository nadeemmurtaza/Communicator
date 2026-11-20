fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/commerce_orders/v1/commerce_orders.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
