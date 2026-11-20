fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/commerce_catalog/v1/commerce_catalog.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
