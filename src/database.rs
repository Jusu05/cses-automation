pub struct Database;

impl Database {
    pub fn new() -> Self {
        Database
    }

    fn create_database(&self) {

    }

    pub fn add_task(&self, id: &i32, file_name: &str, week: &str) {

    }

    pub fn get_passed_by_id(&self, task_id: &i32) {

    }

    pub fn get_id_by_file_name(&self, file_name: &str) -> i32 {
        0
    }

    pub fn get_week_by_file_name(&self, file_name: &str) -> String {
        "".to_owned()
    }

    pub fn get_all_task_names(&self) -> Vec<&str> {
        Vec::new()
    }

    pub fn get_all_weeks(&self) -> Vec<&str> {
        Vec::new()
    }

    pub fn set_passed_by_id(&self, passed: i32, task_id: i32) {

    }

    pub fn set_file_name_by_id(&self, file_name: &str, task_id: i32) {

    }

    pub fn num_of_task(&self) -> i32 {
        0
    }
}
