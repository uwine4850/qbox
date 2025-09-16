use std::{io, fs};
use std::path::Path;

pub fn make(name: &str) -> io::Result<bool>{
    if !Path::new(name).exists(){
        fs::File::create_new(name)?;
        Ok(true)
    } else {
        Ok(false)
    }
}