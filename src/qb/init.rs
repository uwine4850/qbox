use crate::{fd, qb::qbox};
use std::{io};
// use std::env;

fn make_boxes() -> io::Result<()>{
    let boxes_path = qbox::get_boxes_path();
    if !boxes_path.exists(){
        let created = fd::dir::make(boxes_path.to_str().unwrap())?;
        if !created {
            return Err(io::Error::other(format!("Error creating directory \"{}\"", boxes_path.to_str().unwrap())));
        }
    }
    Ok(())
}

pub fn init() -> io::Result<()> {
    // read_config();
    // let home = env::var("HOME").expect("HOME not set");
    // fd::dir::read_all(&Path::new(&home).join(".config"), Some(&["sublime-text", "Code - OSS", "chromium", "lite-xl", "spotify"]))?;
    make_boxes()
}