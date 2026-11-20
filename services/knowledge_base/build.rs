use std::env;

fn main() {
    let proto_path = env::var("CARGO_MANIFEST_DIR").unwrap() + "/../../proto/communicator/knowledge_base/v1/knowledge_base.proto";
    tonic_build::compile_protos(&proto_path).unwrap();
}
