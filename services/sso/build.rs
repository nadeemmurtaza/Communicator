use std::path::PathBuf;

fn main() {
    let mut proto_include = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    proto_include.push("../../proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/gen")
        .compile(
            &["proto/communicator/sso/v1/sso.proto"],
            &[proto_include],
        )
        .unwrap_or_else(|e| panic!("proto compile failed: {}", e));
}
