use std::{fs, io};
use std::path::{Path, PathBuf};

pub fn make(path: &str) -> io::Result<bool> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete(path: &str, force: bool) -> io::Result<bool> {
    if Path::new(path).exists(){
        if force {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_dir(path)?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_all(path: &Path, exclude: Option<&Vec<&str>>) -> io::Result<Vec<PathBuf>> {
    let binding = Vec::new();
    let exclude = exclude.unwrap_or(&binding);
    let mut curr: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !exclude.iter().any(|e| entry.path().ends_with(e)) {
                if entry.path().is_dir() {
                    let sub = read_all(&entry.path(), Some(exclude))?;
                    curr.extend(sub);
                } else {
                    curr.push(entry.path());
                }
            }
        }
        return Ok(curr);
    } else {
        curr.push(path.to_path_buf());
    }
    Ok(curr)
}

pub fn path_exists(path: &Path) -> io::Result<()>{
    if !path.exists(){
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("path {} does not exist", path.to_str()
                .expect("undefined path")))
        );
    }
    Ok(())
}

pub fn clear(path: &Path) -> io::Result<()> {
    path_exists(path)?;
    fs::remove_dir_all(path)?;
    fs::create_dir_all(path)?;
    Ok(())
}