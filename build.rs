fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/registry.proto"], &["."])?;
    tonic_build::configure().compile(
        &["../fr-pipewire-registry/proto/pipewire.proto"],
        &["../fr-pipewire-registry/"],
    )?;
    Ok(())
}
