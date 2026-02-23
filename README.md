# td - Todo.txt CLI Manager

A command-line tool for managing todo.txt files with automatic time context management.

## Features

- **Task Management**: Add, delete, modify, and complete tasks
- **Batch Operations**: Apply operations to multiple tasks by ID
- **Smart Filtering**: Filter by projects (`+project`), contexts (`@context`), and priorities (`A-B`)
- **Time Contexts**: Automatic management of time-based contexts (`@today`, `@tomorrow`, `@thisweek`, etc.)

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- A todo.txt file (default location: `~/todo.txt`)

### Building from Source

```bash
# Clone or navigate to the project directory
cd td

# Build the release version
cargo build --release

# The binary will be at ./target/release/td
# Optionally, copy it to your PATH
cp ./target/release/td ~/.local/bin/
```

## Usage

### Adding Tasks

```bash
td add "(A) Download files @admin +agtk @today"
td add "Call mom +family @phone"
td add "(B) Review report +work @tomorrow"
```

### Completing Tasks

Mark tasks as done by their line numbers:

```bash
td done 23 45 12
```

### Deleting Tasks

```bash
td delete 5 10 15
td rm 5 10 15  # alias
```

### Modifying Tasks

Append text to tasks:

```bash
td append 76 42 "+projectx"
td append 1 2 3 "some text to append"
```

Prepend text to tasks:

```bash
td prepend 76 42 "(A)"
```

Replace text in tasks:

```bash
td replace --ids 5 --from "@today" --to "@tomorrow"
td replace -i 5 -s "@today" -r "@tomorrow"
```

Set priority for tasks:

```bash
td priority A 1 2 3    # Set priority A for tasks 1, 2, 3
td pri B 5             # Alias
td priority - 1 2      # Remove priority from tasks 1, 2
```

Update time context for multiple tasks:

```bash
td update today 34 66    # Set tasks 34 and 66 to @today
td update nextweek 12    # Set task 12 to @nextweek
```

### Listing and Filtering Tasks

List all tasks:

```bash
td list
```

Filter by project:

```bash
td list +agtk
td list agtk  # + is optional for projects
```

Filter by context:

```bash
td list @today
td list today  # @ is optional for contexts
```

Filter by multiple terms:

```bash
td list today tomorrow
td list +work @urgent
```

Filter by priority range:

```bash
td list A-B     # priorities A and B
td list A-C     # priorities A, B, and C
```

## Time Context System

The tool manages time-based contexts automatically:

| Context | Description |
|---------|-------------|
| `@today` | Tasks for today |
| `@tomorrow` | Tasks for tomorrow |
| `@thisweek` | Tasks for this week |
| `@nextweek` | Tasks for next week |
| `@thismonth` | Tasks for this month |
| `@nextmonth` | Tasks for next month |
| `@thisquarter` | Tasks for this quarter |
| `@nextquarter` | Tasks for next quarter |

### Automatic Transitions

Run the update command to process automatic time context transitions:

```bash
td time-update
```

The transitions follow these rules:

- **Daily (each morning)**: `@tomorrow` → `@today`
- **Friday morning**: `@thisweek` → `@today`
- **Monday morning**: `@nextweek` → `@thisweek`
- **Last week of month**: `@thismonth` → `@thisweek`
- **First of month**: `@nextmonth` → `@thismonth`
- **Last month of quarter**: `@thisquarter` → `@thismonth`
- **First of quarter**: `@nextquarter` → `@thisquarter`

### Manual Time Context Updates

Move tasks to a different time context:

```bash
td update nextweek 34 66    # Set tasks 34 and 66 to @nextweek
td update today 12          # Set task 12 to @today
```

## Configuration

By default, `td` looks for `todo.txt` in the current directory, then falls back to `~/todo.txt`.

You can specify a different file:

```bash
td -f /path/to/my/todo.txt list
td --file /path/to/my/todo.txt add "New task"
```

### Config File

Create a config file to set default options:

```bash
td config init              # Create config at ~/.config/td/config.toml
td config show              # Show current configuration
td config path              # Show config file path
```

Example config file (`~/.config/td/config.toml`):

```toml
todo_file = "~/todo.txt"
done_file = "~/done.txt"
color = true
date_on_add = true
auto_archive = false
```

## Shell Completions

Generate shell completions for your shell:

```bash
# Bash
td completions bash > ~/.local/share/bash-completion/completions/td

# Zsh
td completions zsh > ~/.zfunc/_td

# Fish
td completions fish > ~/.config/fish/completions/td.fish
```

## Todo.txt Format

This tool follows the [todo.txt format specification](https://github.com/todotxt/todo.txt):

```
(A) 2024-01-15 Call mom +family @phone
x 2024-01-14 2024-01-10 Completed task +project @context
```

### Format Rules

- **Priority**: `(A)` through `(Z)` at line start
- **Creation date**: `YYYY-MM-DD` after priority
- **Projects**: Prefixed with `+` (e.g., `+work`)
- **Contexts**: Prefixed with `@` (e.g., `@home`)
- **Completion**: Line starts with `x ` followed by completion date

## Dependencies

- `clap` - Command-line argument parsing
- `chrono` - Date and time handling
- `regex` - Pattern matching for todo.txt parsing
- `colored` - Terminal output coloring
- `anyhow` / `thiserror` - Error handling
- `dirs` - Home directory detection
- `lazy_static` - Lazy regex initialization

## Development

```bash
# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- list

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## License

MIT License
