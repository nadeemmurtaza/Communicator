use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile(
            &["../../proto/communicator/meta_sync/v1/meta_sync.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
