#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    pub todo_list: Vec<String>,
}
