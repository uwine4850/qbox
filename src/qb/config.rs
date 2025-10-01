use std::{collections::HashMap, env, io, path::{Path, PathBuf}};
use crate::{fd, qb::{error::QboxError}};
use serde::Deserialize;


const CONFIG_VARIABLES: [&str; 1] = ["HOME"];

#[derive(Debug, Deserialize, PartialEq)]
#[derive(Default)]
pub struct Config {
    pub make_dir: bool,
    pub files: Vec<HashMap<PathBuf, String>>,
    pub excludes: Vec<PathBuf>,
}


impl Config {
    pub fn new() -> Self{
        Self::default()
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
                self.validate_path_style(target_path, &source_path.to_string_lossy())?;
                let valid_source_path = self.format_path(source_path, true)?;
                let valid_target_path = if target_path == "*" {
                    valid_source_path.to_string_lossy().to_string()
                } else if self.make_dir{
                    self.format_path(Path::new(target_path), false)?.to_string_lossy().to_string()
                } else {
                    self.format_path(Path::new(target_path), true)?.to_string_lossy().to_string()
                };
                valid_map.insert(valid_source_path, valid_target_path);
            }
            valid_files.push(valid_map);
        }
        self.files = valid_files;
        self.format_exclude_paths()?;
        Ok(())
    }

    /// Path formatting. Checking path style and applying variables.
    fn format_path(&self, path: &Path, check_exists: bool) -> Result<PathBuf, QboxError>{
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

    fn format_exclude_paths(&mut self) -> Result<(), QboxError> {
        let mut valid_excludes: Vec<PathBuf> = Vec::new();
        for exclude_path in &self.excludes {
            valid_excludes.push(self.format_path(exclude_path, true)?);
        }
        self.excludes = valid_excludes;
        Ok(())
    }

    pub fn excludes_to_str(&self) -> Vec<&str>{
        self.excludes.iter().map(|p| p.to_str().expect("invalid utf-8 in exclude path")).collect()
    }

    fn validate_path_style(&self, target_path: &str, source_path: &str) -> Result<(), QboxError>{
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
        Ok(())
    }

}

pub fn read_config(path: PathBuf) -> Result<Config, QboxError>{
    let content = std::fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&content)?;   
    Ok(cfg)
}