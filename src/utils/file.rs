use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn read_from_filepath<'a>(filepath: &str) -> Result<String, String> {
    let path = Path::new(filepath);

    if !path.exists() || !path.is_file() {
        return Err(format!("File is missing: {}", filepath));
    }

    let ret = fs::read_to_string(path)
        .expect(format!("Failed to read from text file: {}", filepath).as_str());
    Ok(ret)
}

pub fn write_to_filepath(content: &str, filepath: &str, overwrites: bool) -> Result<(), String> {
    let path = Path::new(filepath);

    if path.exists() {
        if !path.is_file() {
            return Err(format!("Found not as file: {}", filepath));
        } else if !overwrites {
            return Err(format!(
                "Not allowed to overwrite existing file: {}",
                filepath
            ));
        }
    }

    let mut file =
        File::create(path).expect(format!("Failed to get file to write to: {}", filepath).as_str());
    file.write_all(content.as_bytes())
        .expect(format!("Failed to write to file: {}", filepath).as_str());

    Ok(())
}
