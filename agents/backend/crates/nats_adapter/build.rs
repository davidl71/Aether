fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../proto");

    prost_build::compile_protos(
        &[proto_root.join("messages.proto")],
        &[&proto_root],
    )?;

    println!("cargo:rerun-if-changed=../../../../proto/messages.proto");
    Ok(())
}
