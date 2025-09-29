use crate::{fd, qb::{qbox}};
use std::{io};

fn make_boxes(data_dir: &str) -> io::Result<()>{
    let boxes_path = qbox::get_boxes_path(data_dir);
    if !boxes_path.exists(){
        let created = fd::dir::make(boxes_path.to_str().unwrap())?;
        if !created {
            return Err(io::Error::other(format!("Error creating directory \"{}\"", boxes_path.to_str().unwrap())));
        }
    }
    Ok(())
}

pub fn init(data_dir: &str) -> io::Result<()> {
    make_boxes(data_dir)
}