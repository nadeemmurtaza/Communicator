fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/cx_qa_scoring/v1/cx_qa_scoring.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
