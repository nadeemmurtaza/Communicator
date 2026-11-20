use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile(
            &["../../proto/communicator/omni_broadcast/v1/omni_broadcast.proto"],
            &["../../proto/communicator/omni_broadcast/v1"],
        )
        .unwrap_or_else(|e| panic!("proto compile failed: {}", e));
}
