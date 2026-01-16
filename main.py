from selenium import webdriver
from selenium.webdriver.firefox.service import Service 
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import Select
from bs4 import BeautifulSoup
from urllib.parse import urlparse
from configparser import ConfigParser
from pathlib import Path
import argparse, os

class IniParser:
    def __init__(self, file: str | Path) -> None:
            self._parser = ConfigParser()
            self.file = file

    @property
    def file(self):
        return self._file

    @file.setter
    def file(self, file: str | Path):
        if isinstance(file, str):
            file = Path(file)

        if not isinstance(file, Path):
            raise TypeError(f"teidosto ei ole merkkijono tai polku vaan {file}")

        if file.suffix != ".ini":
            raise ValueError("tiedosto ei ole ini tiedosto")

        if not file.is_file():
            with open(file, "w") as f:
                pass

        self._file = file

    def edit(self, section: str, option: str, value: str):
        self._parser.set(section, option, value)

        with open(self._file, "w", encoding="utf-8") as file:
            self._parser.write(file)

    def read(self, section: str, option: str):
        self._parser.read(self._file, encoding="utf-8")
        value = self._parser.get(section, option, fallback=None)

        return value


class Settings():
    def __init__(self, path: Path):
        self._parser = IniParser(path.joinpath("settings.ini"))

    def get_webdriver_path(self) -> str:
        driver = self._parser.read("general", "webdriver")

        if not driver:
            raise ValueError("webdriver path is not spefied")

        return driver

    def get_cses_url(self) -> str:
        url = self._parser.read("general", "cses_url")

        if not url:
            raise ValueError("url is not spefied")

        return url

    def get_username_and_password(self) -> tuple[str, str]:
        username = self._parser.read("user", "username")
        password = self._parser.read("user", "password")

        if not username:
            raise ValueError("username is not spefied")
        if not password:
            raise ValueError("password is not spefied")

        return username, password

    def get_working_dir(self) -> str:
        path = self._parser.read("system", "working_dir")

        if not path:
            raise ValueError("working directory is not defined")

        return path

    def set_webdriver_path(self, driver: str):
        if not isinstance(driver, str):
            raise TypeError(f"driver is not str it's {type(driver)}")

        if not driver.endswith("geckodriver.exe"):
            raise ValueError("webdriver is not firefoxs geckodriver")

        if not Path(driver).exists():
            raise FileNotFoundError("webdriver not found")

        self._parser.edit("general", "webdriver", driver)

    def set_cses_url(self, url: str) -> str:
        if not isinstance(url, str):
            raise TypeError(f"url is not str it's {type(url)}")

        parsed_url = urlparse(url)
        if not all([parsed_url.scheme, parsed_url.netloc]):
            raise ValueError(f"url is not valid")

        if "cses.fi" not in url:
            raise ValueError(f"url domain is not cses.fi")

        url = url.removesuffix("/list/")
        url = url.removesuffix("/")

        return self._parser.edit("general", "cses_url")

    def set_password(self, password):
        if not isinstance(password, str):
            raise TypeError(f"url is not str it's {type(password)}")

        self._parser.edit("user", "password", password)

    def set_username(self, username) -> tuple[str, str]:
        if not isinstance(username, str):
            raise TypeError(f"username is not str it's {type(username)}")

        self._parser.edit("user", "username", username)

    def set_working_dir(self, dir: str | Path) -> str:
        if isinstance(dir, str):
            dir = Path(dir)
        elif isinstance(dir, Path):
            pass
        else:
            raise TypeError(f"dir is not str it's {type(dir)}")

        if dir.exists():
            FileNotFoundError("given path does not exist")

        if dir.is_dir():
            FileNotFoundError("given path is not directory")

        return self._parser.edit("system", "working_dir", str(dir))


class CsesConnection:
    def __init__(self, settings: Settings):
        self._settings = settings
        options = Options()
        options.add_argument("--headless")

        driver = self._settings.get_webdriver_path()
        service = Service(driver)
        self.driver = webdriver.Firefox(options=options, service=service)

        self.url = self._settings.get_cses_url()
        self.driver.get(f"{self.url}/list/")

    def login(self):
        accaunt = self.driver.find_element(By.CSS_SELECTOR, "body > div.header > div > div > a.account")
        accaunt.click()
        
        username_field = self.driver.find_element(By.ID, "username")
        password_field = self.driver.find_element(By.ID, "password")

        if username_field and password_field:
            username, password = self._settings.get_username_and_password()
            username_field.send_keys(username)
            password_field.send_keys(password)
            sigin_button = self.driver.find_element(By.CSS_SELECTOR, "#content-area > div.login-align > div > form > input.btn.btn-primary.login-form-button")
            sigin_button.click()

    def get_task_list(self) -> list:
        soup = BeautifulSoup(self.driver.page_source, "html.parser")
        tasks = []
        for task_list in soup.find_all("ul", class_="task-list"):
            for task in task_list.find_all("li", class_ = "task"):
                link = task.find("a")
                task_id = link.attrs["href"].split("/")[-1]
                content = soup.find("div", class_="md")

                self.driver.get(f"{self.url}/task/{task_id}")
                soup2 = BeautifulSoup(self.driver.page_source, "html.parser")
                content = soup2.find("div", class_="md")
                file_name = [file.text for file in content.find_all("code") if file.text.endswith(".py")]
                if len(file_name) == 0:
                    continue

                tasks.append((link.text, file_name[0], task_id))

        return tasks
    
    def load_task(self, tasks: list):
        for task in tasks:
            self.driver.get(f"{self.url}/task/{task[2]}")
            soup = BeautifulSoup(self.driver.page_source, "html.parser")
            content = soup.find("div", class_="md")
            code = content.find("pre", class_="resize-horizontal prettyprint lang-python prettyprinted")
            
            if code:    
                code.extract()
                code = code.get_text()
            
            text = ["#"+child.text for child in content.children]
            text.append("\n")

            if code:
                text.append(code)

            path = Path(self._settings.get_working_dir()).joinpath(task[1])
            with open(path, "w", encoding="utf-8") as file:
                file.writelines(text)

    def submit_task(self, task_name, tasks):
        task = self._get_task(task_name, tasks)
        if not task:
            return

        path = Path(self._settings.get_working_dir()).joinpath(f"{task[1]}")

        with open(path, "r", encoding="utf-8") as file:
            lines = file.readlines()
            skip = lines[0].startswith("#")
            if skip:
                for i, line in enumerate(lines):
                    if line == "\n":
                        lines = lines[i+1:]
                        break
        if skip:
            with open(path, "w", encoding="utf-8") as file:
                file.writelines(lines)

        self.driver.get(f"{self.url}/submit/{task[2]}")
        upload = self.driver.find_element(By.NAME, "file")
        upload.send_keys(str(path))
        lanquage_selector = Select(self.driver.find_element(By.CSS_SELECTOR, "#lang"))
        lanquage_selector.select_by_visible_text("Python3")
        submit = self.driver.find_element(By.CSS_SELECTOR, ".content > form:nth-child(1) > p:nth-child(6) > input:nth-child(1)")
        submit.click()

    def _get_task(self, task_name, tasks):
        task = list(filter(lambda task: task[0]==task_name, tasks))
        if len(task) == 0:
            task = list(filter(lambda task: task[1]==task_name, tasks))
        if len(task) == 0:
            return
        return task[0]

    def task_solution_result(self, task_name, tasks):
        task = self._get_task(task_name, tasks)
        if not task:
            return

        self.driver.get(f"{self.url}/view/{task[2]}/")
        solution = self.driver.find_element(By.CSS_SELECTOR, "body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr > td:nth-child(4) > a")
        solution.click()
        result = self.driver.find_element(By.CSS_SELECTOR, "body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr:nth-child(6) > td:nth-child(2) > span")
        return result.text


class App:
    def __init__(self):
        self.parser = argparse.ArgumentParser(description="This is comand line tool for download and subting for mooc cses exesises")
        self.settings = Settings(Path("."))

    def main(self):
        self.build_arguments()
        self.parse_arquments()

    def build_arguments(self):
        subparsers = self.parser.add_subparsers(help="help for subcommand", dest="subcommand")

        self.parser.add_argument("submit", help="file name file that will be submited")
        subparsers.add_argument("download", help="dowload all not yet download excises")

        settings_parser = subparsers.add_parser("settings", help="settings for program")
        settings_parser.add_argument("--webdriver", type=str, help="set path to browser webdriver")
        settings_parser.add_argument("--url", type=str, help="set url to cses website")
        settings_parser.add_argument("--dir", type=str, help="set dirrectory where files will be downloaded")
        settings_parser.add_argument("--username", type=str, help="set mooc username")
        settings_parser.add_argument("--password", type=str, help="set mooc password")

    def parse_arquments(self):
        args = self.parser.parse_args(["submit", "download", "settings"])

        if args.submit:
            self.handle_submit(args.submit)

        if args.subcommand == "download":
            self.handle_download(args.download)

        if args.subcommand == "settings":
            self.handle_settings(args)

    def handle_settings(self, args):
        if args.webdriver:
            try:
                self.handle_settings.set_webdriver_path(args.webdriver)
            except ValueError:
                print("only supported driver is firefox geckodriver")
            except FileNotFoundError:
                print("You need to download firefox's webdriver and it needs to in path")
                print("It can be here https://github.com/mozilla/geckodriver/releases")
            except:
                pass

        if args.url:
            try:
                self.settings.set_cses_url(args.url)
            except ValueError as e:
                print(e.args[0])
            except:
                pass

        if args.dir:
            try:
                self.settings.set_cses_url(args.dir)
            except FileNotFoundError as e:
                print(e.args[0])
            except:
                pass

        if args.username:
            try:
                self.settings.set_username(args.username)
            except:
                pass

        if args.password:
            try:
                self.settings.set_password(args.password)
            except:
                pass

    def handle_download(self):
        try:
            if not hasattr(self, "cses_connection"):
                self.cses_connection = CsesConnection(self.settings)

            tasks = self.cses_connection.get_task_list()
            dir = self.settings.get_working_dir()
            downloaded_tasks = set(os.listdir(dir))
            tasks = [task for task in tasks if task[1] not in downloaded_tasks]
            self.cses_connection.load_task(tasks)

        except ValueError as e:
            match e.args[0]:
                case "webdriver path is not spefied":
                    print("Webdriver path need to spefied by settings --webdriver")
                case "working directory is not defined":
                    print("Working dir need to spefied by settings --dir")
                case "url is not spefied":
                    print("Url need to spefied by settings --url")
                case "username is not spefied":
                    print("Username need to spefied by settings --username")
                case "password is not spefied":
                    print("Password need to spefied by settings --password")

    def handle_submit(self, file: str):
        try:
            if not hasattr(self, "cses_connection"):
                self.cses_connection = CsesConnection(self.settings)

            tasks = self.cses_connection.get_task_list()

            if len(tasks) == 0:
                return

            if file not in [task[1] for task in tasks]:
                return

            self.cses_connection.submit_task(file, tasks)

        except ValueError as e:
            match e.args[0]:
                case "webdriver path is not spefied":
                    print("Webdriver path need to spefied by settings --webdriver")
                case "working directory is not defined":
                    print("Working dir need to spefied by settings --dir")
                case "url is not spefied":
                    print("Url need to spefied by settings --url")
                case "username is not spefied":
                    print("Username need to spefied by settings --username")
                case "password is not spefied":
                    print("Password need to spefied by settings --password")


app = App()
app.main()