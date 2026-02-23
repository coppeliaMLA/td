use crate::output::Printer;
use crate::todo_file::TodoFile;
use anyhow::{bail, Result};
use std::path::Path;

pub fn delete(path: &Path, ids: &[usize], printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    // Validate all IDs exist first
    let mut valid_ids = Vec::new();
    let mut invalid_ids = Vec::new();

    for &id in ids {
        if todo.get_task(id).is_some() {
            valid_ids.push(id);
        } else {
            invalid_ids.push(id);
        }
    }

    if !invalid_ids.is_empty() {
        printer.print_error(&format!("Tasks not found: {:?}", invalid_ids));
    }

    if valid_ids.is_empty() {
        bail!("No valid tasks to delete");
    }

    // Store task descriptions before deletion
    let deleted_tasks: Vec<String> = valid_ids
        .iter()
        .filter_map(|&id| todo.get_task(id).map(|t| format!("{}: {}", id, t.description.clone())))
        .collect();

    todo.remove_tasks(&valid_ids);
    todo.save()?;

    for task in deleted_tasks {
        printer.print_success(&format!("Deleted {}", task));
    }

    Ok(())
}
