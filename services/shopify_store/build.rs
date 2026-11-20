fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../../proto/communicator/shopify_store/v1/shopify_store.proto"],
            &["../../proto/communicator/shopify_store/v1"],
        )?;
    Ok(())
}
