from selenium import webdriver
from selenium.webdriver.firefox.service import Service 
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.common.by import By
from urllib.parse import urljoin
from dotenv import load_dotenv
from bs4 import BeautifulSoup
import os, time

load_dotenv()

class CsesConnection:
    def __init__(self, url: str):
        options = Options()
        options.add_argument("--headless")
        service = Service(rf"{os.getenv("webdriver")}")
        self.driver = webdriver.Firefox(options=options, service=service)
        self.driver.get(url)
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
        url = self.url.removesuffix("/list/")
        soup = BeautifulSoup(self.driver.page_source, "html.parser")
        tasks = []
        for task_list in soup.find_all("ul", class_="task-list"):
            for task in task_list.find_all("li", class_ = "task"):
                link = task.find("a")
                task_id = link.attrs["href"].split("/")[-1]
                tasks.append((link.text, url+"/task/"+task_id, url+"/submit/"+task_id, url+"/view/"+task_id))

        return tasks
    
    def crete_files(self, tasks: list):
        for task in tasks:
            self.driver.get(task[1])
            soup = BeautifulSoup(self.driver.page_source, "html.parser")
            content = soup.find("div", class_="md")
            code = content.find("pre", class_="resize-horizontal prettyprint lang-python prettyprinted")
            
            if code:    
                code.extract()
                code = code.get_text()
            
            text = ["#"+child.text for child in content.children]
            
            if code:
                text.append(code)
            
            with open("tehtava/"+task[0]+".py", "w", encoding="utf-8") as file:
                file.writelines(text)

if __name__ == "__main__":
    c = CsesConnection("https://cses.fi/tira26k/list/")
    c.login()
    tasks = c.get_task_list()
    c.crete_files(tasks)

        

        
        