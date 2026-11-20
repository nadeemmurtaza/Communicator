use std::env;

fn main() {
    let proto_path = env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../proto/communicator/automation/v1/automation.proto";
    tonic_build::compile_protos(&proto_path).unwrap();
}
