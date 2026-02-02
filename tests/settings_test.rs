use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use cses_automation::settings::Settings;

fn empty_temp_file() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.ini");
    fs::write(&file, "").unwrap();
    (dir, file)
}

fn temp_geckodriver_exe(dir: &Path) -> PathBuf {
    let p = dir.join("geckodriver.exe");
    fs::write(&p, "").unwrap();
    p
}

fn temp_geckodriver(dir: &Path) -> PathBuf {
    let p = dir.join("geckodriver");
    fs::write(&p, "").unwrap();
    p
}

fn filled_temp_file() -> (TempDir, PathBuf, PathBuf) {
    let dir = TempDir::new().unwrap();
    let geckodriver = temp_geckodriver_exe(dir.path());

    let ini = format!(
        r#"
[general]
webdriver = {}
cses_url = www.cses.fi

[user]
username = user
password = 70617373776f7264

[system]
working_dir = {}
"#,
        geckodriver.display(),
        dir.path().display()
    );

    let file = dir.path().join("test.ini");
    fs::write(&file, ini.trim()).unwrap();

    (dir, file, geckodriver)
}

#[test]
fn test_set_webdriver_path() {
    let (_dir, ini) = empty_temp_file();
    let driver = temp_geckodriver_exe(ini.parent().unwrap());

    let s = Settings::new(ini);
    s.set_webdriver_path(&driver);

    let result = s.get_webdriver_path().unwrap();
    assert_eq!(driver.to_str().unwrap(), result);
}

#[test]
fn test_edit_webdriver_path() {
    let (_dir, ini, _) = filled_temp_file();
    let driver = temp_geckodriver(ini.parent().unwrap());

    let s = Settings::new(ini);
    s.set_webdriver_path(&driver);

    let result = s.get_webdriver_path().unwrap();
    assert_eq!(driver.to_str().unwrap(), result);
}

#[test]
fn test_set_webdriver_path_error_if_not_geckodriver() {
    let (_dir, ini, _) = filled_temp_file();
    let s = Settings::new(ini);
    let webdriver = PathBuf::from("/home/test/ChromeDriver");
    assert!(s.set_webdriver_path(&webdriver).is_err());
}

#[test]
fn test_set_webdriver_path_file_not_found() {
    let (_dir, ini, _) = filled_temp_file();
    let s = Settings::new(ini);

    let webdriver = PathBuf::from("/home/test/geckodriver");
    assert!(s.set_webdriver_path(&webdriver).is_err());
}

#[test]
fn test_set_cses_url() {
    let (_dir, ini) = empty_temp_file();
    let s = Settings::new(ini);

    s.set_cses_url("cses.fi").unwrap();
    assert_eq!("cses.fi", s.get_cses_url().unwrap());
}

#[test]
fn test_edit_cses_url() {
    let (_dir, ini, _) = filled_temp_file();
    let s = Settings::new(ini);

    s.set_cses_url("cses.fi/test").unwrap();
    assert_eq!("cses.fi/test", s.get_cses_url().unwrap());
}

#[test]
fn test_set_cses_url_value_error() {
    let (_dir, ini, _) = filled_temp_file();
    let s = Settings::new(ini);

    assert!(s.set_cses_url("google.com").is_err());
}

#[test]
fn test_set_password_and_username() {
    let (_dir, ini) = empty_temp_file();
    let s = Settings::new(ini);

    s.set_password("password").unwrap();
    s.set_username("user").unwrap();

    let (u, p) = s.get_username_and_password().unwrap();
    assert_eq!("user", u);
    assert_eq!("password", p);
}

#[test]
fn test_edit_password_and_username() {
    let (_dir, ini, _) = filled_temp_file();
    let s = Settings::new(ini);

    s.set_password("password1").unwrap();
    s.set_username("user1").unwrap();

    let (u, p) = s.get_username_and_password().unwrap();
    assert_eq!("user1", u);
    assert_eq!("password1", p);
}

#[test]
fn test_set_workdir_path() {
    let (_dir, ini) = empty_temp_file();
    let temp = TempDir::new().unwrap();

    let s = Settings::new(ini);
    s.set_working_dir(temp.path().into()).unwrap();

    assert_eq!(temp.path().to_str().unwrap(), s.get_working_dir().unwrap());
}

#[test]
fn test_set_workdir_str() {
    let (_dir, ini) = empty_temp_file();
    let temp = TempDir::new().unwrap();

    let s = Settings::new(ini);
    s.set_working_dir(temp.path().to_str().unwrap().into()).unwrap();

    assert_eq!(temp.path().to_str().unwrap(), s.get_working_dir().unwrap());
}

#[test]
fn test_set_workdir_error_if_file() {
    let (_dir, ini, _) = filled_temp_file();
    let a = ini.clone();
    let s = Settings::new(ini);

    assert!(s.set_working_dir(a).is_err());
}
