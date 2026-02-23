use crate::task::Task;
use colored::*;

pub struct Printer {
    pub color: bool,
}

impl Printer {
    pub fn new(color: bool) -> Self {
        Printer { color }
    }

    pub fn print_task(&self, task: &Task) {
        let id_str = format!("{:>3}", task.id);
        let task_str = self.format_task(task);

        if self.color {
            print!("{} ", id_str.dimmed());
        } else {
            print!("{} ", id_str);
        }
        println!("{}", task_str);
    }

    pub fn format_task(&self, task: &Task) -> String {
        if !self.color {
            return task.to_string();
        }

        let base = task.to_string();

        if task.is_completed {
            return base.dimmed().strikethrough().to_string();
        }

        let colored = match task.priority {
            Some('A') => base.red().bold(),
            Some('B') => base.yellow(),
            Some('C') => base.blue(),
            Some('D') => base.cyan(),
            _ => base.normal(),
        };

        colored.to_string()
    }

    pub fn print_tasks(&self, tasks: &[&Task]) {
        for task in tasks {
            self.print_task(task);
        }
    }

    pub fn print_summary(&self, total: usize, completed: usize) {
        let pending = total - completed;
        if self.color {
            println!(
                "\n{}: {} total, {} pending, {} completed",
                "Summary".bold(),
                total,
                pending.to_string().green(),
                completed.to_string().dimmed()
            );
        } else {
            println!(
                "\nSummary: {} total, {} pending, {} completed",
                total, pending, completed
            );
        }
    }

    pub fn print_success(&self, message: &str) {
        if self.color {
            println!("{} {}", "✓".green(), message);
        } else {
            println!("OK: {}", message);
        }
    }

    pub fn print_error(&self, message: &str) {
        if self.color {
            eprintln!("{} {}", "✗".red(), message.red());
        } else {
            eprintln!("Error: {}", message);
        }
    }

    pub fn print_warning(&self, message: &str) {
        if self.color {
            println!("{} {}", "!".yellow(), message.yellow());
        } else {
            println!("Warning: {}", message);
        }
    }
}

impl Default for Printer {
    fn default() -> Self {
        Printer::new(true)
    }
}
