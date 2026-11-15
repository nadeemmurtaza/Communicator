fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/autonomous_resolution/v1/autonomous_resolution.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
