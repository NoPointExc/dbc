use std::env;
use std::io::{self, Write};

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

#[derive(Debug, Clone)]
struct NumberFormat {
    has_dollar: bool,
    has_commas: bool,
    decimal_places: usize,
}

fn parse_number(input: &str) -> Option<(f64, NumberFormat)> {
    let original = input.trim();
    let s = original;

    // Check if has dollar sign
    let has_dollar = s.starts_with('$');

    // Check if has commas
    let has_commas = s.contains(',');

    // Remove $ and commas for parsing
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '$' && *c != ',')
        .collect();

    // Count decimal places
    let decimal_places = if let Some(pos) = cleaned.find('.') {
        cleaned[pos + 1..].len()
    } else {
        0
    };

    // Parse the number
    let value: f64 = cleaned.parse().ok()?;

    Some((value, NumberFormat {
        has_dollar,
        has_commas,
        decimal_places,
    }))
}

fn format_number(value: f64, format: &NumberFormat) -> String {
    let abs_value = value.abs();
    let sign = if value < 0.0 { "-" } else { "" };

    // Round to appropriate decimal places first
    let d = if format.decimal_places > 2 { 2 } else { format.decimal_places };
    let rounded = if d > 0 {
        let multiplier = 10_f64.powi(d as i32);
        (abs_value * multiplier).round() / multiplier
    } else {
        abs_value.round()
    };

    // Split into integer and decimal parts after rounding
    let int_part = rounded.trunc() as u64;
    let decimal_part = rounded.fract();

    // Format decimal part
    let decimals = if format.decimal_places > 0 {
        // Format with exactly the original decimal places, then remove trailing zeros
        let decimal_str = format!("{:.width$}", decimal_part, width = d + 1);
        // Remove leading "0." and trailing zeros
        let decimal_str = decimal_str.trim_start_matches("0.").trim_end_matches('0');
        if decimal_str.is_empty() {
            String::new()
        } else {
            format!(".{}", decimal_str)
        }
    } else {
        String::new()
    };

    let num_str = if format.has_commas {
        // Format with commas
        let int_str = int_part.to_string();
        let chars: Vec<char> = int_str.chars().collect();
        let mut result = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*c);
        }
        result
    } else {
        int_part.to_string()
    };

    let prefix = if format.has_dollar { "$" } else { "" };
    format!("{}{}{}{}", sign, prefix, num_str, decimals)
}

fn evaluate(input: &str) -> Result<String, String> {
    let input = input.trim();

    // Special handling for % at the end (percentage operation)
    let has_percent_suffix = input.ends_with('%');
    let expr = if has_percent_suffix {
        &input[..input.len()-1]
    } else {
        input
    };

    // Simple approach: find first operator that separates two numbers
    // Operators: +, -, *, /, %
    let operators = ['+', '-', '*', '/', '%'];
    
    let mut op_pos: Option<(usize, char)> = None;
    let chars: Vec<char> = expr.chars().collect();
    
    for (i, &c) in chars.iter().enumerate() {
        if operators.contains(&c) {
            // Skip if at position 0 (would be negative number)
            if i == 0 {
                continue;
            }
            // Make sure there's a number before and after
            let before = chars[..i].iter().filter(|&c| !c.is_whitespace()).collect::<String>();
            let after = chars[i+1..].iter().filter(|&c| !c.is_whitespace()).collect::<String>();
            
            if !before.is_empty() && !after.is_empty() {
                op_pos = Some((i, c));
                break;
            }
        }
    }

    let (left_str, right_str) = if let Some((pos, _)) = op_pos {
        (&expr[..pos], &expr[pos+1..])
    } else {
        return Err("No operator found".to_string());
    };

    let left_str = left_str.trim();
    let right_str = right_str.trim();

    // Parse both numbers
    let (left_val, left_format) = parse_number(left_str)
        .ok_or_else(|| format!("Invalid number: {}", left_str))?;
    
    let (right_val, _) = parse_number(right_str)
        .ok_or_else(|| format!("Invalid number: {}", right_str))?;

    // Perform calculation
    let result = if has_percent_suffix {
        // Calculate percentage: left * right / 100
        left_val * right_val / 100.0
    } else {
        let (_, op) = op_pos.unwrap();
        match op {
            '+' => left_val + right_val,
            '-' => left_val - right_val,
            '*' => left_val * right_val,
            '/' => {
                if right_val == 0.0 {
                    return Err("Division by zero".to_string());
                }
                left_val / right_val
            }
            '%' => left_val % right_val,
            _ => return Err("Unknown operator".to_string()),
        }
    };

    Ok(format_number(result, &left_format))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_dollar_and_commas() {
        let (val, fmt) = parse_number("$1,420,368.94").unwrap();
        assert_eq!(val, 1420368.94);
        assert!(fmt.has_dollar);
        assert!(fmt.has_commas);
        assert_eq!(fmt.decimal_places, 2);
    }

    #[test]
    fn test_parse_with_commas() {
        let (val, fmt) = parse_number("1,420,368.94").unwrap();
        assert_eq!(val, 1420368.94);
        assert!(!fmt.has_dollar);
        assert!(fmt.has_commas);
    }

    #[test]
    fn test_parse_without_format() {
        let (val, fmt) = parse_number("1420368.94").unwrap();
        assert_eq!(val, 1420368.94);
        assert!(!fmt.has_dollar);
        assert!(!fmt.has_commas);
    }

    #[test]
    fn test_addition() {
        let result = evaluate("$1,420,368.94 + $1").unwrap();
        assert_eq!(result, "$1,420,369.94");
    }

    #[test]
    fn test_percentage() {
        let result = evaluate("400 * 4%").unwrap();
        assert_eq!(result, "16");
    }

    #[test]
    fn test_format_preservation() {
        // First number has $ and commas, second doesn't
        let result = evaluate("420368.94 + $2").unwrap();
        // Should use format of first number (no $)
        assert_eq!(result, "420370.94");
    }
}
