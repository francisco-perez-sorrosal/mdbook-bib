use std::ffi::OsStr;
use std::path::Path;

/// Get filename extension
pub fn get_filename_extension(filename: &Path) -> Option<&str> {
    filename.extension().and_then(OsStr::to_str)
}
