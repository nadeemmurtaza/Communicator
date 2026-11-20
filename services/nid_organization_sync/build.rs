use std::path::PathBuf;

fn main() {
    let mut proto = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    proto.push("../../proto");

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir("src/gen")
        .compile(
            &[proto.join("communicator/nid_organization_sync/v1/nid_organization_sync.proto")],
            &[proto],
        )
        .unwrap_or_else(|e| panic!("proto compile failed: {}", e));
}
