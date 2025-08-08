use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=proto/tfplugin6.proto");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tofu_dir = out_dir.join("tofu");
    std::fs::create_dir_all(&tofu_dir).unwrap();
    
    // Try to compile protos, fallback to placeholder if protoc missing
    match tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("tfplugin6_descriptor.bin"))
        .out_dir(&tofu_dir)
        .compile(&["proto/tfplugin6.proto"], &["proto"]) {
        Ok(_) => println!("cargo:warning=Successfully compiled protobuf files"),
        Err(e) => {
            eprintln!("Warning: Failed to compile protos: {}", e);
            eprintln!("Install protoc: https://github.com/protocolbuffers/protobuf/releases");
            
            // Create placeholder file
            let placeholder = r#"// Placeholder - install protoc to generate actual types
pub struct GetProviderSchema;
pub struct ValidateProviderConfig;
pub struct ConfigureProvider;
"#;
            std::fs::write(tofu_dir.join("tfplugin6.rs"), placeholder).unwrap();
        }
    }
}
