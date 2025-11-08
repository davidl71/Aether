fn main() -> Result<(), Box<dyn std::error::Error>> {
  let proto_path = "../../proto/ib_backend.proto";
  tonic_build::configure()
    .build_client(true)
    .build_server(true)
    .compile(&[proto_path], &["../../proto"])?;
  Ok(())
}
