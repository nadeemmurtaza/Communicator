use std::error::Error;
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/whatsapp_template_sync/v1/whatsapp_template_sync.proto"],
            &["../../proto/communicator/whatsapp_template_sync/v1"],
        )?;
    Ok(())
}
