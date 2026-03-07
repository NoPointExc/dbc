use std::env;
use std::io::{self, Write};

use dbc::evaluate;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle help flags
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return;
    }

    // One-shot mode: if arguments are provided, evaluate them as an expression
    if args.len() > 1 {
        let expr = args[1..].join(" ");
        match evaluate(&expr) {
            Ok(result) => {
                println!("{}", result);
                return;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Interactive mode
    println!("dbc - Dollar Calculator (Type /help for help, /exit to exit)");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            break;
        }
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
                    // Clear screen using ANSI escape codes
                    print!("\x1B[2J\x1B[H");
                    io::stdout().flush().unwrap();
                }
                "/exit" | "/quit" => {
                    break;
                }
                _ => {
                    println!("Unknown command: {}", input);
                    println!("Available commands: /help, /clear, /exit");
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
    println!("Usage:");
    println!("  dbc [expression]    Evaluate the expression and exit");
    println!("  dbc                 Enter interactive mode");
    println!("  dbc --help, -h      Show this help message");
    println!();
    println!("Examples:");
    println!("  dbc \"$1,200.50 + $300\"");
    println!("  dbc \"400 * 5%\"");
    println!();
    println!("Interactive Commands:");
    println!("  /help    - Show this help message");
    println!("  /clear   - Clear the screen");
    println!("  /exit    - Exit the calculator");
    println!("  #        - Comments (ignored)");
}
