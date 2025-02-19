use std::env;
use std::fs;
use std::path::Path;

const DEFAULT_CRATE_TYPE: &str = "rlib";

fn main() {
    let crate_type = crate_type();

    rewrite_cargo_toml(crate_type.as_str());
}

/// get [lib.crate-type] on target platform in Cargo.toml
fn crate_type() -> String {
    let features: Vec<String> = env::var("CARGO_FEATURES")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.to_string())
        .collect();

    // case: "pyo3" feature is activated
    if features.contains(&"pyo3".to_string()) {
        return "cdylib".to_owned();
    }

    DEFAULT_CRATE_TYPE.to_owned()
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
