use std::{env, fs::create_dir, path::{PathBuf}};

pub mod init;
pub mod qbox;
pub mod error;
pub mod config;

const QBOX_CONFIG_NAME: &str = "qbox.yaml";
const RESERVED_KEYWORDS: [&str; 1] = ["backup"];
const V_BACKUP_NAME: &str = "backup";

pub fn data_dir() -> PathBuf {
    let mut home_fir = env::var("HOME").map(PathBuf::from).expect("home env variable not found");
    home_fir.push(".local/share/qbox");
    
    if !home_fir.exists(){
        create_dir(&home_fir).expect("error create qbox directory");
    }

    home_fir
}