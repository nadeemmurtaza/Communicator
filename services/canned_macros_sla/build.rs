fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/canned_macros_sla/v1/canned_macros_sla.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
