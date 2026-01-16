from selenium import webdriver
from selenium.webdriver.firefox.service import Service 
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import Select
from dotenv import load_dotenv
from bs4 import BeautifulSoup
from pathlib import Path
import os

load_dotenv()

class CsesConnection:
    def __init__(self, url: str):
        options = Options()
        options.add_argument("--headless")
        service = Service(rf"{os.getenv("webdriver")}")
        self.driver = webdriver.Firefox(options=options, service=service)
        self.driver.get(f"{url}/list/")
        self.url = url

    def login(self):
        accaunt = self.driver.find_element(By.CSS_SELECTOR, "body > div.header > div > div > a.account")
        accaunt.click()
        
        username = self.driver.find_element(By.ID, "username")
        password = self.driver.find_element(By.ID, "password")

        if username and password:
            username.send_keys(os.getenv("mooc_user"))
            password.send_keys(os.getenv("mooc_password"))
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

            path = Path().cwd().joinpath(f"tehtava/{task[1]}")
            with open(path, "w", encoding="utf-8") as file:
                file.writelines(text)

    def submit_task(self, task_name, tasks):
        task = self.get_task(task_name, tasks)
        if not task:
            return

        path = Path().cwd().joinpath(f"tehtava/{task[1]}")

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