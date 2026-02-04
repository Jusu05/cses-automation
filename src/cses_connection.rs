use crate::database::Database;
use crate::settings::settings::{Settings, SettingsError};
use scraper::{Html, Selector, element_ref::ElementRef};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{thread, time::Duration};
use thirtyfour::prelude::*;

#[derive(Debug)]
pub enum CsesConnectionError {
    WebDriver(thirtyfour::error::WebDriverError),
    Sql(rusqlite::Error),
    Io(std::io::Error),
    Setting(SettingsError),
    ValueError(String),
}

impl From<thirtyfour::error::WebDriverError> for CsesConnectionError {
    fn from(err: thirtyfour::error::WebDriverError) -> Self {
        CsesConnectionError::WebDriver(err)
    }
}

impl From<rusqlite::Error> for CsesConnectionError {
    fn from(err: rusqlite::Error) -> Self {
        CsesConnectionError::Sql(err)
    }
}

impl From<std::io::Error> for CsesConnectionError {
    fn from(err: std::io::Error) -> Self {
        CsesConnectionError::Io(err)
    }
}

impl From<std::num::ParseIntError> for CsesConnectionError {
    fn from(err: std::num::ParseIntError) -> Self {
        CsesConnectionError::ValueError(format!("{:?}", err))
    }
}

impl From<SettingsError> for CsesConnectionError {
    fn from(err: SettingsError) -> Self {
        CsesConnectionError::Setting(err)
    }
}

pub struct CsesConnection {
    settings: Settings,
    database: Database,
    driver: WebDriver,
    child: std::process::Child,
}

impl Drop for CsesConnection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl CsesConnection {
    pub async fn new(path: &PathBuf) -> Result<Self, CsesConnectionError> {
        let settings = Settings::new(path.join("settings.ini"));
        let database = Database::new(path.join("tasks.db"))?;
        let webrdriver = settings.get_webdriver_path()?;
        let child = Command::new(webrdriver)
            .arg("--port")
            .arg("4444")
            .arg("--headless")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        let url = settings.get_cses_url()?;
        driver.goto(format!("{}/list/", url)).await?;
        Ok(Self {
            driver,
            settings,
            database,
            child,
        })
    }

    async fn login(&self) -> Result<(), CsesConnectionError> {
        let url = self.settings.get_cses_url()?;
        self.driver.goto(format!("{}/list/", url)).await?;

        let accaunt = self
            .driver
            .find(By::Css("body > div.header > div > div > a.account"))
            .await?;
        accaunt.click().await?;

        let username_field = self.driver.find(By::Id("username")).await.ok();
        let password_field = self.driver.find(By::Id("password")).await.ok();

        if let (Some(username_field), Some(password_field)) = (username_field, password_field) {
            let (username, password) = self.settings.get_username_and_password()?;
            username_field.send_keys(username).await?;
            password_field.send_keys(password).await?;
            let sigin_button = self.driver.find(By::Css("#content-area > div.login-align > div > form > input.btn.btn-primary.login-form-button")).await?;
            sigin_button.click().await?;
        }
        Ok(())
    }

    pub async fn load_task(&self) -> Result<(), CsesConnectionError> {
        let tasks = self.get_tasks().await?;

        for task in tasks {
            let url = self.settings.get_cses_url()?;
            self.driver.goto(format!("{}/task/{}", url, task.0)).await?;
            let page_source = self.driver.source().await.unwrap();
            let html = Html::parse_document(&page_source);
            let content_selector = Selector::parse("div.md").unwrap();
            let content = html.select(&content_selector).next().unwrap();

            if let Some(file_name) = self.task_has_file_name(content) {
                self.database.set_file_name_by_id(&file_name, &task.0)?;
            } else {
                continue;
            }

            let code_selector =
                Selector::parse("pre.resize-horizontal prettyprint lang-python prettyprinted")
                    .unwrap();

            let code_elment = content.select(&code_selector).next();
            let code = if let Some(code_element) = code_elment {
                code_element.text().next()
            } else {
                None
            };

            let dir = self.settings.get_working_dir()?;
            let path = PathBuf::from(dir).join(task.2).join(task.1);
            let mut file = File::create(path)?;
            for child in content.child_elements() {
                if let Some(code_elment) = code_elment {
                    if code_elment == child {
                        continue;
                    }
                }
                let s = child.text().collect::<String>();
                writeln!(file, "#{}", s)?;
            }

            if let Some(code) = code {
                writeln!(file, "{}", code)?;
            }
        }

        Ok(())
    }

    async fn get_tasks(&self) -> Result<Vec<(i32, String, String)>, CsesConnectionError> {
        let loaded_tasks = self.database.get_all_task_names()?;
        let url = self.settings.get_cses_url()?;
        self.driver.goto(format!("{}/list/", url)).await?;

        let html = Html::parse_document(&self.driver.source().await.unwrap());
        let ul_selector = Selector::parse("ul.task-list").unwrap();
        let h2_selector = Selector::parse("h2").unwrap();
        let li_selector = Selector::parse("li.task a").unwrap();

        let mut tasks: Vec<(i32, String, String)> = Vec::new();
        for (task_list, heading) in html.select(&ul_selector).zip(html.select(&h2_selector)) {
            for element in task_list.select(&li_selector) {
                let task_name = element.text().collect::<String>();

                if !loaded_tasks.contains(&task_name) {
                    continue;
                }

                let link = element.value().attr("href");

                if let Some(link) = link {
                    let parts: Vec<&str> = link.split("/").collect();
                    let id: i32 = parts.last().unwrap().parse()?;
                    let week: String = heading.text().next().unwrap().to_owned();
                    self.database.add_task(&id, &task_name, &week)?;
                    tasks.push((id, task_name, week));
                }
            }
        }

        Ok(tasks)
    }

    fn list_task_dir(&self) -> Result<HashSet<String>, CsesConnectionError> {
        let path = PathBuf::from(self.settings.get_working_dir()?);
        let weeks = self.database.get_all_weeks()?;

        if weeks.is_empty() {
            return Ok(HashSet::new());
        }

        let mut tasks = HashSet::new();
        for week in weeks {
            let week_path = path.join(&week);

            if week_path.is_dir() {
                for entry in fs::read_dir(&week_path)? {
                    let entry = entry?;
                    let file_name = entry.file_name().into_string().map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Invalid UTF-8 sequence")
                    })?;
                    tasks.insert(file_name);
                }
            }
        }

        Ok(tasks)
    }

    fn task_has_file_name(&self, html: ElementRef<'_>) -> Option<String> {
        let code_selector = Selector::parse("code").unwrap();
        for code_block in html.select(&code_selector) {
            let text: String = code_block.text().collect();
            if text.ends_with(".py") {
                return Some(text);
            }
        }
        None
    }

    pub async fn submit_task(&self, file: String) -> Result<(), CsesConnectionError> {
        let task_dir = self.list_task_dir()?;

        if !task_dir.contains(&file) {
            return Err(CsesConnectionError::ValueError(
                "Task no downloaded".to_owned(),
            ));
        }

        let id = self.database.get_id_by_file_name(&file)?;
        let week = self.database.get_week_by_file_name(&file)?;
        let dir = self.settings.get_working_dir()?;
        let path = PathBuf::from(dir).join(week).join(&file);

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
        let skip = lines
            .first()
            .map_or(true, |first_line| !first_line.starts_with('#'));

        if !skip {
            for (i, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    lines = lines[i + 1..].to_vec();
                    break;
                }
            }

            let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;

            for line in lines {
                writeln!(file, "{}", line)?;
            }
        }

        self.login().await?;

        let url = self.settings.get_cses_url()?;

        self.driver.goto(format!("{}/submit/{}", url, id)).await?;
        thread::sleep(Duration::from_millis(500));
        let upload = self.driver.find(By::Name("file")).await?;
        thread::sleep(Duration::from_millis(500));
        let s = path.to_str().unwrap();
        upload.send_keys(s).await?;
        thread::sleep(Duration::from_millis(500));
        let submit = self
            .driver
            .find(By::Css(
                ".content > form:nth-child(1) > p:nth-child(6) > input:nth-child(1)",
            ))
            .await?;
        thread::sleep(Duration::from_millis(500));
        submit.click().await?;

        Ok(())
    }

    pub async fn task_solution_result(
        &self,
        task_id: &i32,
    ) -> Result<String, CsesConnectionError> {
        let url = self.settings.get_cses_url()?;
        self.driver.goto(format!("{}/view/{}/", url, task_id)).await?;
        let solution = self.driver.find(By::Css( "body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr > td:nth-child(4) > a")).await?;
        solution.click().await?;

        let result = self.driver.find(By::Css("body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr:nth-child(6) > td:nth-child(2) > span")).await?;
        let mut text = result.text().await?;

        if text == "TEST FAILED" {
            let current_url = self.driver.current_url().await?.to_string();
            let error = format!("\n find reason from: {}", &current_url);
            text += &error;
        }

        return Ok(text);
    }
}
