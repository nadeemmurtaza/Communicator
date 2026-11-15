use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("proto");
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/generated")
        .compile(
            &[proto_dir.join("communicator/access_control/v1/access_control.proto").to_str().unwrap()],
            &[proto_dir],
        )?;
    Ok(())
}
