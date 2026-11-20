fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &["../../proto/communicator/services_sequences/v1/services_sequences.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("proto compilation failed: {}", e));
}
