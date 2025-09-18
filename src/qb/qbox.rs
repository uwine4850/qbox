use std::{collections::HashMap, io, path::PathBuf};
use crate::{fd, qb::{error::QboxError, QBOX_CONFIG_NAME}};
use serde::Deserialize;

const BOX_DIR: &str = "boxes";
pub fn get_boxes_path() -> PathBuf {
    let path = PathBuf::from(fd::DATA_DIR);
    path.join(BOX_DIR)
}

pub fn make_qbox_path(name: &str) -> io::Result<PathBuf>{
    let path = get_boxes_path();
    if !path.exists() {
        return Err(
            io::Error::other("boxes directory not exists")
        );
    }
    let qbox_path = path.join(format!("qbox_{}", name));
    Ok(qbox_path)
}

pub fn make(name: &str) -> io::Result<()>{
    let qbox_path = make_qbox_path(name)?;
    if !qbox_path.exists() {
        let is_make = fd::dir::make(qbox_path.to_str().unwrap())?;
        if !is_make {
            return Err(
                io::Error::other("unexpected error")
            );
        }
        Ok(())
    } else {
        Err(
            io::Error::other(format!("qbox qbox_{} already exists", name))
        )
    }
}

pub fn delete(name: &str) -> io::Result<()>{
    let qbox_path = make_qbox_path(name)?;
    if qbox_path.exists(){
        let is_delete = fd::dir::delete(qbox_path.to_str().unwrap(), false)?;
        if !is_delete {
            return Err(
                io::Error::other("unexpected error")
            );
        }
        Ok(())
    } else {
        Err(
            io::Error::other(format!("qbox qbox_{} not exists",name))
        )
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    make_dir: bool,
    files: Vec<HashMap<String, String>>,
    excludes: Vec<String>,
}

fn read_config(path: PathBuf) -> Result<Config, QboxError>{
    let content = std::fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&content)?;   
    Ok(cfg)
}

#[derive(Debug)]
pub struct Qbox {
    name: String,
    config: Option<Config>,
    qbox_path: PathBuf,
}

impl Qbox {
    pub fn new(name: &str) -> Result<Self, QboxError> {
        let qbox_path = make_qbox_path(name)?;
        if qbox_path.exists() {
            Ok(
                Self { name: name.into(), config: None, qbox_path }
            )
        } else {
            Err(
                QboxError::MissingQbox(qbox_path)
            )
        }
    }

    pub fn open(& mut self) -> Result<&Self, QboxError>{
        let config_path = self.qbox_path.join(QBOX_CONFIG_NAME);
        if config_path.exists(){
            self.config = Some(read_config(config_path)?);
            Ok(self)
        } else {
            Err(
                QboxError::MissingConfig(config_path)
            )
        }
    }

    pub fn new_version(&self, name: &str) -> Result<(), QboxError> {
        let version_path = self.qbox_path.join(name);
        if !version_path.exists(){
            if !fd::dir::make(version_path.to_str().unwrap())? {
                return Err(
                    QboxError::VersionPathError(version_path, "unexpected error".to_string())
                );
            }
        } else {
            return Err(
                QboxError::VersionPathError(version_path, "version already exists".to_string())
            );
        }
        Ok(())
    }

    pub fn remove_version(&self, name: &str, force: bool) -> Result<(), QboxError> {
        let version_path = self.qbox_path.join(name);
        if version_path.exists(){
            if !fd::dir::delete(version_path.to_str().unwrap(), force)? {
                return Err(
                    QboxError::VersionPathError(version_path, "unexpected error".to_string())
                );
            }
        } else {
            return Err(
                QboxError::VersionPathError(version_path, "version not exists".to_string())
            );
        }
        Ok(())
    }
}
