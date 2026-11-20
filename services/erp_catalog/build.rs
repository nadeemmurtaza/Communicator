use std::error::Error;
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/erp_catalog/v1/erp_catalog.proto"],
            &["../../proto/communicator/erp_catalog/v1"],
        )?;
    Ok(())
}
