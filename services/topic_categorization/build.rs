fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/topic_categorization/v1/topic_categorization.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
