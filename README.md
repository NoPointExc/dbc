# DBC - Dollar Calculator CLI Tool

A command-line calculator for dollar number calculations with flexible number format support.

## Features

- **Basic Math Operations**: +, -, *, /, %
- **Flexible Number Formats**: Supports $ prefix and comma separators
  - `$1,420,368.94`
  - `1,420,368.94`
  - `1420368.94`
- **Format Preservation**: Output maintains the same format as the first operand
- **Interactive REPL Mode**: Enter expressions interactively

## Installation

```bash
cd dbc
cargo build --release
```

## Usage

### Interactive Mode

Run the calculator without arguments to enter interactive mode:

```bash
dbc
```

### Command-Line Arguments

```
dbc [--help] [expression]
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

> $100 - $50
$50

> 420368.94 + $2
420370.94
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
