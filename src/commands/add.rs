use crate::task::Task;
use crate::todo_file::TodoFile;
use crate::output::Printer;
use anyhow::Result;
use std::path::Path;

pub fn add(path: &Path, text: &str, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load_or_create(path)?;

    let task = Task::new(text);
    let task_str = task.to_string();

    todo.add_task(task);
    todo.save()?;

    printer.print_success(&format!("Added: {}", task_str));

    Ok(())
}
