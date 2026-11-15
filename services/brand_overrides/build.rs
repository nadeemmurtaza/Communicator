fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/brand_overrides/v1/brand_overrides.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
