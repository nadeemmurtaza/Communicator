fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/in_app_messaging/v1/in_app_messaging.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
