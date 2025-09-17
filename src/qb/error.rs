use std::{error, fmt, io, path::{PathBuf}};

#[derive(Debug)]
pub enum QboxError {
    MissingQbox(PathBuf),
    MissingConfig(PathBuf),
    VersionPathError(PathBuf, String),
    ConfigParse(serde_yaml::Error),
    IO(io::Error),
}

impl fmt::Display for QboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QboxError::MissingQbox(path) => write!(f, "Qbox dir not found: {}", path.display()),
            QboxError::MissingConfig(path) => write!(f, "Config file not found: {}", path.display()),
            QboxError::VersionPathError(path, err) => write!(f, "Version {} path error: {}", path.display(), err),
            QboxError::ConfigParse(e) => write!(f, "Parse config error: {}", e),
            QboxError::IO(e) => write!(f, "IO error {}", e),
        }
    }
}

impl error::Error for QboxError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            QboxError::ConfigParse(e) => Some(e),
            _ => None,           
        }
    }
}

impl From<serde_yaml::Error> for QboxError {
    fn from(err: serde_yaml::Error) -> Self {
        QboxError::ConfigParse(err)
    }
}

impl From<io::Error> for QboxError {
    fn from(err: io::Error) -> Self {
        QboxError::IO(err)
    }
}