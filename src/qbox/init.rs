use crate::fd;
use std::path::{PathBuf};
use std::{io};

const BOX_DIR: &str = "boxes";

fn make_boxes() -> io::Result<()>{
    let mut new_path = PathBuf::from(fd::DATA_DIR);
    new_path.push(BOX_DIR);
    if !new_path.exists(){
        let created = fd::dir::make(new_path.to_str().unwrap())?;
        if !created {
            return Err(io::Error::other(format!("Error creating directory \"{}\"", BOX_DIR)));
        }
    }
    Ok(())
}

pub fn init() -> io::Result<()> {
    make_boxes()
}