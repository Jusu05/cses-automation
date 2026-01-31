use std::vec;
use thirtyfour::prelude::*;
use scraper::{Html, Selector};

use crate::settings::Settings;
use crate::database::Database;
struct CsesConnection {
    settings: Settings,
    database: Database,
    driver: WebDriver,
}

impl CsesConnection {
    fn new(settings: Settings, database: Database) -> Self {

        let driver = WebDriver::new();

        CsesConnection {
            settings,
            database,
            driver,
        }
    }

    pub fn login(&self) {
    }

    pub fn load_task(&self) {

    }

    pub fn submit_task(&self) {

    }

    fn get_tasks(&self, tasks_name: &str) -> Vec<&str> {
        Vec::new()
    }
}
