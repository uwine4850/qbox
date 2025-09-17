use core::fmt;
use std::error;

#[derive(Debug)]
pub enum FDError {
    IO(std::io::Error),
}

impl fmt::Display for FDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FDError::IO(e) => write!(f, "FileDir io error: {}", e),
        }
    }
}

impl error::Error for FDError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            FDError::IO(e) => Some(e),
            _ => None,
        }
    }
}