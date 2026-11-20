use std::error::Error;
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &["../../proto/communicator/nid_tenant_sync/v1/nid_tenant_sync.proto"],
            &["../../proto/communicator/nid_tenant_sync/v1"],
        )?;
    Ok(())
}
