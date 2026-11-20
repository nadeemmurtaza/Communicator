fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &["../../proto/communicator/fin_operator/v1/fin_operator.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("proto compilation failed: {}", e));
}
