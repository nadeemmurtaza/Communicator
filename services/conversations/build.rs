fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/conversations/v1/conversations.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
