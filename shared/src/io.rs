use std::fs;
use std::io;

/// Read the source file and retun it as a [String].
pub fn read_file_to_string(file_path: &str) -> Result<String, io::Error> {
    fs::read_to_string(file_path)
}

/// Write the output string to the specified file.
pub fn write_string_to_file(file_path: &str, output: &str) -> Result<(), io::Error> {
    fs::write(file_path, output)
}
