use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;
use std::path::PathBuf;

use td::commands;
use td::config::Config;
use td::output::Printer;
use td::todo_file::TodoFile;

#[derive(Parser)]
#[command(name = "td")]
#[command(author, version, about = "A CLI tool for managing todo.txt files")]
struct Cli {
    /// Path to todo.txt file
    #[arg(short, long, global = true)]
    file: Option<PathBuf>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task description (can include priority, projects, contexts)
        text: Vec<String>,
    },

    /// List tasks (optionally filtered)
    List {
        /// Filter terms (projects, contexts, priorities, or text)
        terms: Vec<String>,

        /// Show completed tasks too
        #[arg(short, long)]
        all: bool,
    },

    /// Alias for list
    Ls {
        /// Filter terms
        terms: Vec<String>,

        /// Show completed tasks too
        #[arg(short, long)]
        all: bool,
    },

    /// Mark tasks as done
    Done {
        /// Task IDs to mark as done
        ids: Vec<usize>,
    },

    /// Delete tasks
    Delete {
        /// Task IDs to delete
        ids: Vec<usize>,
    },

    /// Alias for delete
    Rm {
        /// Task IDs to delete
        ids: Vec<usize>,
    },

    /// Append text to tasks (usage: td append 1 2 3 -- "text to append")
    #[command(trailing_var_arg = true)]
    Append {
        /// Task IDs to modify
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// Prepend text to tasks (usage: td prepend 1 2 3 -- "text to prepend")
    #[command(trailing_var_arg = true)]
    Prepend {
        /// Task IDs to modify
        #[arg(required = true)]
        ids: Vec<String>,
    },

    /// Replace text in tasks
    Replace {
        /// Task IDs to modify
        #[arg(short = 'i', long, required = true, num_args = 1..)]
        ids: Vec<usize>,

        /// Text to find
        #[arg(short = 's', long)]
        from: String,

        /// Text to replace with
        #[arg(short = 'r', long)]
        to: String,
    },

    /// Set priority for tasks (usage: td priority A 1 2 3)
    Priority {
        /// Priority (A-Z) or - to remove
        priority: String,

        /// Task IDs to modify
        ids: Vec<usize>,
    },

    /// Alias for priority
    Pri {
        /// Priority (A-Z) or - to remove
        priority: String,

        /// Task IDs to modify
        ids: Vec<usize>,
    },

    /// Update time context for tasks (usage: td update today 1 2 3)
    Update {
        /// Time context (today, tomorrow, thisweek, nextweek, thismonth, nextmonth, thisquarter, nextquarter)
        context: String,

        /// Task IDs to update
        ids: Vec<usize>,
    },

    /// Run automatic time context transitions
    TimeUpdate,

    /// Archive completed tasks to done.txt
    Archive {
        /// Path to done.txt file (default: done.txt in same directory as todo.txt)
        #[arg(short, long)]
        done_file: Option<PathBuf>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Initialize or show configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Create a default config file
    Init {
        /// Path to create config file (default: ~/.config/td/config.toml)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Show current configuration
    Show,
    /// Show config file path
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let printer = Printer::new(!cli.no_color);

    // Handle commands that don't need todo file
    match &cli.command {
        Commands::Completions { shell } => {
            generate(*shell, &mut Cli::command(), "td", &mut io::stdout());
            return Ok(());
        }
        Commands::Config { action } => {
            return handle_config(action, &printer);
        }
        _ => {}
    }

    let todo_path = match cli.file {
        Some(path) => path,
        None => {
            // Try to load from config first, fall back to default
            let config = Config::load()?;
            if config.todo_file.is_absolute() {
                config.todo_file
            } else if config.todo_file != std::path::PathBuf::from("todo.txt") {
                // Config has a non-default relative path
                config.get_todo_path()
            } else {
                // Use default path discovery
                TodoFile::default_path()?
            }
        }
    };

    match cli.command {
        Commands::Add { text } => {
            let task_text = text.join(" ");
            commands::add(&todo_path, &task_text, &printer)?;
        }

        Commands::List { terms, all } | Commands::Ls { terms, all } => {
            commands::list(&todo_path, &terms, all, &printer)?;
        }

        Commands::Done { ids } => {
            commands::done(&todo_path, &ids, &printer)?;
        }

        Commands::Delete { ids } | Commands::Rm { ids } => {
            commands::delete(&todo_path, &ids, &printer)?;
        }

        Commands::Append { ids } => {
            let (task_ids, text) = parse_trailing_args(&ids)?;
            commands::append(&todo_path, &task_ids, &text, &printer)?;
        }

        Commands::Prepend { ids } => {
            let (task_ids, text) = parse_trailing_args(&ids)?;
            commands::prepend(&todo_path, &task_ids, &text, &printer)?;
        }

        Commands::Replace { ids, from, to } => {
            commands::replace(&todo_path, &ids, &from, &to, &printer)?;
        }

        Commands::Priority { ids, priority } | Commands::Pri { ids, priority } => {
            let p = if priority == "-" {
                None
            } else {
                let c = priority.chars().next().ok_or_else(|| {
                    anyhow::anyhow!("Priority must be a letter A-Z or - to remove")
                })?;
                if !c.is_ascii_uppercase() {
                    anyhow::bail!("Priority must be a letter A-Z");
                }
                Some(c)
            };
            commands::priority(&todo_path, &ids, p, &printer)?;
        }

        Commands::Update { ids, context } => {
            commands::update(&todo_path, &ids, &context, &printer)?;
        }

        Commands::TimeUpdate => {
            commands::update::time_update(&todo_path, &printer)?;
        }

        Commands::Archive { done_file } => {
            let done_path = done_file.unwrap_or_else(|| {
                todo_path
                    .parent()
                    .map(|p| p.join("done.txt"))
                    .unwrap_or_else(|| PathBuf::from("done.txt"))
            });
            commands::archive(&todo_path, &done_path, &printer)?;
        }

        Commands::Completions { .. } | Commands::Config { .. } => {
            unreachable!("Handled above")
        }
    }

    Ok(())
}

fn handle_config(action: &Option<ConfigAction>, printer: &Printer) -> Result<()> {
    match action {
        Some(ConfigAction::Init { path }) => {
            let config_path = path.clone().unwrap_or_else(|| {
                dirs::config_dir()
                    .map(|d| d.join("td").join("config.toml"))
                    .unwrap_or_else(|| PathBuf::from(".tdrc"))
            });

            if config_path.exists() {
                printer.print_warning(&format!(
                    "Config file already exists: {}",
                    config_path.display()
                ));
                return Ok(());
            }

            let config = Config::default();
            config.save_to_file(&config_path)?;
            printer.print_success(&format!("Created config file: {}", config_path.display()));
        }
        Some(ConfigAction::Show) => {
            let config = Config::load()?;
            println!("Current configuration:");
            println!("  todo_file: {}", config.todo_file.display());
            if let Some(ref done) = config.done_file {
                println!("  done_file: {}", done.display());
            }
            println!("  color: {}", config.color);
            println!("  date_on_add: {}", config.date_on_add);
            println!("  auto_archive: {}", config.auto_archive);
        }
        Some(ConfigAction::Path) | None => {
            if let Some(path) = Config::default_path() {
                println!("{}", path.display());
            } else {
                println!("No config file found.");
                println!("Run 'td config init' to create one at:");
                if let Some(config_dir) = dirs::config_dir() {
                    println!("  {}", config_dir.join("td").join("config.toml").display());
                }
            }
        }
    }
    Ok(())
}

/// Parse trailing arguments into task IDs and text
/// Format: 1 2 3 "text" or 1 2 3 -- text with spaces
fn parse_trailing_args(args: &[String]) -> Result<(Vec<usize>, String)> {
    let mut ids = Vec::new();
    let mut text_parts = Vec::new();
    let mut found_text = false;

    for arg in args {
        if found_text || arg == "--" {
            if arg != "--" {
                text_parts.push(arg.clone());
            }
            found_text = true;
        } else if let Ok(id) = arg.parse::<usize>() {
            ids.push(id);
        } else {
            // Start of text
            text_parts.push(arg.clone());
            found_text = true;
        }
    }

    if ids.is_empty() {
        anyhow::bail!("No task IDs provided");
    }

    if text_parts.is_empty() {
        anyhow::bail!("No text provided");
    }

    Ok((ids, text_parts.join(" ")))
}
