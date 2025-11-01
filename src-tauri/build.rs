fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use default OUT_DIR instead of custom path
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["../proto/murmure.proto"], &["../proto"])?;
    Ok(())
}
