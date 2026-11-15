fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/tone_of_voice/v1/tone_of_voice.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
