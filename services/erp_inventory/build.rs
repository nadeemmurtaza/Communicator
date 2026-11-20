use std::error::Error;
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/erp_inventory/v1/erp_inventory.proto"],
            &["../../proto/communicator/erp_inventory/v1"],
        )?;
    Ok(())
}
