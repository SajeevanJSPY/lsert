use std::fs;
use std::io::Result as IOResult;
use std::path::Path;

pub fn read_plaintext_file<P: AsRef<Path>>(file_path: P) -> IOResult<String> {
    let source = fs::read_to_string(file_path)?;

    Ok(source)
}
