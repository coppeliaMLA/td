use crate::output::Printer;
use crate::todo_file::TodoFile;
use anyhow::{bail, Result};
use std::path::Path;

pub fn done(path: &Path, ids: &[usize], printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    let mut marked = Vec::new();
    let mut not_found = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            if task.is_completed {
                printer.print_warning(&format!("Task {} is already completed", id));
            } else {
                task.mark_done();
                marked.push(id);
            }
        } else {
            not_found.push(id);
        }
    }

    if !not_found.is_empty() {
        printer.print_error(&format!("Tasks not found: {:?}", not_found));
    }

    if marked.is_empty() {
        bail!("No tasks were marked as done");
    }

    todo.save()?;

    for id in &marked {
        printer.print_success(&format!("Marked task {} as done", id));
    }

    Ok(())
}
