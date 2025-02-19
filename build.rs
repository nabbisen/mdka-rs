use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // get the target platform (e.g., x86_64-unknown-linux-musl)
    let target = env::var("TARGET").unwrap();

    let crate_type = crate_type(target.as_str());

    rewrite_cargo_toml(crate_type.as_str());
}

// [lib.crate-type] on target platform
fn crate_type(_target: &str) -> String {
    // get the enabled features (e.g., pyo3)
    let features: Vec<String> = env::var("CARGO_FEATURES")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.to_string())
        .collect();

    // set the default crate-type
    let mut crate_type = String::from("rlib");
    // check if the "pyo3" feature is enabled
    if features.contains(&"pyo3".to_string()) {
        // set to "cdylib" if "pyo3" feature is enabled
        crate_type = "cdylib".to_string();
    }
    // // if targeting musl (either x86_64 or aarch64), set crate-type to staticlib
    // if target.contains("musl") {
    //     // override crate-type for musl
    //     crate_type = "staticlib".to_string();
    // }
    // // if both pyo3 and musl are enabled, prefer staticlib for musl
    // if features.contains(&"pyo3".to_string()) && target.contains("musl") {
    //     // musl with pyo3 should use staticlib
    //     crate_type = "staticlib".to_string();
    // }
    crate_type
}

fn rewrite_cargo_toml(crate_type: &str) {
    // prepare the path to Cargo.toml
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_toml_path = Path::new(&cargo_manifest_dir).join("Cargo.toml");

    // read the contents of Cargo.toml
    let mut cargo_toml_content = fs::read_to_string(&cargo_toml_path).unwrap();

    let crate_type_entry = format!(
        "crate-type = [\"{}\"] # defined due to target platform by build.rs",
        crate_type
    );
    // check if the `[lib]` section exists
    if let Some(pos) = cargo_toml_content.find("[lib]") {
        // find the `crate-type` line and replace it if it exists
        let start = cargo_toml_content[pos..].find("crate-type").unwrap();
        let end = cargo_toml_content[pos + start..].find('\n').unwrap();
        cargo_toml_content.replace_range(
            pos + start..pos + start + end,
            &format!("{}", crate_type_entry),
        );
    } else {
        // if no `[lib]` section exists, append the crate-type at the end
        cargo_toml_content.push_str(&format!("\n[lib]\n{}\n", crate_type_entry));
    }

    // write the updated content back to Cargo.toml
    fs::write(cargo_toml_path, cargo_toml_content).unwrap();
}
