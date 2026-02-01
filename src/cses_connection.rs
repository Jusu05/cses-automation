use crate::database::Database;
use crate::settings::Settings;
use scraper::{Html, Selector, element_ref::ElementRef};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{thread, time::Duration};
use thirtyfour::prelude::*;
struct CsesConnection {
    settings: Settings,
    database: Database,
    driver: WebDriver,
}

impl CsesConnection {
    pub async fn new(settings: Settings, database: Database) -> WebDriverResult<Self> {
        let webrdriver = settings.get_webdriver_path();
        Command::new(webrdriver)
            .arg("--port")
            .arg("4444")
            .arg("--headless")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start GeckoDriver");

        let caps = DesiredCapabilities::firefox();
        let driver = WebDriver::new("http://localhost:4444", caps).await?;
        let url = settings.get_cses_url();
        driver.goto(format!("{}/list/", url)).await?;

        Ok(Self {
            driver,
            settings,
            database,
        })
    }

    async fn login(&self) -> WebDriverResult<()> {
        let url = self.settings.get_cses_url();
        self.driver.goto(format!("{}/list/", url)).await?;

        let accaunt = self
            .driver
            .find(By::Css("body > div.header > div > div > a.account"))
            .await?;
        accaunt.click();

        let username_field = self.driver.find(By::Id("username")).await.ok();
        let password_field = self.driver.find(By::Id("password")).await.ok();

        if let (Some(username_field), Some(password_field)) = (username_field, password_field) {
            let (username, password) = self.settings.get_username_and_password();
            username_field.send_keys(username);
            password_field.send_keys(password);
            let sigin_button = self.driver.find(By::Css("#content-area > div.login-align > div > form > input.btn.btn-primary.login-form-button")).await?;
            sigin_button.click();
        }
        Ok(())
    }

    pub async fn load_task(&self) -> WebDriverResult<()> {
        let tasks = self.get_tasks().await?;

        for task in tasks {
            let url = self.settings.get_cses_url();
            self.driver.goto(format!("{}/task/{}", url, task.0)).await?;
            let page_source = self.driver.source().await.unwrap();
            let html = Html::parse_document(&page_source);
            let content_selector = Selector::parse("div.md").unwrap();
            let content = html.select(&content_selector).next().unwrap();

            if !self.task_has_file_name(content) {
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

            let dir = self.settings.get_working_dir();
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
                writeln!(file, "{}", code);
            }
        }

        Ok(())
    }

    async fn get_tasks(&self) -> WebDriverResult<Vec<(String, String, String)>> {
        let loaded_tasks = self.list_task_dir()?;
        let url = self.settings.get_cses_url();
        self.driver.goto(format!("{}/list/", url)).await?;

        let html = Html::parse_document(&self.driver.source().await.unwrap());
        let ul_selector = Selector::parse("ul.task-list").unwrap();
        let h2_selector = Selector::parse("h2").unwrap();
        let li_selector = Selector::parse("li.task a").unwrap();

        let mut tasks_ids: Vec<(String, String, String)> = Vec::new();
        for (task_list, heading) in html.select(&ul_selector).zip(html.select(&h2_selector)) {
            for element in task_list.select(&li_selector) {
                let task_name = element.text().collect::<String>();

                if !loaded_tasks.contains(&task_name) {
                    continue;
                }

                let link = element.value().attr("href");

                if let Some(link) = link {
                    let parts: Vec<&str> = link.split("/").collect();
                    let id = parts.last().unwrap();
                    let week = heading.text().next().unwrap().to_owned();
                    tasks_ids.push((id.to_string(), task_name, week));
                }
            }
        }

        Ok(tasks_ids)
    }

    fn list_task_dir(&self) -> Result<HashSet<String>, io::Error> {
        let path = PathBuf::from(self.settings.get_working_dir());
        let weeks = self.database.get_all_weeks();

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

    fn task_has_file_name(&self, html: ElementRef<'_>) -> bool {
        let code_selector = Selector::parse("code").unwrap();
        for code_block in html.select(&code_selector) {
            let text: String = code_block.text().collect();
            if text.ends_with(".py") {
                return true;
            }
        }
        false
    }

    pub async fn submit_task(&self, file: String) -> WebDriverResult<()> {
        let id = self.database.get_id_by_file_name(&file);
        let week = self.database.get_week_by_file_name(&file);
        let dir = self.settings.get_working_dir();
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

        self.login().await;

        let url = self.settings.get_cses_url();

        self.driver.goto(format!("{}/submit/{}", url, id));
        thread::sleep(Duration::from_millis(500));
        let upload = self.driver.find(By::Name("file")).await?;
        thread::sleep(Duration::from_millis(500));
        let s = path.to_str().unwrap();
        upload.send_keys(s);
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
}
