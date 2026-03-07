use std::env;
use std::io::{self, Write};

use dbc::evaluate;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle --help flag
    if args.len() > 1 && args[1] == "--help" {
        print_help();
        return;
    }

    println!("dbc - Dollar Calculator");
    println!("Enter expressions with dollar amounts (e.g., $1,420,368.94 + $1)");
    println!("Type 'quit' or 'exit' to exit.\n");
    println!("Interactive commands:");
    println!("  /help      - Show this help message");
    println!("  /clear     - Clear all records from interactive mode interface");
    println!("  /exit|quit - exit");
    println!("  #          - Start a comment (everything after # is ignored)");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle comment - skip lines starting with #
        if input.starts_with('#') {
            continue;
        }

        // Handle interactive commands starting with /
        if input.starts_with('/') {
            match input {
                "/help" => {
                    print_help();
                }
                "/clear" => {
                    // Clear screen by printing newlines
                    println!("\x1B[2J\x1B[1J");
                }
                "/exit" | "/quit" => {
                    break;
                }
                _ => {
                    println!("Unknown command: {}", input);
                    println!("Available commands: /help, /clear");
                }
            }
            continue;
        }

        if input == "quit" || input == "exit" {
            break;
        }

        // Handle inline comments - everything after # is ignored
        let expr = if let Some(pos) = input.find('#') {
            input[..pos].trim()
        } else {
            input
        };

        if expr.is_empty() {
            continue;
        }

        match evaluate(expr) {
            Ok(result) => println!("{}", result),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn print_help() {
    println!("dbc - Dollar Calculator");
    println!();
    println!("Usage: dbc [--help]");
    println!();
    println!("Options:");
    println!("  --help    Show this help message");
    println!();
    println!("Interactive Mode:");
    println!("  Enter mathematical expressions with dollar amounts.");
    println!("  Examples:");
    println!("    $1,420,368.94 + $1");
    println!("    400 * 4%");
    println!("    $100 - $50");
    println!();
    println!("Interactive Commands:");
    println!("  /help    - Show this help message");
    println!("  /clear   - Clear all records from interactive mode interface");
    println!("  #        - Start a comment (everything after # is ignored)");
    println!();
    println!("Type 'quit' or 'exit' to exit.");
}
