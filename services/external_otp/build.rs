fn main() {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &["../../proto/communicator/external_otp/v1/external_otp.proto"],
            &["../../proto"],
        )
        .unwrap_or_else(|e| panic!("proto compilation failed: {}", e));
}
