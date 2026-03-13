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
        // Format with exactly the original decimal places
        let decimal_str = format!("{:.width$}", decimal_part, width = d);
        // Remove leading "0."
        let decimal_str = if decimal_str.starts_with("0.") {
            &decimal_str[2..]
        } else if decimal_str.starts_with('.') {
            &decimal_str[1..]
        } else {
            &decimal_str
        };
        format!(".{}", decimal_str)
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
    Function(String),
    LeftParen,
    RightParen,
    Comma,
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
        if c.is_ascii_digit() || c == '$' || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            // Collect the number string
            let mut num_str = String::new();
            let mut has_digit = false;
            
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_ascii_digit() || ch == '.' {
                    num_str.push(ch);
                    has_digit = true;
                } else if ch == '$' {
                    num_str.push(ch);
                } else if ch == ',' {
                    // Lookahead: is this a thousands separator?
                    // Must be followed by exactly 3 digits then a non-digit or end
                    let mut is_thousands = false;
                    if i + 3 < chars.len() && 
                       chars[i+1].is_ascii_digit() && 
                       chars[i+2].is_ascii_digit() && 
                       chars[i+3].is_ascii_digit() {
                        if i + 4 == chars.len() || !chars[i+4].is_ascii_digit() {
                            is_thousands = true;
                        }
                    }
                    
                    if is_thousands {
                        num_str.push(ch);
                    } else {
                        // Not a thousands separator, stop number parsing here
                        break;
                    }
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

        // Handle functions and alphabetic operators
        if c.is_alphabetic() {
            let mut name = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                name.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::Function(name.to_lowercase()));
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

        // Handle comma (as argument separator)
        if c == ',' {
            tokens.push(Token::Comma);
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
    let mut operators: Vec<Token> = Vec::new();

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
            Token::Function(_) => {
                operators.push(token);
            }
            Token::Comma => {
                let mut found_left_paren = false;
                while let Some(top) = operators.last() {
                    if matches!(top, Token::LeftParen) {
                        found_left_paren = true;
                        break;
                    }
                    output.push(operators.pop().unwrap());
                }
                if !found_left_paren {
                    return Err("Comma misplaced or mismatched parentheses".to_string());
                }
            }
            Token::Operator(op) => {
                while let Some(top_token) = operators.last() {
                    match top_token {
                        Token::Operator(top_op) => {
                            if precedence(*top_op) >= precedence(op) {
                                output.push(operators.pop().unwrap());
                            } else {
                                break;
                            }
                        }
                        Token::Function(_) => {
                            output.push(operators.pop().unwrap());
                        }
                        _ => break,
                    }
                }
                operators.push(Token::Operator(op));
            }
            Token::LeftParen => {
                operators.push(Token::LeftParen);
            }
            Token::RightParen => {
                let mut found_left_paren = false;
                while let Some(top) = operators.pop() {
                    if matches!(top, Token::LeftParen) {
                        found_left_paren = true;
                        break;
                    }
                    output.push(top);
                }
                if !found_left_paren {
                    return Err("Mismatched parentheses".to_string());
                }
                if let Some(Token::Function(_)) = operators.last() {
                    output.push(operators.pop().unwrap());
                }
            }
        }
    }

    while let Some(op) = operators.pop() {
        if matches!(op, Token::LeftParen) {
            return Err("Mismatched parentheses".to_string());
        }
        output.push(op);
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
            Token::Function(name) => {
                match name.as_str() {
                    "sqrt" => {
                        if stack.is_empty() { return Err("sqrt requires 1 argument".to_string()); }
                        let val = stack.pop().unwrap();
                        stack.push(val.sqrt());
                    }
                    "abs" => {
                        if stack.is_empty() { return Err("abs requires 1 argument".to_string()); }
                        let val = stack.pop().unwrap();
                        stack.push(val.abs());
                    }
                    "pow" => {
                        if stack.len() < 2 { return Err("pow requires 2 arguments".to_string()); }
                        let exponent = stack.pop().unwrap();
                        let base = stack.pop().unwrap();
                        stack.push(base.powf(exponent));
                    }
                    "max" => {
                        if stack.len() < 2 { return Err("max requires at least 2 arguments".to_string()); }
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a.max(b));
                    }
                    "min" => {
                        if stack.len() < 2 { return Err("min requires at least 2 arguments".to_string()); }
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        stack.push(a.min(b));
                    }
                    _ => return Err(format!("Unknown function: {}", name)),
                }
            }
            Token::LeftParen | Token::RightParen | Token::Comma => {
                return Err("Unexpected token during evaluation".to_string());
            }
        }
    }

    if stack.len() != 1 {
        return Err("Invalid expression".to_string());
    }

    Ok(stack[0])
}

/// Evaluate a mathematical expression string and return the formatted result.
pub fn evaluate(input: &str) -> Result<String, String> {
    let input = input.trim();

    // Tokenize the expression
    let tokens = tokenize(input)?;
    
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Convert to RPN
    let rpn = to_rpn(tokens)?;

    // Find first number in original expression to get format
    let trimmed_for_format = input.trim_start_matches(|c: char| !c.is_ascii_digit() && c != '$' && c != '.');
    let first_num_str: String = trimmed_for_format
        .chars()
        .take_while(|&c| c.is_ascii_digit() || c == '$' || c == ',' || c == '.')
        .collect();
    
    let format = if let Some((_, fmt)) = parse_number(&first_num_str) {
        fmt
    } else {
        NumberFormat { has_dollar: false, has_commas: false, decimal_places: 0 }
    };

    // Evaluate RPN
    let result = evaluate_rpn(rpn)?;

    Ok(format_number(result, &format))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(evaluate("sqrt(9)").unwrap(), "3");
        assert_eq!(evaluate("sqrt($16)").unwrap(), "$4");
    }

    #[test]
    fn test_pow() {
        assert_eq!(evaluate("pow(2, 3)").unwrap(), "8");
    }

    #[test]
    fn test_max_min() {
        assert_eq!(evaluate("max(10, 20)").unwrap(), "20");
        assert_eq!(evaluate("min($10, $20)").unwrap(), "$10");
        assert_eq!(evaluate("max(1, 2)").unwrap(), "2");
        assert_eq!(evaluate("max(1,2)").unwrap(), "2");
    }

    #[test]
    fn test_complex_func() {
        assert_eq!(evaluate("sqrt(pow(3, 2) + pow(4, 2))").unwrap(), "5");
    }

    #[test]
    fn test_parse_with_dollar_and_commas() {
        let (val, fmt) = parse_number("$1,420,368.94").unwrap();
        assert_eq!(val, 1420368.94);
        assert!(fmt.has_dollar);
        assert!(fmt.has_commas);
        assert_eq!(fmt.decimal_places, 2);
    }

    #[test]
    fn test_addition() {
        let result = evaluate("$1,420,368.94 + $1").unwrap();
        assert_eq!(result, "$1,420,369.94");
    }

    #[test]
    fn test_simple_parentheses() {
        let result = evaluate("(1 + 2) * 3").unwrap();
        assert_eq!(result, "9");
    }
}
