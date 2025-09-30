use std::{collections::HashMap, env, fs, io, path::{Path, PathBuf}};
use crate::{fd, qb::{error::QboxError, QBOX_CONFIG_NAME, RESERVED_KEYWORDS, V_BACKUP_NAME}};
use serde::Deserialize;

const BOX_DIR: &str = "boxes";
pub fn get_boxes_path(data_dir: PathBuf) -> PathBuf {
    data_dir.join(BOX_DIR)
}

pub fn make_qbox_path(name: &str, data_dir: PathBuf) -> io::Result<PathBuf>{
    let path = get_boxes_path(data_dir);
    if !path.exists() {
        return Err(
            io::Error::other("boxes directory not exists")
        );
    }
    let qbox_path = path.join(format!("qbox_{}", name));
    Ok(qbox_path)
}

pub fn make(name: &str, data_dir: PathBuf) -> io::Result<()>{
    let qbox_path = make_qbox_path(name, data_dir)?;
    if !qbox_path.exists() {
        let is_make = fd::dir::make(&qbox_path.to_string_lossy())?;
        if !is_make {
            return Err(
                io::Error::other("error creating qbox directory")
            );
        }
        Ok(())
    } else {
        Err(
            io::Error::other(format!("qbox qbox_{} already exists", name))
        )
    }
}

pub fn delete(name: &str, data_dir: PathBuf, force: bool) -> io::Result<()>{
    let qbox_path = make_qbox_path(name, data_dir)?;
    if qbox_path.exists(){
        let is_delete = fd::dir::delete(&qbox_path.to_string_lossy(), force)?;
        if !is_delete {
            return Err(
                io::Error::other("error deleting qbox directory")
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

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub make_dir: bool,
    pub files: Vec<HashMap<PathBuf, String>>,
    pub excludes: Vec<PathBuf>,
}

impl Config {
    fn new() -> Self{
        Self { make_dir: false, files: Vec::new(), excludes: Vec::new() }
    }

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
                if target_path != "*" && (source_path.ends_with("/") || target_path.ends_with("/")) {
                    return Err(QboxError::IO(
                        io::Error::new(io::ErrorKind::NotFound, "config path must not end with '/'"))
                    );
                }
                let valid_source_path = self.path_validate(source_path, true)?;
                let valid_target_path = if target_path == "*" {
                    valid_source_path.to_string_lossy().to_string()
                } else if self.make_dir{
                    self.path_validate(Path::new(target_path), false)?.to_string_lossy().to_string()
                } else {
                    self.path_validate(Path::new(target_path), true)?.to_string_lossy().to_string()
                };
                valid_map.insert(valid_source_path, valid_target_path);
            }
            valid_files.push(valid_map);
        }
        self.files = valid_files;
        let mut valid_excludes: Vec<PathBuf> = Vec::new();
        for exclude_path in &self.excludes {
            valid_excludes.push(self.path_validate(exclude_path, true)?);
        }
        self.excludes = valid_excludes;
        Ok(())
    }

    fn path_validate(&self, path: &Path, check_exists: bool) -> Result<PathBuf, QboxError>{
        let system_file_path = path.to_string_lossy();
        if let Some(dollar_pos) = system_file_path.find("$")
            && let Some(slash_pos) = system_file_path[dollar_pos..].find("/"){
                let variable = &system_file_path[dollar_pos+1..dollar_pos+1 + slash_pos-1];
                if CONFIG_VARIABLES.contains(&variable){
                    let var_data = Config::variable_data(variable)?;
                    let new_path = PathBuf::from(system_file_path[..dollar_pos].trim())
                        .join(var_data)
                        .join(system_file_path[dollar_pos + slash_pos..].trim_start_matches('/'));
                    if check_exists{
                        fd::dir::path_exists(&new_path)?;
                    }
                    Ok(new_path)
                } else {
                    Err(QboxError::ConfigUndefinedVariable(variable.to_string()))
                }
        } else {
            if check_exists{
                fd::dir::path_exists(path)?;
            }
            Ok(path.to_path_buf())
        }
    }

    pub fn excludes_to_str(&self) -> Vec<&str>{
        self.excludes.iter().map(|p| p.to_str().expect("invalid utf-8 in exclude path")).collect()
    }

}

pub fn read_config(path: PathBuf) -> Result<Config, QboxError>{
    let content = std::fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&content)?;   
    Ok(cfg)
}

#[derive(Debug)]
pub struct Qbox {
    config: Config,
    qbox_path: PathBuf,
}

impl Qbox {
    pub fn new(name: &str, data_dir: PathBuf) -> Result<Self, QboxError> {
        let qbox_path = make_qbox_path(name, data_dir)?;
        if qbox_path.exists() {
            Ok(
                Self {config: Config::new(), qbox_path }
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
            self.config = readed_config;
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
            if !fd::dir::make(&version_path.to_string_lossy())? {
                return Err(
                    QboxError::VersionPathError(version_path, "error creating version directory".to_string())
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
            if !fd::dir::delete(&version_path.to_string_lossy(), force)? {
                return Err(
                    QboxError::VersionPathError(version_path, "uerror deleting version directory".to_string())
                );
            }
        } else {
            return Err(
                QboxError::VersionPathError(version_path, "version not exists".to_string())
            );
        }
        Ok(())
    }

    pub fn record(&self, version: &str, force: bool) -> Result<(), QboxError> {
        let version_path = self.qbox_path.join(version);
        if !version_path.exists(){
            return Err(
                QboxError::VersionPathError(version_path, "version not exists".to_string())
            );
        }
        if force{
            fd::dir::clear(&version_path)?;
        }
        for file in &self.config.files {
            for source_path in file.keys(){
                for write_file_path in fd::dir::read_all(source_path, Some(&self.config.excludes_to_str()))? {
                    fd::file::create_in_dir(&write_file_path, &version_path)?;
                }
            }
        }
        Ok(())
    }

    /// Creates files that are stored in the version in the selected directory.
    /// Deletes all files from the selected directory and creates items there that are stored in the version.
    /// IMPORTANT: Only items at the end of the source path will be created. For example:
    /// If the source path is /home/user/temp, only items stored in the “temp” directory will be created; nothing else will be touched.
    pub fn apply(&self, version: &str, force: bool) -> Result<(), QboxError> {
        if version == V_BACKUP_NAME {
            self.apply_backup()?;
            return Ok(());
        }
        let version_path = self.qbox_path.join(version);
        if !version_path.exists(){
            return Err(
                QboxError::VersionPathError(version_path, "version not exists".to_string())
            );
        }
        let mut formatted_v_file_paths: Vec<String> = vec![];
        for v_file_path in fd::dir::read_all(&version_path, None)? {
            let formatted_v_file_path =
                v_file_path
                    .to_string_lossy()
                    .strip_prefix(&*version_path.to_string_lossy())
                    .expect("path is not prefixed by version_path").to_string();
            formatted_v_file_paths.push(formatted_v_file_path);
        }
        for file in &self.config.files {
            for (source_path, target_path) in file {
                let string_source_path = source_path.to_string_lossy();
                for formatted_v_file_path in &formatted_v_file_paths {
                    if formatted_v_file_path.starts_with(&*string_source_path){
                        let real_files = formatted_v_file_path.trim_start_matches(&*string_source_path).trim_start_matches("/");
                        let new_file = Path::new(target_path).join(Path::new(real_files));
                        if let Some(new_file_parent) = new_file.parent(){
                            if new_file_parent.exists() && force{
                                println!("{:?}", new_file_parent);
                                fs::remove_dir_all(new_file_parent)?;
                            }
                            fs::create_dir_all(new_file_parent)?;
                        }
                        fs::copy(formatted_v_file_path, new_file)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn make_backup(&self) -> Result<(), QboxError>{
        let v_backup_path = self.qbox_path.join(V_BACKUP_NAME);
        if !v_backup_path.exists() {
            self.new_version(V_BACKUP_NAME)?;
        }
        fd::dir::clear(&v_backup_path)?;
        for file in &self.config.files {
            for target_dir in file.values() {
                let target_dir_path = Path::new(target_dir);
                if !target_dir_path.exists(){
                    continue;
                }
                for target_file_path in fd::dir::read_all(target_dir_path, Some(&self.config.excludes_to_str()))? {
                    fd::file::create_in_dir(&target_file_path, &v_backup_path)?;
                }
            }
        }
        Ok(())
    }
    
    /// Apply backup.
    /// The backup directory contains directories that are absolute paths to files.
    /// The algorithm formats them so that they are perceived as absolute paths and 
    /// creates files from the backup using these paths.
    fn apply_backup(&self) -> Result<(), QboxError> {
        let v_backup_path = self.qbox_path.join(V_BACKUP_NAME);
        if !v_backup_path.exists() {
            return Err(
                QboxError::VersionPathError(v_backup_path, "backup not exists".to_string())
            );
        }
        for file_path in fd::dir::read_all(&v_backup_path, None)? {
            let file_path_str = file_path.to_string_lossy();
            let backup_file_path = file_path_str.trim_start_matches(&*v_backup_path.to_string_lossy());
            if let Some(target_dir) = Path::new(backup_file_path).parent()
                && !target_dir.exists() {
                    fs::create_dir_all(target_dir)?;
                }
            fs::copy(&file_path, backup_file_path)?;
        }
        Ok(())
    }
}

pub fn check_keywords(name: &str) -> Result<(), QboxError>{
    if RESERVED_KEYWORDS.contains(&name){
        return Err(QboxError::ReservedKeyword(name.to_string()));
    }
    Ok(())
}