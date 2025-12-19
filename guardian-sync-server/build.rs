fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "proto/auth.proto",
        "proto/sync.proto",
        "proto/file.proto",
        "proto/family.proto",
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(protos, &["proto"])?;

    // Recompile if proto files change
    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    Ok(())
}
