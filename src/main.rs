use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    restore,
    style::{self, palette::material::*, Color, Style, Stylize},
    symbols::border,
    text::{self, Line, Text},
    widgets::{Block, Borders, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::env::{self};
use std::fs::{self, canonicalize, read_dir, DirEntry, OpenOptions, ReadDir};
use std::io::{self, prelude::*, Write};
use std::path::Path;
use std::{collections::HashMap, default};

#[derive(Debug, Default)]
pub struct App {
    input: String,
    input_mode: InputMode,
    cur_char_index: usize,
    exit: bool,
}
#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}
#[derive(Debug, Serialize, Deserialize)]
struct Table {
    data: HashMap<String, i32>,
}
fn render(frame: &mut Frame) {
    frame.render_widget("WIDGET", frame.area());
    frame.area();
}
fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    //if args.len() < 2 {
    //    println!("Usage: <command> <arguments>");
    //    return Ok(());
    //}

    let mut terminal = ratatui::init();
    //let mut app = App; //let result = run(terminal);
    let app_result = App::new().run(&mut terminal);
    restore();
    app_result

    //let mut entries = fs::read_dir("..")?
    //    .map(|res| res.map(|e| e.path()))
    //    .collect::<Result<Vec<_>, io::Error>>()?;
    //
    //entries.sort();
    //let command = &args[1];
    //let filename = "data.json";
    //let mut table = Table::load(filename);
    //
    //match command.as_str() {
    //    "sort" => {
    //        for entry in &entries {
    //            println!("{}", entry.display());
    //        }
    //        return Ok(());
    //    }
    //    "add" => {
    //        if args.len() != 4 {
    //            println!("Usage: append <team> <score>");
    //            return Ok(());
    //        }
    //        let team = args[2].clone();
    //        let score: i32 = args[3].parse().expect("Score should be an integer.");
    //        table.append(team, score);
    //    }
    //    "update" => {
    //        if args.len() != 4 {
    //            println!("Usage: update <team> <score>");
    //            return Ok(());
    //        }
    //        let team = args[2].clone();
    //        let score: i32 = args[3].parse().expect("Score should be an integer.");
    //        table.update(team, score);
    //    }
    //    "delete" => {
    //        if args.len() != 3 {
    //            println!("Usage: delete <team>");
    //            return Ok(());
    //        }
    //
    //        let team = args[2].clone();
    //        table.delete(team.clone());
    //        if args[2] == "*" {}
    //    }
    //    "*" => {
    //        table.delete_all();
    //    }
    //    "print" => {
    //        table.print();
    //    }
    //    _ => {
    //        println!("Unknown command.");
    //        return Ok(());
    //    }
    //}
    //
    //table.save(filename);
    //Ok(())
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
fn print_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                print_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            exit: false,
            cur_char_index: 0,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    //NOTE: in progress:
    //
    //fn enter_char(&mut self, new_char: char) {
    //    let index = self.cur_char_index.byte;
    //    //let index =
    //    self.input.insert(index, new_char);
    //    self.move_cursor_right();
    //}

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cur_char_index.saturating_sub(1);
        self.cur_char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cur_char_index.saturating_add(1);
        self.cur_char_index = self.clamp_cursor(cursor_moved_right);
    }

    fn reset_cursor(&mut self) {
        self.cur_char_index = 0;
    }

    fn draw(&self, frame: &mut Frame) {
        let size = frame.area();

        let paths = self.current_path();
        let pwd = env::current_dir().unwrap();
        let tittle = pwd.display().to_string();
        let input_bar = Paragraph::new("").block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red)),
        );

        let input_area = Rect {
            x: 0,
            y: size.height.saturating_sub(3),
            width: size.width,
            height: 2,
        };
        let paragraph = Paragraph::new(paths)
            .style(Style::default().fg(Color::Rgb(165, 180, 201)))
            .block(
                Block::default()
                    .title(tittle)
                    .borders(Borders::ALL)
                    .padding(ratatui::widgets::Padding {
                        left: 2,
                        right: 5,
                        top: 1,
                        bottom: 0,
                    }),
            );

        let block = Block::new().style(Style::default().bg(Color::Rgb(30, 25, 89)));
        frame.render_widget(paragraph, size);
        frame.render_widget(block, size);
        frame.render_widget(input_bar, input_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Handle user input (e.g., quitting)
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn current_path(&self) -> String {
        let mut paths = String::new();
        if let Ok(entries) = fs::read_dir("./") {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_name() {
                    paths.push_str(&format!("{}\n", name.to_string_lossy()));
                }
            }
        }
        paths
    }
}
