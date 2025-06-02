use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::*,
    restore,
    style::{self, palette::material::*, Color, Style, Stylize},
    symbols::{border, half_block},
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
    hl_block: Rect,
    input_mode: InputMode,
    char_index: usize,
    path_string: Vec<String>,
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
            path_string: Vec::new(),
            char_index: 0,
            hl_block: Rect {
                x: 0,
                y: 1, // Adjust based on your paragraph's layout
                width: 0,
                height: 1, // Adjust based on your line height
            },
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

    fn enter_char(&mut self, new_char: char) {
        let index = self.char_index + 1;
        //let index =
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.char_index == 0 {
            return;
        }
        let cursor_delete_char = self.char_index.saturating_sub(1);

        //self.char_index = self.clamp_cursor(cursor_delete_char);
        // self.input = self.input.remove(cursor_delete_char);
        if let Some((byte_index, _)) = self.input.char_indices().nth(cursor_delete_char) {
            self.input.remove(byte_index);
            self.char_index -= 1;
        }

        // self.move_cursor_left();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    fn reset_cursor(&mut self) {
        self.char_index = 0;
    }
    fn move_highlight_up(&mut self) {
        if self.hl_block.y > 0 {
            self.hl_block.y -= 1;
        }
    }

    fn submit_message(&mut self) {
        // if !self.hl_block.is_empty() {
        //     return;
        // } else {
        //     // return self.hl_block.positions();
        //     self.current_path();
        // }
        // self.exit = true;
    }
    fn move_highlight_down(&mut self) {
        self.hl_block.y += 1;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let paths = self.current_path();
        let pwd = env::current_dir().unwrap();
        let tittle = pwd.display().to_string();
        let input_bar = Paragraph::new(">").block(
            Block::default()
                .borders(Borders::ALL)
                .fg(Color::Rgb(25, 25, 25))
                .style(Style::default().fg(Color::Magenta))
                .bg(Color::Rgb(85, 55, 55)),
        );

        let [help_area, input_area, messages_area] = vertical.areas(frame.area());
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);

        let input_area = Rect {
            x: 2,
            y: size.height.saturating_sub(3),
            width: size.width / 2,
            height: 3,
        };
        let paragraph = Paragraph::new(paths.clone())
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
        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        let block = Block::new().style(Style::default().bg(Color::Rgb(30, 25, 89)));
        frame.render_widget(paragraph, size);
        frame.render_widget(help_message, help_area);
        frame.render_widget(block, size);
        frame.render_widget(input_bar, input_area);
        //
        if !paths.is_empty() {
            self.hl_block.width = frame.area().width;
            let highlight_block = Block::default().style(Style::default().bg(Color::White));
            frame.render_widget(highlight_block, self.hl_block);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Handle user input (e.g., quitting)

        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                InputMode::Normal => match key.code {
                    // KeyCode::Enter => self.submit_message(),
                    KeyCode::Char('i') => {
                        self.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => self.move_highlight_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.move_highlight_down(),

                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    // KeyCode::Enter => self.submit_message(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    //KeyCode::Backspace => self.delete_char(),
                    KeyCode::Up => self.move_highlight_up(),
                    KeyCode::Modifier(event::ModifierKeyCode::LeftControl)
                        if key.code == KeyCode::Char('k') =>
                    {
                        self.move_highlight_up()
                    }
                    //KeyCode::Modifier(event::ModifierKeyCode::LeftControl) => {
                    //     KeyCode::Char('k') => self.move_highlight_up();
                    //     KeyCode::Char('j') => self.move_highlight_down();
                    //},
                    KeyCode::Down => self.move_highlight_down(),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                },
                InputMode::Editing => {}
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
