use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set by cargo"));
    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(&out_dir)
        .include(".")
        .input("definition.proto")
        .run()
        .unwrap();

    // Post-process the generated file: Rust edition 2024 does not allow
    // `#![...]` inner attributes or `//!` inner doc comments inside a
    // `mod { include!(...) }` block.  Convert them to harmless equivalents.
    let gen_path = out_dir.join("definition.rs");
    let src =
        std::fs::read_to_string(&gen_path).expect("failed to read generated ap_plmn_mapping.rs");
    let fixed = src
        .lines()
        .map(|line| {
            if line.starts_with("#![") {
                // Replace inner attribute with equivalent outer-style allow on a dummy item,
                // or simply comment it out — suppressing warnings from generated code
                // is not mandatory for correctness.
                format!("// (stripped for edition 2024 compat) {}", line)
            } else if let Some(l) = line.strip_prefix("//!") {
                // Inner doc comment → regular comment
                format!("//{l}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let fixed = if fixed.ends_with('\n') {
        fixed
    } else {
        fixed + "\n"
    };
    std::fs::write(&gen_path, fixed).unwrap();

    println!("cargo:rerun-if-changed=definition.proto");
}
