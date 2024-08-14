fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/registry.proto"], &["."])?;
    tonic_build::configure().compile(&["proto-pipewire/port.proto"], &["."])?;
    Ok(())
}
