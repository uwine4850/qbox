use std::{fs, io, path::{Path, PathBuf}};
use crate::{fd, qb::{config::{read_config, Config}, error::QboxError, QBOX_CONFIG_NAME, RESERVED_KEYWORDS, V_BACKUP_NAME}};

const BOX_DIR: &str = "boxes";
/// Creates a complete path to the boxes.
pub fn get_boxes_path(data_dir: PathBuf) -> PathBuf {
    data_dir.join(BOX_DIR)
}

/// Creates the full path to qbox.
/// Uses the passed directory path as the start, formats the qbox directory name.
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

/// Creates a qbox.
/// Error if such a qbox already exists.
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

/// Deleting qbox.
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