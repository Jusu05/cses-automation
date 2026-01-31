pub struct Settings;

impl Settings {
    pub fn new() -> Self {
        Settings
    }

    pub fn get_webdriver_path(&self) -> String {
        "".to_owned()
    }

    pub fn get_cses_url(self) -> String {
        "".to_owned()
    }

    pub fn get_username_and_password(&self) -> (String, String) {
        let username = "".to_owned();
        let password = "".to_owned();

        (username, password)
    }

    pub fn get_working_dir(&self) -> String {
        "".to_owned()
    }

    pub fn set_webdriver_path(&self, driver: &str) {

    }

    pub fn set_cses_url(&self, url: &str) {

    }

    pub fn set_password(&self, password: &str) {

    }

    pub fn set_username(&self, username: &str) {

    }

    pub fn set_working_dir(&self, dir: &str) {

    }
}
