fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/embed_widget/v1/embed_widget.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
