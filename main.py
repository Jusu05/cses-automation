from selenium import webdriver
from selenium.webdriver.firefox.service import Service 
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import Select
from configparser import ConfigParser
from pathlib import Path
from bs4 import BeautifulSoup
from urllib.parse import urlparse
import os

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

        if not file.is_file():
            raise ValueError("annettu tiedosto ei ole tiedosto")

        if file.suffix != ".ini":
            raise ValueError("tiedosto ei ole ini tiedosto")

        self._file = file

    def edit(self, class_: str, variable: str, value: str):
        self._parser.set(class_, variable, value)

        with open(self._file, 'w', encoding="utf-8") as file:
            self._parser.write(file)

    def read(self, class_: str, variable: str):
        self._parser.read(self._file, encoding="utf-8")
        value = self._parser.get(class_, variable)

        return value


class Settings():
    def __init__(self, path: Path) -> None:
        self._parser = IniParser(path.joinpath("settings.ini"))

    def get_webdriver_path(self) -> str:
        return self._parser.read("general", "webdriver")

    def get_cses_url(self) -> str:
        return self._parser.read("general", "cses_url")

    def get_username_and_password(self) -> tuple[str, str]:
        return self._parser.read("user", "username"), self._parser.read("password", "username")

    def get_working_dir(self) -> str:
        return self._parser.read("system", "working_dir")

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

    def set_username_and_password(self, username, password) -> tuple[str, str]:
        if not isinstance(username, str):
            raise TypeError(f"username is not str it's {type(username)}")
        if not isinstance(password, str):
            raise TypeError(f"url is not str it's {type(password)}")

        self._parser.edit("user", "username", username)
        self._parser.edit("user", "password", password)

    def set_working_dir(self, dir: str | Path) -> str:
        if isinstance(dir, str):
            dir = Path(dir)
        elif isinstance(dir, Path):
            pass
        else:
            raise TypeError(f"dir is not str it's {type(dir)}")

        if dir.exists():
            FileNotFoundError("folder does not exis")

        if dir.is_dir():
            FileNotFoundError("dir does is not fil")

        return self._parser.edit("system", "working_dir", str(dir))


class CsesConnection:
    def __init__(self, url: str):
        self._settings = Settings()
        options = Options()
        options.add_argument("--headless")
        service = Service(self._settings.get_webdriver_path())
        self.driver = webdriver.Firefox(options=options, service=service)
        self.driver.get(f"{url}/list/")
        self.url = url

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
        task = self.get_task(task_name, tasks)
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

    def get_task(self, task_name, tasks):
        task = list(filter(lambda task: task[0]==task_name, tasks))
        if len(task) == 0:
            task = list(filter(lambda task: task[1]==task_name, tasks))
        if len(task) == 0:
            return
        return task[0]

    def task_solution_result(self, task_name, tasks):
        task = self.get_task(task_name, tasks)
        if not task:
            return

        self.driver.get(f"{self.url}/view/{task[2]}/")
        solution = self.driver.find_element(By.CSS_SELECTOR, "body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr > td:nth-child(4) > a")
        solution.click()
        result = self.driver.find_element(By.CSS_SELECTOR, "body > div.skeleton > div.content-wrapper > div.content > table > tbody > tr:nth-child(6) > td:nth-child(2) > span")
        return result.text