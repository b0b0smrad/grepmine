mod table
impl Table {
    fn new() -> Table {
        Table {
            data: HashMap::new(),
        }
    }

    fn load(filename: &str) -> Table {
        let content = fs::read_to_string(filename);
        match content {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| Table::new()),
            Err(_) => Table::new(),
        }
    }

    fn save(&self, filename: &str) {
        let json = serde_json::to_string(&self).expect("Failed to serialize table");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .expect("Failed to open file");
        file.write_all(json.as_bytes())
            .expect("Failed to write to file");
    }

    fn append(&mut self, team: String, score: i32) {
        self.data.insert(team, score);
    }

    fn update(&mut self, team: String, score: i32) {
        if let Some(entry) = self.data.get_mut(&team) {
            *entry = score;
        } else {
            println!("Team not found.");
        }
    }

    fn delete(&mut self, team: String) {
        if self.data.remove(&team).is_none() {
            println!("Team not found.");
        }
    }
    fn delete_all(&mut self) {
        self.data.clear();
    }

    fn print(&self) {
        println!("________________________________________");
        println!("{:<20} {:<10} {:<20}", "| Team", "| |", "Score |");
        println!("---------------------------------------");
        for (team, score) in &self.data {
            println!("{:<20}{:<10}  {:<10}", team, "|", score);
        }
    }
}
