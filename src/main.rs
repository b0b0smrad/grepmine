use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::{self};
use std::fs::{self, OpenOptions};
use std::io::{Write};

#[derive(Debug, Serialize, Deserialize)]
struct Table {
    data: HashMap<String, i32>,
}

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: <command> <arguments>");
        return;
    }

    let command = &args[1];
    let filename = "data.json";
    let mut table = Table::load(filename);

    match command.as_str() {
        "add" => {
            if args.len() != 4 {
                println!("Usage: append <team> <score>");
                return;
            }
            let team = args[2].clone();
            let score: i32 = args[3].parse().expect("Score should be an integer.");
            table.append(team, score);
        }
        "update" => {
            if args.len() != 4 {
                println!("Usage: update <team> <score>");
                return;
            }
            let team = args[2].clone();
            let score: i32 = args[3].parse().expect("Score should be an integer.");
            table.update(team, score);
        }
        "delete" => {
            if args.len() != 3 {
                println!("Usage: delete <team>");
                return;
            }

            let team = args[2].clone();
            table.delete(team.clone());
            if args[2] == "*" {}
        }
        "*" => {
            table.delete_all();
        }
        "print" => {
            table.print();
        }
        _ => {
            println!("Unknown command.");
        }
    }

    table.save(filename);
}
