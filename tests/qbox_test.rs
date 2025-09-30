use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use qbox::{fd, qb::{self, data_dir}};
use tempfile::{self, tempdir, TempDir};

struct TempQbox {
    _tmp: TempDir,
    path: PathBuf,
}

fn temp_qbox() -> TempQbox{
    let base = temp_boxes();
    let result_make = qb::qbox::make("Q", base.path.as_path().to_path_buf());
    fs::copy("tests/qbox.yaml", base.path.join("boxes/qbox_Q/qbox.yaml")).unwrap();
    assert!(result_make.is_ok(), "expected Ok, but got {:?}", result_make);
    base
}

fn temp_boxes() -> TempQbox{
    let tmp = tempdir().unwrap();
    let base = tmp.path().join("qbox");
    fs::create_dir_all(base.join("boxes")).unwrap();
    TempQbox { _tmp: tmp, path: base }
}

fn make_config() -> qb::qbox::Config{
    let mut map: HashMap<PathBuf, String> = HashMap::new();
    map.insert(Path::new("/$HOME/rust_projects/vanilla/qbox/tests/source").to_path_buf(), "/$HOME/rust_projects/vanilla/qbox/tests/target".to_string());
    qb::qbox::Config {
        make_dir: true,
        files: vec![map],
        excludes: vec![Path::new("/$HOME/rust_projects/vanilla/qbox/tests/source/ex").to_path_buf()]
    }
}

#[test]
fn init_test(){
    let result = qb::init::init(data_dir());
    assert!(result.is_ok(), "expected Ok, but got {:?}", result);
}

#[test]
fn get_boxes_path_test(){
    let base = temp_boxes();
    let path = qb::qbox::get_boxes_path(base.path.as_path().to_path_buf());
    assert_eq!(path, base.path.join("boxes"));
}

#[test]
fn make_qbox_path_test(){
    let base = temp_boxes();
    let result = qb::qbox::make_qbox_path("tee", base.path.as_path().to_path_buf());
    assert!(result.is_ok(), "expected Ok, but got {:?}", result);

    let path = result.unwrap();
    assert_eq!(path, base.path.join("boxes/qbox_tee"));
}

#[test]
fn make_test(){
    let result = qb::qbox::make("Q", temp_boxes().path);
    assert!(result.is_ok(), "expected Ok, but got {:?}", result);
}

#[test]
fn delete_test(){
    let result = qb::qbox::delete("Q", temp_qbox().path, true);
    assert!(result.is_ok(), "expected Ok, but got {:?}", result);    
}

#[test]
fn read_config_test(){
    let base = temp_qbox();
    let config = qb::qbox::read_config(base.path.join("boxes/qbox_Q/qbox.yaml")).unwrap();

    let expect_config = make_config();
    assert_eq!(config, expect_config);
}

#[test]
fn config_validate_test(){
    let mut config = make_config();
    let result = config.validate();
    assert!(result.is_ok(), "expected Ok, but got {:?}", result) 
}

fn open_qbox() -> (TempQbox, qb::qbox::Qbox){
    let base = temp_qbox();
    let qbox = qb::qbox::Qbox::new("Q", base.path.as_path().to_path_buf());
    assert!(qbox.is_ok(), "expected Ok, but got {:?}", qbox);
    let mut u_qbox = qbox.unwrap();
    let opened_qbox = u_qbox.open();
    assert!(opened_qbox.is_ok(), "expected Ok, but got {:?}", opened_qbox);
    (base, u_qbox)
}

#[test]
fn open_qbox_test() {
    open_qbox();
}

#[test]
fn qbox_new_remove_version_test(){
    let (base, qbox) = open_qbox();
    let result = qbox.new_version("v1");
    assert!(result.is_ok(), "expected Ok, but got {:?}", result);
    assert!(base.path.join("boxes/qbox_Q/v1").exists(), "version directory not created");
    
    let remove_result = qbox.remove_version("v1", true);
    assert!(remove_result.is_ok(), "expected Ok, but got {:?}", remove_result);
    assert!(!base.path.join("boxes/qbox_Q/v1").exists(), "version directory not removed");
}

fn record_version() -> (TempQbox, qb::qbox::Qbox){
    let (base, qbox) = open_qbox();
    fs::create_dir(base.path.join("boxes/qbox_Q/v1")).unwrap();
    let result_record = qbox.record("v1", true);
    assert!(result_record.is_ok(), "expected Ok, but got {:?}", result_record);
    
    let expected_files = ["f1.txt", "f2.txt"];
    for file in fd::dir::read_all(&base.path.join("boxes/qbox_Q/v1"), None).unwrap() {
        if let Some(file_name) = file.file_name() {
            assert!(expected_files.contains(&file_name.to_str().unwrap()), "source file {} not created", file_name.to_str().unwrap());
        }
    }
    (base, qbox)
}

#[test]
fn qbox_record_test(){
    record_version();
}

#[test]
fn qbox_make_backup_test(){
    let (base, qbox) = open_qbox();
    let result_backup = qbox.make_backup();
    assert!(result_backup.is_ok(), "expected Ok, but got {:?}", result_backup);
    assert!(base.path.join("boxes/qbox_Q/backup").exists(), "backup dir not found");
    
    let mut found = false;
    let files = fd::dir::read_all(&base.path.join("boxes/qbox_Q/backup"), None).unwrap();
    assert!(!files.is_empty(), "files not created");
    for file in files {
        if let Some(file_name) = file.file_name()
            && file_name == "file.txt" {
                found = true;
                break;
            }
    }
    assert!(found, "failed to create backup, files not found");
}

#[test]
fn qbox_apply_test(){
    fd::dir::clear(Path::new("tests/target/tee")).unwrap();
    let (_base, qbox) = record_version();
    let result_apply = qbox.apply("v1", false);
    assert!(result_apply.is_ok(), "expected Ok, but got {:?}", result_apply);

    let expected_file = ["f1.txt", "f2.txt"];
    let files = fd::dir::read_all(Path::new("tests/target/tee"), None).unwrap();
    assert!(files.len() == 2, "files not created");
    for file in files {
        if let Some(file_name) = file.file_name() {
            assert!(expected_file.contains(&file_name.to_str().unwrap()), "failed to apply, files not found");
        }
    }
}