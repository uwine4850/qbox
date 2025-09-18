use std::{collections::HashMap, env, io, path::{Path, PathBuf}};
use crate::{fd::{self}, qb::{error::QboxError, QBOX_CONFIG_NAME}};
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

const CONFIG_VARIABLES: [&str; 1] = ["HOME"];

#[derive(Debug, Deserialize)]
struct Config {
    make_dir: bool,
    pub files: Vec<HashMap<PathBuf, String>>,
    excludes: Vec<String>,
}

impl Config {
    pub fn variable_data(variable: &str) -> Result<String, QboxError> {
        match variable {
            "HOME" => {
                let home = env::var("HOME")?;
                Ok(home)
            },
            _ => Err(
                QboxError::ConfigUndefinedVariable(variable.to_string())
            ),
        }
    }

    pub fn validate(&mut self) -> Result<(), QboxError> {
        let mut valid_files: Vec<HashMap<PathBuf, String>> = Vec::new();
        for file in &self.files {
            let mut valid_map: HashMap<PathBuf, String> = HashMap::new();
            for (source_path, target_path) in file {
                if target_path != "*" && (!source_path.starts_with("/") || !target_path.starts_with("/")) {
                    return Err(QboxError::IO(
                        io::Error::new(io::ErrorKind::NotFound, "config path must be in absolute format"))
                    );
                }
                let valid_source_path = self.path_validate(source_path)?;
                let valid_target_path = if target_path == "*" {
                    valid_source_path.to_str().expect("unexpected path error").to_string()
                } else if self.make_dir{
                    target_path.to_string()
                } else {
                    self.path_validate(Path::new(target_path))?.to_str()
                        .expect("unexpected path error")
                        .to_string()
                };
                valid_map.insert(valid_source_path, valid_target_path);
            }
            valid_files.push(valid_map);
        }
        self.files = valid_files;
        Ok(())
    }

    fn path_validate(&self, path: &Path) -> Result<PathBuf, QboxError>{
        let system_file_path = path.to_str().unwrap();
        if let Some(dollar_pos) = system_file_path.find("$")
            && let Some(slash_pos) = system_file_path[dollar_pos..].find("/"){
                let variable = &system_file_path[dollar_pos+1..dollar_pos+1 + slash_pos-1];
                if CONFIG_VARIABLES.contains(&variable){
                    let var_data = Config::variable_data(variable)?;
                    let new_path = PathBuf::from(system_file_path[..dollar_pos].trim())
                        .join(var_data)
                        .join(system_file_path[dollar_pos + slash_pos..].trim_start_matches('/'));
                    path_exists(&new_path)?;
                    Ok(new_path)
                } else {
                    Err(QboxError::ConfigUndefinedVariable(variable.to_string()))
                }
        } else {
            path_exists(path)?;
            Ok(path.to_path_buf())
        }
    }

}

fn read_config(path: PathBuf) -> Result<Config, QboxError>{
    let content = std::fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&content)?;   
    Ok(cfg)
}

#[derive(Debug)]
pub struct Qbox {
    config: Option<Config>,
    qbox_path: PathBuf,
}

impl Qbox {
    pub fn new(name: &str) -> Result<Self, QboxError> {
        let qbox_path = make_qbox_path(name)?;
        if qbox_path.exists() {
            Ok(
                Self {config: None, qbox_path }
            )
        } else {
            Err(
                QboxError::MissingQbox(qbox_path)
            )
        }
    }

    pub fn open(&mut self) -> Result<&Self, QboxError>{
        let config_path = self.qbox_path.join(QBOX_CONFIG_NAME);
        if config_path.exists(){
            let mut readed_config = read_config(config_path)?;
            readed_config.validate()?;
            self.config = Some(readed_config);
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

    pub fn record(&self, version: &str) {
        
    }
}

fn path_exists(path: &Path) -> Result<(), QboxError>{
    if !path.exists(){
        return Err(QboxError::IO(
            io::Error::new(io::ErrorKind::NotFound, format!("path {} does not exist", path.to_str()
                .expect("undefined path"))))
        );
    }
    Ok(())
}