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
        let decimal_str = format!("{:.width$}", decimal_part, width = d);
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

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Operator(char),
    LeftParen,
    RightParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let input = input.trim();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Handle numbers (including dollar sign and commas)
        if c.is_ascii_digit() || c == '$' || c == ',' || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            // Collect the number string
            let mut num_str = String::new();
            let mut has_digit = false;
            
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_ascii_digit() || ch == '.' {
                    num_str.push(ch);
                    has_digit = true;
                } else if ch == '$' || ch == ',' {
                    // Include but don't mark as having digit yet
                    num_str.push(ch);
                } else if ch == '%' {
                    // Percentage - append and continue
                    num_str.push(ch);
                    i += 1;
                    break;
                } else {
                    break;
                }
                i += 1;
            }

            if !has_digit {
                return Err("Invalid number".to_string());
            }
            
            // Parse the number
            let cleaned: String = num_str
                .chars()
                .filter(|c| *c != '$' && *c != ',')
                .collect();
            
            // Check if it's a percentage
            let (value, _is_percent) = if cleaned.ends_with('%') {
                let pct_str = &cleaned[..cleaned.len()-1];
                let val: f64 = pct_str.parse().map_err(|_| "Invalid number".to_string())?;
                (val / 100.0, true)
            } else {
                let val: f64 = cleaned.parse().map_err(|_| "Invalid number".to_string())?;
                (val, false)
            };

            tokens.push(Token::Number(value));
            continue;
        }

        // Handle operators
        if "+-*/%".contains(c) {
            tokens.push(Token::Operator(c));
            i += 1;
            continue;
        }

        // Handle parentheses
        if c == '(' {
            tokens.push(Token::LeftParen);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(Token::RightParen);
            i += 1;
            continue;
        }

        return Err(format!("Unexpected character: {}", c));
    }

    Ok(tokens)
}

// Convert infix tokens to Reverse Polish Notation (RPN) using Shunting-yard algorithm
fn to_rpn(tokens: Vec<Token>) -> Result<Vec<Token>, String> {
    let mut output: Vec<Token> = Vec::new();
    let mut operators: Vec<char> = Vec::new();

    // Operator precedence
    let precedence = |op: char| -> u8 {
        match op {
            '+' | '-' => 1,
            '*' | '/' | '%' => 2,
            _ => 0,
        }
    };

    for token in tokens {
        match token {
            Token::Number(_) => {
                output.push(token);
            }
            Token::Operator(op) => {
                while let Some(&top) = operators.last() {
                    if top != '(' && precedence(top) >= precedence(op) {
                        output.push(Token::Operator(operators.pop().unwrap()));
                    } else {
                        break;
                    }
                }
                operators.push(op);
            }
            Token::LeftParen => {
                operators.push('(');
            }
            Token::RightParen => {
                let mut found_left_paren = false;
                while let Some(top) = operators.pop() {
                    if top == '(' {
                        found_left_paren = true;
                        break;
                    }
                    output.push(Token::Operator(top));
                }
                if !found_left_paren {
                    return Err("Mismatched parentheses".to_string());
                }
            }
        }
    }

    while let Some(op) = operators.pop() {
        if op == '(' {
            return Err("Mismatched parentheses".to_string());
        }
        output.push(Token::Operator(op));
    }

    Ok(output)
}

// Evaluate RPN tokens
fn evaluate_rpn(tokens: Vec<Token>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(val) => {
                stack.push(val);
            }
            Token::Operator(op) => {
                if stack.len() < 2 {
                    return Err("Invalid expression".to_string());
                }
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                let result = match op {
                    '+' => left + right,
                    '-' => left - right,
                    '*' => left * right,
                    '/' => {
                        if right == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        left / right
                    }
                    '%' => left % right,
                    _ => return Err("Unknown operator".to_string()),
                };
                stack.push(result);
            }
            Token::LeftParen | Token::RightParen => {
                return Err("Unexpected parentheses".to_string());
            }
        }
    }

    if stack.len() != 1 {
        return Err("Invalid expression".to_string());
    }

    Ok(stack[0])
}

// Check if expression ends with % (percentage of something)
fn has_percentage_suffix(input: &str) -> bool {
    let trimmed = input.trim();
    // Find last number-like token and check if it ends with %
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = chars.len();
    
    // Skip whitespace
    while i > 0 && chars[i-1].is_whitespace() {
        i -= 1;
    }
    
    // Check if ends with %
    if i > 0 && chars[i-1] == '%' {
        return true;
    }
    
    false
}

/// Evaluate a mathematical expression string and return the formatted result.
///
/// Supports dollar amounts (e.g., `$1,420,368.94`), basic arithmetic operators
/// (`+`, `-`, `*`, `/`, `%`), parentheses, and percentages (e.g., `400 * 4%`).
///
/// The result is formatted to match the style of the first number in the expression
/// (dollar sign, commas, decimal places).
pub fn evaluate(input: &str) -> Result<String, String> {
    let input = input.trim();

    // Handle percentage suffix (e.g., "400 * 4%")
    // In this case, 4% means 4/100 = 0.04
    let expr: String = if has_percentage_suffix(input) {
        // Replace % with /100 for the last number
        let chars: Vec<char> = input.chars().collect();
        let mut result = String::new();
        let mut in_number = false;
        let mut last_num_start = 0;
        
        for (i, &c) in chars.iter().enumerate() {
            if c.is_ascii_digit() || c == '.' {
                if !in_number {
                    last_num_start = i;  // Track start of number
                }
                in_number = true;
            } else if in_number && c == '%' {
                // Found percentage - replace with /100
                result.push_str(&input[last_num_start..i]);
                result.push_str("/100");
                in_number = false;
            } else if c == '$' || c == ',' {
                // Just continue, don't change in_number state
            } else {
                if in_number {
                    // End of a number, copy it
                    result.push_str(&input[last_num_start..i]);
                }
                // Copy the current character (operator, space, etc)
                result.push(c);
                in_number = false;
            }
        }
        
        // Handle case where expression ends with a number (no % to replace)
        if !result.is_empty() && result.len() < input.len() {
            // We already handled the percentage replacement
            result
        } else if result.is_empty() {
            input.to_string()
        } else {
            result
        }
    } else {
        input.to_string()
    };

    // Tokenize the expression
    let tokens = tokenize(&expr)?;
    
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Convert to RPN
    let rpn = to_rpn(tokens)?;

    // Find first number in original expression to get format
    // Skip leading parentheses and whitespace to find the actual first number
    let trimmed_for_format = input.trim_start_matches(|c: char| c == '(' || c.is_whitespace());
    let first_num_str: String = trimmed_for_format
        .chars()
        .take_while(|&c| !"+*-/()% ".contains(c))
        .collect();
    let (_, format) = parse_number(&first_num_str)
        .ok_or_else(|| "Invalid number format".to_string())?;

    // Evaluate RPN
    let result = evaluate_rpn(rpn)?;

    Ok(format_number(result, &format))
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

    // === Regression tests for Bug #1: Parentheses were always erroring ===

    #[test]
    fn test_simple_parentheses() {
        let result = evaluate("(1 + 2) * 3").unwrap();
        assert_eq!(result, "9");
    }

    #[test]
    fn test_parentheses_with_dollars() {
        let result = evaluate("($10 + $5) * 2").unwrap();
        assert_eq!(result, "$30");
    }

    #[test]
    fn test_nested_parentheses() {
        let result = evaluate("((2 + 3) * 2) + 1").unwrap();
        assert_eq!(result, "11");
    }

    #[test]
    fn test_mismatched_paren_open() {
        let result = evaluate("(1 + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_paren_close() {
        let result = evaluate("1 + 2)");
        assert!(result.is_err());
    }

    // === Regression tests for Bug #2: Multi-operator expressions ===

    #[test]
    fn test_multi_operator_addition() {
        let result = evaluate("1+2+3+$1").unwrap();
        assert_eq!(result, "7");
    }

    #[test]
    fn test_multi_operator_with_dollars() {
        let result = evaluate("$1,420,368.94 + $1 + $1").unwrap();
        assert_eq!(result, "$1,420,370.94");
    }

    #[test]
    fn test_multi_operator_mixed_precedence() {
        // $10 * 2 + $5 * 3 = 20 + 15 = 35
        let result = evaluate("$10 * 2 + $5 * 3").unwrap();
        assert_eq!(result, "$35");
    }

    #[test]
    fn test_multi_operator_subtraction_addition() {
        let result = evaluate("$100 - $50 + $25").unwrap();
        assert_eq!(result, "$75");
    }

    #[test]
    fn test_operator_precedence() {
        // $10 + $5 * 2 = 10 + 10 = 20
        let result = evaluate("$10 + $5 * 2").unwrap();
        assert_eq!(result, "$20");
    }
}
