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
    path_index: usize,
    hl_block: Rect,
    input_mode: InputMode,
    char_index: usize,
    dir_paths: Vec<String>,
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
    // let args: Vec<String> = env::args().collect();
    //if args.len() < 2 {
    //    println!("Usage: <command> <arguments>");
    //    return Ok(());
    //}

    // println!("before the terminal");
    let mut terminal = ratatui::init();
    let mut app = App::new(); //let result = run(terminal);
                              // app.path_index = app.dir_paths.len() as usize;
                              // app.hl_block.y += app.path_index as u16;
                              // app.hl_block.y += 3;

    // app.dir_paths = app.current_path();
    let app_result = app.run(&mut terminal);
    restore();

    let selected_path = app.print_output();
    if app.exit == true {
        println!("{}", selected_path);
    }
    // if selected_path.is_some() {
    // } else {
    //     println!("directory or file doesn't exists");
    // }

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
            input_mode: InputMode::Normal,
            path_index: 0,
            exit: false,
            dir_paths: Vec::new(),
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
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
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

    fn debug_path(&mut self) -> Paragraph {
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
    fn draw(&mut self, frame: &mut Frame) {
        let paths = self.current_path();
        self.dir_paths = self.current_path();
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

        let i = 0;
        let paragraph = Paragraph::new(format!("{}", paths.join("\n")))
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
        frame.render_widget(input, input_area);

        let block = Block::new().style(Style::default().bg(Color::Rgb(62, 30, 104)));
        // .borders(Borders::ALL);
        // .border_style(Style::on_light_red());
        let debug_p = self.debug_path();
        let size_debug = Rect {
            x: (32),
            y: (18),
            width: (40),
            height: (10),
        };
        frame.render_widget(debug_p, size_debug);
        frame.render_widget(paragraph, size);
        frame.render_widget(help_message, help_area);
        frame.render_widget(block, size);
        frame.render_widget(input_bar, input_area);
        //
        if !paths.is_empty() {
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
