use crate::cses_connection::{CsesConnection, CsesConnectionError};
use crate::database::{Database, TaskStatus};
use crate::settings::settings::{Settings, SettingsError};
use std::{env, path::PathBuf, sync::LazyLock};

static DEVELOPMENT_MODE: LazyLock<bool> = LazyLock::new(|| env::var("DEVELOPMENT").is_ok());

pub struct Cli {
    settings: Settings,
    database: Database,
    path: PathBuf,
    cses_connection: Option<CsesConnection>,
}

impl Cli {
    pub fn new(path: PathBuf) -> Option<Self> {
        let settings = Settings::new(path.join("settings.ini"));
        let database = Database::new(path.join("tasks.db"));

        if database.is_err() {
            println!("Database cannot be create");
            return None;
        }
        let database = database.unwrap();
        Some(Cli {
            settings,
            database,
            path,
            cses_connection: None,
        })
    }

    pub async fn main_loop(&mut self) {
        let args: Vec<String> = env::args().skip(1).collect();
        if args.is_empty() {
            self.help(true, true, true, true);
            return;
        }

        if args[0] == "--help" || args[0] == "-h" {
            self.help(true, true, true, true);
            return;
        }

        match args[0].as_str() {
            "submit" => {
                if args[1] == "--help" || args[1] == "-h" {
                    self.help(true, false, false, false);
                    return;
                }
                self.handle_submit(&args[1]).await;
                return;
            }
            "download" => {
                if args.len() > 1 {
                    self.help(false, true, false, false);
                    return;
                }
                self.handle_download().await;
                return;
            }
            "url" => {
                if args[1] == "--help" || args[1] == "-h" {
                    self.help(false, false, false, true);
                    return;
                }
                self.handle_url(&args[1]);
                return;
            }
            "settings" => {
                if args[1] == "--help" || args[1] == "-h" {
                    self.help(false, false, true, false);
                    return;
                }
                self.handle_settings(args);
            }
            _ => {
                self.help(true, true, true, true);
            }
        }
        if let Some(connection) = self.cses_connection.as_mut() {
            connection.close();
        }
    }

    fn help(&self, submit: bool, download: bool, settings: bool, url: bool) {
        let mut helptext = String::new();
        if submit && download && settings && url {
            helptext += "Usage: program <command>\n\n";
            helptext += "list of commands:\n";
        }
        if submit {
            helptext += "submit <file> - file name that will be submitted\n";
        }
        if url {
            helptext += "url <file> - file name that shows excise page\n";
        }
        if download {
            helptext += "download - downloads all not yet downloaded excises\n";
        }
        if settings {
            helptext += "settings <arguments...> - settings for program\n\n";
            helptext += "    list of arguments\n";
            helptext += "    --webdriver <driver> - set path to browser webdriver\n";
            helptext += "    --url <url> - set url to cses website\n";
            helptext += "    --dir <path> - set directory where files will be downloaded\n";
            helptext += "    --username <username> - set mooc username\n";
            helptext += "    --password <password> - set mooc password";
        }
        println!("{}", helptext)
    }

    fn handle_settings(&self, args: Vec<String>) {
        for pair in args.iter().skip(1).collect::<Vec<&String>>().chunks(2) {
            match pair[0].as_str() {
                "--webdriver" => {
                    let webdriver = PathBuf::from(pair[1]);
                    if let Err(e) = self.settings.set_webdriver_path(&webdriver) {
                        match e {
                            SettingsError::ReadWriteError => {
                                println!("Can't read settings file");
                            }
                            SettingsError::ValueError(_) => {
                                println!("only supported driver is firefox geckodriver");
                            }
                            SettingsError::FileNotFoundError(_) => {
                                println!("You need to download firefox's webdriver");
                                println!(
                                    "It can be here https://github.com/mozilla/geckodriver/releases"
                                );
                                println!("And set it's path to setting");
                            }
                            e => {
                                if *DEVELOPMENT_MODE {
                                    print!("{:?}", e);
                                }
                            }
                        }
                    }
                }
                "--url" => {
                    if let Err(e) = self.settings.set_cses_url(args[1].as_str()) {
                        match e {
                            SettingsError::ValueError(s) => {
                                println!("{}", s)
                            }
                            e => {
                                if *DEVELOPMENT_MODE {
                                    println!("{:?}", e);
                                }
                            }
                        }
                    }
                }
                "--dir" => {
                    let dir = PathBuf::from(pair[1]);
                    if let Err(e) = self.settings.set_working_dir(dir) {
                        match e {
                            SettingsError::FileNotFoundError(s) => {
                                println!("{}", s);
                            }
                            e => {
                                if *DEVELOPMENT_MODE {
                                    println!("{:?}", e);
                                }
                            }
                        }
                    }
                }
                "--username" => {
                    if let Err(e) = self.settings.set_username(pair[1]) {
                        if *DEVELOPMENT_MODE {
                            println!("{:?}", e);
                        }
                    }
                }
                "--password" => {
                    if let Err(e) = self.settings.set_password(pair[1]) {
                        if *DEVELOPMENT_MODE {
                            println!("{:?}", e);
                        }
                    }
                }
                _ => {
                    self.help(true, true, true, true);
                }
            }
        }
    }

    async fn create_cses_connection(&mut self) -> Option<()> {
        if self.cses_connection.is_none() {
            match CsesConnection::new(&self.path).await {
                Ok(connection) => {
                    self.cses_connection = Some(connection);
                }
                Err(e) => {
                    if *DEVELOPMENT_MODE {
                        println!("{:?}", e);
                    }
                    return None;
                }
            };
        }
        Some(())
    }

    async fn handle_download(&mut self) {
        if let None = self.create_cses_connection().await {
            println!("cannot connect to cses website");
            return;
        }

        if let Err(CsesConnectionError::Setting(e)) =
            self.cses_connection.as_ref().unwrap().load_tasks().await
        {
            if let SettingsError::SettingNotFound(s) = e {
                match s.as_str() {
                    "webdriver is not firefox's geckodriver" => {
                        println!("Webdriver path need to specified by settings --webdriver")
                    }
                    "working directory is not defined" => {
                        println!("Working dir need to specified by settings --dir")
                    }
                    "Setting cses url is not set" => {
                        println!("Url need to specified by settings --url")
                    }
                    "Setting username is not set" => {
                        println!("Username need to specified by settings --username")
                    }
                    "password is not specified" => {
                        println!("Password need to specified by settings --password")
                    }
                    _ => {}
                }
            } else {
                if *DEVELOPMENT_MODE {
                    println!("{:?}", e);
                }
            }
        }

        if let Some(connection) = self.cses_connection.as_mut() {
            connection.close();
        }
    }

    async fn handle_submit(&mut self, file: &str) {
        let file = if file.contains("/") {
            file.split("/").last().unwrap().to_owned()
        } else if file.contains("\\") {
            file.split("\\").last().unwrap().to_owned()
        } else {
            file.to_owned()
        };

        let workdir = self.settings.get_working_dir();

        if workdir.is_err() {
            if let Err(SettingsError::SettingNotFound(_)) = workdir {
                println!("Webdriver path need to specified by settings --webdriver")
            } else {
                if *DEVELOPMENT_MODE {
                    println!("{:?}", workdir);
                }
            }
            return;
        }
        let dir = workdir.unwrap();

        let task_id = self.database.get_id_by_file_name(&file);
        if let Err(e) = task_id {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        }
        let id = task_id.unwrap();

        let week = self.database.get_week_by_file_name(&file);
        if let Err(e) = week {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        }

        let path = PathBuf::from(&dir).join(&week.unwrap()).join(&file);
        if !path.exists() {
            println!("your file provided does not exists");
            return;
        }

        if let None = self.create_cses_connection().await {
            println!("cannot connect to cses website");
            return;
        }

        if let Err(e) = self
            .cses_connection
            .as_ref()
            .unwrap()
            .submit_task(file)
            .await
        {
            match e {
                CsesConnectionError::Setting(e) => {
                    if let SettingsError::SettingNotFound(s) = e {
                        match s.as_str() {
                            "webdriver is not firefox's geckodriver" => {
                                println!("Webdriver path need to specified by settings --webdriver")
                            }
                            "working directory is not defined" => {
                                println!("Working dir need to specified by settings --dir")
                            }
                            "Setting cses url is not set" => {
                                println!("Url need to specified by settings --url")
                            }
                            "Setting username is not set" => {
                                println!("Username need to specified by settings --username")
                            }
                            "password is not specified" => {
                                println!("Password need to specified by settings --password")
                            }
                            _ => {}
                        }
                    } else {
                        if *DEVELOPMENT_MODE {
                            println!("{:?}", e);
                        }
                    }
                    return;
                }
                CsesConnectionError::ValueError(_) => {
                    println!("Nothing can be submitted");
                    println!("Exercises need to be downloaded");
                    return;
                }
                _ => {
                    if *DEVELOPMENT_MODE {
                        println!("{:?}", e);
                    }
                    return;
                }
            }
        }

        let result = self
            .cses_connection
            .as_ref()
            .unwrap()
            .task_solution_result(&id)
            .await;
        if let Err(e) = result {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        }

        let s = result.unwrap();
        let passed = if s == "ACCEPTED" {
            TaskStatus::Passed
        } else {
            TaskStatus::NotDone
        };

        if let Err(e) = self.database.set_passed_by_id(passed, &id) {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        };
        println!("{}", s);

        if let Some(connection) = self.cses_connection.as_mut() {
            connection.close();
        }
    }

    fn handle_url(&self, file: &str) {
        let task_id = self.database.get_id_by_file_name(&file);
        if let Err(e) = task_id {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        }

        let url = self.settings.get_cses_url();
        if let Err(e) = url {
            if *DEVELOPMENT_MODE {
                println!("{:?}", e);
            }
            return;
        }

        println!("{}/task/{}", url.unwrap(), task_id.unwrap())
    }
}
