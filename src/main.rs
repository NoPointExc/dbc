use std::io::{self, Write};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    terminal::{self, ClearType},
    cursor::{self, MoveToColumn},
    style::Print,
};
use clap::Parser;

use dbc::evaluate;

#[derive(Parser)]
#[command(name = "dbc")]
#[command(version)]
#[command(about = "Dollar Calculator with Real-Time Stock Prices", long_about = "A CLI tool to evaluate mathematical expressions with dollar formatting and real-time stock prices (e.g., '$100 + $AAPL').")]
struct Cli {
    /// The expression to evaluate (e.g., "$100 + $AAPL")
    #[arg(value_name = "EXPRESSION")]
    expression: Vec<String>,

    /// Update dbc via homebrew
    #[arg(long)]
    update: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.update {
        println!("Updating dbc via homebrew...");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg("brew update && brew upgrade dbc")
            .status()?;
        
        if status.success() {
            println!("Update successful!");
        } else {
            eprintln!("Update failed. Please ensure homebrew is installed and dbc is tapped.");
        }
        return Ok(());
    }

    if !cli.expression.is_empty() {
        // One-shot mode
        let expr = cli.expression.join(" ");
        match evaluate(&expr) {
            Ok(result) => {
                println!("{}", result);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Interactive mode
    run_repl()
}

fn print_repl_help(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(
        stdout,
        Print("Interactive Mode Shortcuts:\r\n"),
        Print("\r\n  [Functions]\r\n"),
        Print("    sqrt(x), pow(base, exp), abs(x), max(a, b), min(a, b)\r\n"),
        Print("    Stock Symbols: $AAPL, $TSLA, $BTC-USD, etc.\r\n"),
        Print("\r\n  [Cursor Movement]\r\n"),
        Print("    Arrows Left/Right - Move by character\r\n"),
        Print("    Alt+b / Alt+f     - Move back/forward by token\r\n"),
        Print("    Ctrl+a / Ctrl+e   - Move to start/end of line\r\n"),
        Print("\r\n  [Editing]\r\n"),
        Print("    Backspace / Del   - Delete character\r\n"),
        Print("    Alt+d             - Delete next token\r\n"),
        Print("    Ctrl+w            - Delete previous token\r\n"),
        Print("    Ctrl+k / Ctrl+u   - Delete to end/start of line\r\n"),
        Print("\r\n  [History & System]\r\n"),
        Print("    Arrows Up/Down    - Navigate history (loops)\r\n"),
        Print("    Ctrl+l            - Clear screen\r\n"),
        Print("    Ctrl+c / Ctrl+d   - Exit\r\n")
    )
}

struct ReplState {
    buffer: Vec<char>,
    cursor: usize,
    history: Vec<String>,
    history_index: usize,
    temp_buffer: Vec<char>,
}

impl ReplState {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: 0,
            temp_buffer: Vec::new(),
        }
    }

    fn reset_input(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.history_index = self.history.len();
        self.temp_buffer.clear();
    }

    fn update_buffer_from_history(&mut self) {
        if self.history_index < self.history.len() {
            self.buffer = self.history[self.history_index].chars().collect();
        } else {
            self.buffer = self.temp_buffer.clone();
        }
        self.cursor = self.buffer.len();
    }

    fn buffer_string(&self) -> String {
        self.buffer.iter().collect()
    }
}

fn run_repl() -> io::Result<()> {
    println!("dbc - Dollar Calculator (Type /help for help, /exit to exit)");
    
    let mut state = ReplState::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;

    loop {
        // Render prompt and buffer
        let buffer_str: String = state.buffer.iter().collect();
        queue!(
            stdout,
            MoveToColumn(0),
            terminal::Clear(ClearType::UntilNewLine),
            Print("> "),
            Print(&buffer_str),
            MoveToColumn((2 + state.cursor) as u16)
        )?;
        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            // Handle key press and repeat, ignore release
            if key.kind == KeyEventKind::Release {
                continue;
            }

            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) && state.buffer.is_empty() => break,
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if state.cursor < state.buffer.len() {
                        state.buffer.remove(state.cursor);
                    }
                }

                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.cursor = 0;
                }
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.cursor = state.buffer.len();
                }
                KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.buffer.truncate(state.cursor);
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.buffer.drain(0..state.cursor);
                    state.cursor = 0;
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let start = find_prev_token_boundary(&state.buffer, state.cursor);
                    if start < state.cursor {
                        state.buffer.drain(start..state.cursor);
                        state.cursor = start;
                    }
                }
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                }

                KeyCode::Enter => {
                    execute!(stdout, Print("\r\n"))?;
                    
                    let input_raw = state.buffer_string();
                    let input = input_raw.trim();
                    
                    if input == "/exit" || input == "/quit" || input == "exit" || input == "quit" {
                        break;
                    }
                    
                    if input == "/help" {
                        print_repl_help(&mut stdout)?;
                        state.reset_input();
                        continue;
                    }
                    
                    if input == "/clear" {
                        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                        state.reset_input();
                        continue;
                    }

                    if input.is_empty() || input.starts_with('#') {
                        state.reset_input();
                        continue;
                    }

                    let expr = if let Some(pos) = input.find('#') {
                        input[..pos].trim()
                    } else {
                        input
                    };

                    if !expr.is_empty() {
                        match evaluate(expr) {
                            Ok(result) => {
                                execute!(stdout, Print(format!("{}\r\n", result)))?;
                                state.history.push(expr.to_string());
                                state.history.push(result);
                            }
                            Err(e) => {
                                execute!(stdout, Print(format!("Error: {}\r\n", e)))?;
                                state.history.push(expr.to_string());
                            }
                        }
                    }
                    state.reset_input();
                }

                KeyCode::Left => {
                    if state.cursor > 0 {
                        state.cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if state.cursor < state.buffer.len() {
                        state.cursor += 1;
                    }
                }

                KeyCode::Up => {
                    if state.history.is_empty() { continue; }
                    
                    if state.history_index == state.history.len() {
                        state.temp_buffer = state.buffer.clone();
                    }

                    if state.history_index == 0 {
                        // Loop back to current formula
                        state.history_index = state.history.len();
                    } else {
                        state.history_index -= 1;
                    }
                    state.update_buffer_from_history();
                }
                KeyCode::Down => {
                    if state.history.is_empty() { continue; }

                    if state.history_index >= state.history.len() {
                        // Loop to top of history
                        state.history_index = 0;
                    } else {
                        state.history_index += 1;
                    }
                    state.update_buffer_from_history();
                }

                KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                    state.cursor = find_prev_token_boundary(&state.buffer, state.cursor);
                }
                KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                    state.cursor = find_next_token_boundary(&state.buffer, state.cursor);
                }
                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
                    let end = find_next_token_boundary(&state.buffer, state.cursor);
                    if end > state.cursor {
                        state.buffer.drain(state.cursor..end);
                    }
                }

                KeyCode::Backspace => {
                    if state.cursor > 0 {
                        state.buffer.remove(state.cursor - 1);
                        state.cursor -= 1;
                    }
                }
                KeyCode::Delete => {
                    if state.cursor < state.buffer.len() {
                        state.buffer.remove(state.cursor);
                    }
                }
                KeyCode::Char(c) => {
                    state.buffer.insert(state.cursor, c);
                    state.cursor += 1;
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

fn find_prev_token_boundary(chars: &[char], current: usize) -> usize {
    if current == 0 { return 0; }
    let mut i = current;
    while i > 0 && chars[i-1].is_whitespace() { i -= 1; }
    if i == 0 { return 0; }
    let target_type = get_char_type(chars[i-1]);
    while i > 0 && get_char_type(chars[i-1]) == target_type && !chars[i-1].is_whitespace() {
        i -= 1;
    }
    i
}

fn find_next_token_boundary(chars: &[char], current: usize) -> usize {
    if current >= chars.len() { return chars.len(); }
    let mut i = current;
    while i < chars.len() && chars[i].is_whitespace() { i += 1; }
    if i >= chars.len() { return chars.len(); }
    let target_type = get_char_type(chars[i]);
    while i < chars.len() && get_char_type(chars[i]) == target_type && !chars[i].is_whitespace() {
        i += 1;
    }
    i
}

#[derive(PartialEq, Eq)]
enum CharType { Number, Operator, Paren, Whitespace, Other }

fn get_char_type(c: char) -> CharType {
    if c.is_ascii_digit() || "$.,%".contains(c) { CharType::Number }
    else if "+-*/".contains(c) { CharType::Operator }
    else if "()".contains(c) { CharType::Paren }
    else if c.is_whitespace() { CharType::Whitespace }
    else { CharType::Other }
}
