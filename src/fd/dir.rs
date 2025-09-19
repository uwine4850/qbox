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

// pub fn read_all(path: &Path, exclude: Option<&Vec<&str>>) -> io::Result<Vec<PathBuf>> {
//     let binding = Vec::new();
//     let exclude = exclude.unwrap_or(&binding);

//     let mut results: Vec<Vec<PathBuf>> = Vec::new();

//     if path.is_dir() {
//         let mut current: Vec<PathBuf> = Vec::new();

//         for entry in fs::read_dir(path)? {
//             let entry = entry?;
//             let path = entry.path();

//             if !exclude.iter().any(|e| path.ends_with(e)) {
//                 if path.is_dir() {
//                     let sub = read_all(&path, Some(exclude))?;
//                     results.push(sub);
//                 } else {
//                     current.push(path);
//                 }
//             }
//         }

//         if !current.is_empty() {
//             // println!("curr {:?}", current);
//             // results.push(current);
//             return Ok(current);
//         }
//     } else {
//         // println!("aa {:?}", path)
//         results.push(vec![path.to_path_buf()]);
//     }

//     Ok(vec![])
// }

pub fn path_exists(path: &Path) -> io::Result<()>{
    if !path.exists(){
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("path {} does not exist", path.to_str()
                .expect("undefined path")))
        );
    }
    Ok(())
}