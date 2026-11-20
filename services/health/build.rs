fn main() {
    proto_codegen();
}

fn proto_codegen() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/gen")
        .compile(
            &["proto/communicator/health/v1/health.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protobuf: {}", e));
}
