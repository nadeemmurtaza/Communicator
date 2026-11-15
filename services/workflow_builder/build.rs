fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/workflow_builder/v1/workflow_builder.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
