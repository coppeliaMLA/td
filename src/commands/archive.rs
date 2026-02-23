use crate::output::Printer;
use crate::todo_file::TodoFile;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn archive(todo_path: &Path, done_path: &Path, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(todo_path)?;

    let completed: Vec<_> = todo.tasks.iter().filter(|t| t.is_completed).collect();

    if completed.is_empty() {
        printer.print_warning("No completed tasks to archive");
        return Ok(());
    }

    // Append completed tasks to done.txt
    let mut done_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(done_path)?;

    for task in &completed {
        writeln!(done_file, "{}", task)?;
    }

    let archived_count = completed.len();

    // Remove completed tasks from todo.txt
    let completed_ids: Vec<usize> = completed.iter().map(|t| t.id).collect();
    todo.remove_tasks(&completed_ids);
    todo.save()?;

    printer.print_success(&format!(
        "Archived {} completed tasks to {}",
        archived_count,
        done_path.display()
    ));

    Ok(())
}
