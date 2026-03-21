# $dbc - $Dollar Calculator CLI

A command-line calculator for dollar number calculations with flexible number format support.

## Features

- **Basic Math Operations**: +, -, *, /, %
- **Mathematical Functions**: `sqrt(x)`, `pow(base, exp)`, `abs(x)`, `max(a, b)`, `min(a, b)`
- **Real-Time Stock Prices**: Use `$SYMBOL` as a variable to fetch the current market price
  - `$AAPL`, `$TSLA`, `$BTC-USD`
- **Flexible Number Formats**: Supports $ prefix and comma separators
  - `$1,420,368.94`
  - `1,420,368.94`
  - `1420368.94`
- **Format Preservation**: Output matches the format (dollar sign, commas, decimal precision) of the first number in the expression.
- **Interactive REPL Mode**: Enter expressions interactively.
- **One-Shot Mode**: Evaluate expressions directly from command-line arguments.

## Installation

### Via Homebrew (macOS/Linux)

```bash
brew tap NoPointExc/homebrew-tap
brew install dbc
```

### From Source (Requires Rust)

```bash
git clone https://github.com/NoPointExc/dbc.git
cd dbc
cargo install --path .
```

## Usage

### One-Shot Mode (Command-Line Arguments)

Pass an expression as arguments to evaluate it and exit:

```bash
dbc "$1,420,368.94 + $1"
# Output: $1,420,369.94

dbc "400 * 5%"
# Output: 20

dbc "sqrt(9)"
# Output: 3
```

*Tip: Always use single quotes ('...') or double quotes ("...") around expressions in your shell to avoid interpretation of characters like `$` or `*`.*

### Interactive Mode

Run `dbc` without arguments to enter the interactive REPL:

```bash
dbc
```

### Interactive Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help message |
| `/clear` | Clear the screen |
| `/exit` | Exit the calculator |
| `#` | Start a comment (everything after # is ignored) |

### Examples

```
> $1,420,368.94 + $1
$1,420,369.94

> 400 * 4%
16

> max($100, $50)
$100

> sqrt(pow(3, 2) + pow(4, 2))
5
```

## Number Format Handling

| Input | Parsed Value |
|-------|--------------|
| `$1,420,368.94` | 1420368.94 |
| `1,420,368.94` | 1420368.94 |
| `1420368.94` | 1420368.94 |

Output format matches the first operand in the expression.

## License

MIT
