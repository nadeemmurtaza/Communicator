use std::env;

fn main() {
    let proto_path = env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../proto/communicator/business_profile/v1/business_profile.proto";
    tonic_build::compile_protos(&proto_path).unwrap();
}
