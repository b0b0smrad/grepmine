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
use std::{thread, time::Duration};

#[derive(Debug, Default)]
pub struct App {
    input: String,
    prev_input: String,
    path_index: usize,
    hl_block: Rect,
    input_mode: InputMode,
    char_index: usize,
    dir_paths: Vec<String>,
    dir_filtered: Vec<String>,
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
        thread::sleep(Duration::from_millis(100));
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     println!("Usage: <string> <string>");
    //     return Ok(());
    // }
    // let command = &args[1];
    // match command.as_str() {
    //
    //
    // }

    //TODO:(b0b0)test for levenshtein
    // let string1 = String::from("kitty");
    // let string2 = String::from("shitty");

    // let mut string1 = args[1].to_string();
    // // println!("{string1}");
    // let mut string2 = args[2].to_string();
    // // println!("{string2}");

    // println!("first string:");
    // io::stdin()
    //     .read_line(&mut string1)
    //     .expect("Failed to read line");
    // println!("second string:");
    // io::stdin()
    //     .read_line(&mut string2)
    //     .expect("Failed to read line");
    // let results: usize = levenshtein_recursive(&string1, &string2, string1.len(), string2.len());
    // println!("{results}");
    // Ok(())
    // println!("before the terminal");
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    restore();
    //
    let selected_path = app.print_output();
    if app.exit == true {
        println!("{}", selected_path);
    }
    //
    app_result
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
    fn new() -> Self {
        let preset = "> ";
        Self {
            input: preset.to_string(),
            prev_input: String::new(),
            input_mode: InputMode::Normal,
            path_index: 0,
            exit: false,
            dir_paths: Vec::new(),
            dir_filtered: Vec::new(),
            char_index: preset.chars().count(),
            hl_block: Rect {
                x: 0,
                y: 0, // Adjust based on your paragraph's layout
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
            // thread::sleep(Duration::from_millis(100));
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.update();
        }
        Ok(())
    }

    //NOTE: in progress:

    fn enter_char(&mut self, new_char: char) {
        let byte_index = self
            .input
            .char_indices()
            .nth(self.char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len());
        self.input.insert(byte_index, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.char_index == 0 {
            return;
        }
        let cursor_delete_char = self.char_index.saturating_sub(1);
        if let Some((byte_index, _)) = self.input.char_indices().nth(cursor_delete_char) {
            self.input.remove(byte_index);
            self.char_index -= 1;
        }
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
        if self.dir_paths.is_empty() {
            return;
        }
        // println!(
        //     "path_index: {}, dir_paths.len(): {}",
        //     self.path_index,
        //     self.dir_paths.len()
        // );
        // if self.hl_block.y > 0 {
        //     self.hl_block.y -= 1;
        // }
        if self.path_index > 0 {
            self.path_index -= 1;
        }
    }

    fn move_highlight_down(&mut self) {
        // self.hl_block.y = self.path_index.clone() as u16;
        if self.path_index < self.dir_paths.len() - 1 {
            self.path_index += 1;
        }
    }
    fn submit(&mut self) {
        self.exit = true;
    }

    // fn highlight_fuzzily(&mut self){
    //
    //
    // }
    fn print_output(&mut self) -> String {
        // let export_path = self.dir_paths.get(self.path_index);
        self.dir_paths = self.current_path();
        let export_path = self.dir_paths[self.path_index].clone();

        // let print: bool = self.exit;
        // let export_path = self.dir_paths;
        // if print == true {
        //     if export_path.is_some() {
        //     } else {
        //         panic!("no valid directory at the index {}", self.path_index);
        //     }
        // }
        export_path
    }

    fn debug_path(&mut self) -> Paragraph<'_> {
        let paragraph = Paragraph::new(format!("P:{} hl|y: {}", self.path_index, self.hl_block.y))
            .style(Style::default().fg(Color::Rgb(165, 180, 201)))
            .block(
                Block::default()
                    .title("Debug")
                    .borders(Borders::ALL)
                    .padding(ratatui::widgets::Padding {
                        left: 1,
                        right: 1,
                        top: 1,
                        bottom: 1,
                    }),
            );
        paragraph
    }
    fn update(&mut self) {
        //NOTE: i would create something like this and then
        //make it : if self.input has changed do this: and put this function into the uppper side
        //of draw or under self.input and input widget?

        self.dir_paths = self.current_path();

        // for dir in self.dir_paths.iter() {
        let filtered: Vec<String> = self
            .dir_paths
            .iter()
            .filter(|dir| levenshtein_recursive(&self.input, dir, self.input.len(), dir.len()) < 2)
            .cloned()
            .collect();

        // let lv_dirs: Vec<String> = dir;
        // let distance = levenshtein_recursive(&self.input, dir, self.input.len(), dir.len());
        // if distance < 2 {
        // }
        self.dir_filtered = filtered;
        // }

        //this needs to  be a the end of the loop
        self.prev_input = self.input.clone();
    }
    fn draw(&mut self, frame: &mut Frame) {
        // self.dir_paths = self.current_path();
        let size = frame.area();

        let vertical = Layout::vertical([
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        // self.path_index += 1;
        // let lm = self.dir_paths[1].len() as u16;
        self.hl_block.y = frame.area().y + 2 + self.path_index as u16;
        // self.hl_block.y = lm;

        let pwd = env::current_dir().unwrap();
        // self.dir_paths = vec![pwd.display().to_string()];
        let tittle = pwd.display().to_string();
        let input_bar = Paragraph::new(">").block(
            Block::default()
                .borders(Borders::BOTTOM)
                .fg(Color::Rgb(255, 172, 172))
                .style(Style::default().fg(Color::Magenta)), // .bg(Color::Rgb(62, 30, 104)),
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

        let _i = 0;
        let mut temp_dirs = self.dir_paths.clone();
        if (self.input != self.prev_input) {
            temp_dirs = self.dir_filtered.clone();
        }
        let paragraph = Paragraph::new(format!("{}", temp_dirs.join("\n")))
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 172, 172))
                    .bg(Color::Rgb(62, 30, 104)),
            )
            .block(
                Block::default()
                    .title(tittle)
                    .style(Style::default().fg(Color::Rgb(255, 172, 172)))
                    .bg(Color::Rgb(62, 30, 104))
                    .borders(Borders::ALL)
                    .padding(ratatui::widgets::Padding {
                        left: 3,
                        right: 2,
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

        let block = Block::new().style(Style::default().bg(Color::Rgb(62, 30, 104)));
        // .borders(Borders::ALL);
        // .border_style(Style::on_light_red());
        // let debug_p = self.debug_path().clone();
        let size_debug = Rect {
            x: (32),
            y: (18),
            width: (40),
            height: (10),
        };
        frame.render_widget(input, input_area);
        // frame.render_widget(debug_p, size_debug);
        frame.render_widget(paragraph, size);
        frame.render_widget(help_message, help_area);
        frame.render_widget(block, size);
        frame.render_widget(input_bar, input_area);
        //
        if !self.dir_paths.is_empty() {
            self.hl_block.width = frame.area().width;
            let highlight_block =
                Block::default().style(Style::default().bg(Color::Rgb(228, 90, 147)));
            frame.render_widget(highlight_block, self.hl_block);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Handle user input (e.g., quitting)

        // if self.exit == true {
        //     self.print_output();
        // }
        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Enter => self.submit(),
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
            // self.hl_block.y = self.path_index as u16;
        }
        Ok(())
    }

    fn current_path(&self) -> Vec<String> {
        // let mut paths: Vec<str> = Vec::new();
        let mut paths: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir("./") {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_name() {
                    // paths.push_str(&format!("{}\n", name.to_string_lossy()));
                    paths.push(name.to_string_lossy().to_string());
                }
            }
        }
        // println!("Total entries found: {}", paths.len()); // Add this line
        paths
    }
}

fn levenshtein_recursive(str1: &String, str2: &String, m: usize, n: usize) -> usize {
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    if str1.chars().nth(m - 1) == str2.chars().nth(n - 1) {
        let m2 = m - 1;
        let n2 = n - 1;
        levenshtein_recursive(&str1, &str2, m2, n2)
    } else {
        let insert = levenshtein_recursive(&str1, &str2, m, n - 1);
        let remove = levenshtein_recursive(&str1, &str2, m - 1, n);
        let replace = levenshtein_recursive(&str1, &str2, m - 1, n - 1);
        return 1 + std::cmp::min(insert, std::cmp::min(remove, replace));
    }
}
// fn levenshteinRecursive(const &str str1,
//                         const &str str2,int m, int n)
// {
//
//     // str1 is empty
//     if (m == 0) {
//         return n;
//     }
//     // str2 is empty
//     if (n == 0) {
//         return m;
//     }
//
//     if (str1[m - 1] == str2[n - 1]) {
//         return levenshteinRecursive(str1, str2, m - 1,
//                                     n - 1);
//     }
//
//     return 1
//         + min(
//
//             // Insert
//             levenshteinRecursive(str1, str2, m, n - 1),
//             min(
//
//                 // Remove
//                 levenshteinRecursive(str1, str2, m - 1,
//                                         n),
//
//                 // Replace
//                 levenshteinRecursive(str1, str2, m - 1,
//                                         n - 1)));
// }
//
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
