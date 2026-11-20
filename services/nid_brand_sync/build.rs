fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &["../../proto/communicator/nid_brand_sync/v1/nid_brand_sync.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("proto compilation failed: {}", e));
}
