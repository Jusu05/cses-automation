use configparser::ini::Ini;
use std::path::PathBuf;

pub enum SettingsError {
    SettingNotFuond(String),
    ValueError(String),
    ParseError(String),
    ReadWriteError,
}

impl From<std::io::Error> for SettingsError {
    fn from(_: std::io::Error) -> Self {
        SettingsError::ReadWriteError
    }
}

pub struct Settings {
    file: PathBuf,
}

impl Settings {
    pub fn new(file: PathBuf) -> Self {
        Settings { file }
    }

    pub fn get_webdriver_path(&self) -> Result<String, SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        match config.get("general", "webdriver") {
            Some(s) => {
                if s.is_empty() {
                    return Err(SettingsError::SettingNotFuond(
                        "Setting webriver is not set".to_owned(),
                    ));
                }

                Ok(s)
            }
            None => Err(SettingsError::SettingNotFuond(
                "Setting webriver is not set".to_owned(),
            )),
        }
    }

    pub fn get_cses_url(&self) -> Result<String, SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        match config.get("general", "cses_url") {
            Some(s) => {
                if s.is_empty() {
                    return Err(SettingsError::SettingNotFuond(
                        "Setting cses url is not set".to_owned(),
                    ));
                }
                Ok(s)
            }
            None => Err(SettingsError::SettingNotFuond(
                "Setting cses url is not set".to_owned(),
            )),
        }
    }

    pub fn get_username_and_password(&self) -> Result<(String, String), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }

        let mut username = String::new();
        match config.get("user", "username") {
            Some(s) => {
                if s.is_empty() {
                    return Err(SettingsError::SettingNotFuond(
                        "Setting username is not set".to_owned(),
                    ));
                }
                username.push_str(&s);
            }
            None => {
                return Err(SettingsError::SettingNotFuond(
                    "Setting username is not set".to_owned(),
                ));
            }
        }

        let mut password = String::new();
        match config.get("user", "password") {
            Some(hex) => {
                if hex.is_empty() {
                    return Err(SettingsError::SettingNotFuond(
                        "Setting password is not set".to_owned(),
                    ));
                }
                password.push_str(&self.hex_to_string(&hex)?);
            }
            None => {
                return Err(SettingsError::SettingNotFuond(
                    "Setting password is not set".to_owned(),
                ));
            }
        }

        Ok((username, password))
    }

    fn hex_to_string(&self, hex: &str) -> Result<String, SettingsError> {
        let bytes: Result<Vec<u8>, std::num::ParseIntError> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect();

        if bytes.is_err() {
            return Err(SettingsError::ParseError(format!(
                "Hex cannot be parsed to bytes"
            )));
        }

        String::from_utf8(bytes.unwrap()).map_err(|_| {
            SettingsError::ParseError(format!("Hex's {} bytes cannot be parsed to utf-8", hex))
        })
    }

    pub fn get_working_dir(&self) -> Result<String, SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        match config.get("system", "working_dir") {
            Some(s) => {
                if s.is_empty() {
                    return Err(SettingsError::SettingNotFuond(
                        "Setting working dir is not set".to_owned(),
                    ));
                }
                Ok(s)
            }
            None => Err(SettingsError::SettingNotFuond(
                "Setting working dir is not set".to_owned(),
            )),
        }
    }

    pub fn set_webdriver_path(&self, driver: &PathBuf) -> Result<(), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }

        if !driver.ends_with("geckodriver.exe") || !driver.ends_with("geckodriver") {
            return Err(SettingsError::ValueError(
                "webdriver is not firefoxs geckodriver".to_owned(),
            ));
        }
        let s = driver.to_str().unwrap().to_owned();
        config.set("general", "webdriver", Some(s));
        config.write(&self.file)?;
        Ok(())
    }

    pub fn set_cses_url(&self, url: &str) -> Result<(), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }

        if !url.contains("cses.fi") {
            return Err(SettingsError::ValueError(
                "domain is not cses.fi".to_owned(),
            ));
        }

        let url = url.trim();
        let url = url.strip_suffix("/list/").unwrap();
        let url = url.strip_suffix("/").unwrap();
        config.set("general", "cses_url", Some(url.to_owned()));
        config.write(&self.file)?;
        Ok(())
    }

    pub fn set_password(&self, password: &str) -> Result<(), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        config.set(
            "user",
            "password",
            Some(self.string_to_hex(password).to_owned()),
        );
        config.write(&self.file)?;
        Ok(())
    }

    fn string_to_hex(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len() * 2);
        for b in s.as_bytes() {
            use std::fmt::Write;
            write!(&mut out, "{:02x}", b).unwrap();
        }
        out
    }

    pub fn set_username(&self, username: &str) -> Result<(), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        config.set("user", "username", Some(username.to_owned()));
        config.write(&self.file)?;
        Ok(())
    }

    pub fn set_working_dir(&self, dir: PathBuf) -> Result<(), SettingsError> {
        let mut config = Ini::new();
        let result = config.load(&self.file);
        if result.is_err() {
            return Err(SettingsError::ReadWriteError);
        }
        if !dir.exists() {
            return Err(SettingsError::ValueError(
                "working dir does not exist".to_owned(),
            ));
        }
        if !dir.is_dir() {
            return Err(SettingsError::ValueError(
                "working dir is not dir".to_owned(),
            ));
        }
        config.set(
            "system",
            "working_dir",
            Some(dir.to_str().unwrap().to_string()),
        );
        config.write(&self.file)?;
        Ok(())
    }
}
