pub mod init;
pub mod qbox;
pub mod error;

const QBOX_CONFIG_NAME: &str = "qbox.yaml";
const RESERVED_KEYWORDS: [&str; 1] = ["backup"];
const V_BACKUP_NAME: &str = "backup";