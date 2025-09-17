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

pub fn delete(path: &str) -> io::Result<bool> {
    if Path::new(path).exists(){
        fs::remove_dir(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_all(path: &Path, exclude: Option<&[&str]>) -> io::Result<()> {
    let exclude = exclude.unwrap_or(&[]);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if !exclude.iter().any(|e| path.ends_with(e)) {
                if path.is_dir() {
                    read_all(&path, Some(exclude))?;
                } else {
                    println!("{:?}", path);
                }
            }
        }
    } else {
        println!("{:?}", path);
    }
    Ok(())
}