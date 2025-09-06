use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Get the Cargo output directory.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Define the list of .proto files.
    let proto_files = &[
        "protos/data.proto",
        "protos/enums.proto",
        "protos/synthetic.proto",
        "protos/webcast.proto",
    ];

    // Configure and run the prost builder.
    prost_build::Config::new()
        // Add serde derives for JSON serialization.
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        // === THIS IS THE CRITICAL FIX ===
        // We explicitly tell it to compile all protos into a SINGLE file in the OUT_DIR.
        .out_dir(&out_dir)
        .compile_protos(proto_files, &["protos/"])?;

    println!("cargo:rerun-if-changed=build.rs");
    for file in proto_files {
        println!("cargo:rerun-if-changed={}", file);
    }

    Ok(())
}