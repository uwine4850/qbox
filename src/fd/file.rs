use std::{io, fs};
use std::path::Path;

use crate::fd::dir;

/// Creates a file along with all directories in the target directory.
/// It is important to understand that the file will only be created relative to the target directory.
/// This means that even an absolute path to the file will still be created in the target directory.
pub fn create_in_dir(filename: &Path, target_dir: &Path) -> io::Result<()>{
    println!("{:?}", target_dir);
    dir::path_exists(target_dir)?;
    let str_filename = filename.to_str().
        expect("invalid utf-8 in source path");
    let formated_write_file_path = str_filename.trim_start_matches('/');
    let new_file_path = target_dir.join(formated_write_file_path);
    let new_file_dir_path = new_file_path.parent()
        .unwrap_or_else(|| panic!("the parent directory for the file \"{}\" does not exist", str_filename));
    fs::create_dir_all(new_file_dir_path)?;
    fs::copy(str_filename, &new_file_path)?;
    Ok(())
}