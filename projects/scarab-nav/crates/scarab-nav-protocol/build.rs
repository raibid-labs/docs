fn main() {
    let proto_file = "proto/scarab_nav.proto";
    println!("cargo:rerun-if-changed={}", proto_file);

    prost_build::compile_protos(&[proto_file], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
}
