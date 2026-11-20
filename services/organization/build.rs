fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/gen")
        .compile(
            &["communicator/organization/v1/organization.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protobuf: {}", e));
}
