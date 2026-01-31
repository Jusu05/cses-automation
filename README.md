
# CSES Automation

This project is comand linetool for submissions and problem fetching Helsinki University cses courses. It uses python 3.10+, selenium and beutifullsuop4 for interacting with cses website and curently supports only firefox and geckoriver.


## Installation

```bash
# clone repority
git clone https://github.com/Jusu05/cses-automation.git

# Navigate to the project directory
cd cses-automation

# Create a virtual environment (optional but recommended)
python -m venv .venv
source .venv/bin/activate  # On Windows use `.venv\Scripts\activate`

# Install the required packages
pip install -r requirements.txt

```
To run program also Firfox and it's webdriver Geckodriver need to be installed.

## Running
Before usign commands 'submit' or 'download' all settings need to be set
```bash
python main.py <command>

# list of commands:
# submit <file> - file name that will be submited
# download - downloads all not yet downloaded excises
# url <file> - file name that shows execise page
# settings <argumets...> - settings for program
# 
#     list of argumets
#     --webdriver <driver> - set path to browser webdriver
#     --url <url> - set url to cses website
#     --dir <path> - set dirrectory where files will be downloaded
#     --username <username> - set mooc username
#     --password <password> - set mooc password
```
## Building 
This is recommended way build program to executable
```bash
source .venv/bin/activate  # On Windows use `.venv\Scripts\activate`
pip install nuitka ordered-set zstandard 
python -m nuitka --standalone cses.py
```

