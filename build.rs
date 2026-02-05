fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf definitions if proto files exist
    let proto_path = "proto/keyring.proto";
    if std::path::Path::new(proto_path).exists() {
        tonic_build::compile_protos(proto_path)?;
    }
    Ok(())
}
