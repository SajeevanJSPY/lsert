use std::fs;
use std::io::Result as IOResult;
use std::path::Path;

mod xml_file;

// Re-exports
pub use xml_file::read_xml_file;

pub fn read_plain_file<P: AsRef<Path>>(file_path: P) -> IOResult<String> {
    let source = fs::read_to_string(file_path)?;
    Ok(source)
}
