use std::{fs, io};
use std::path::Path;

pub fn make(path: &str) -> io::Result<bool> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
