fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../proto/communicator/smtp_email/v1/smtp_email.proto"],
            &["../../proto"],
        )?;
    Ok(())
}
