
# CSES Automation

This project is comandlinetool for submissions and problem fetching Helsinki University cses courses. Its writen in rust and uses browser automation to interact with cses website and curently supports only firefox and geckoriver.


## Installation

```bash
git clone https://github.com/Jusu05/cses-automation.git
cd cses-automation

```
To run program also Firfox and it's webdriver Geckodriver need to be installed.

## Running
Before usign commands 'submit' or 'download' all settings need to be set
```bash
cargo run <command>

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
cargo build -r
```

