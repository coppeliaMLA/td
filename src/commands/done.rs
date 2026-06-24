use crate::output::Printer;
use crate::todo_file::TodoFile;
use anyhow::{bail, Result};
use std::path::Path;

pub fn done(path: &Path, ids: &[usize], printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;
    let today = chrono::Local::now().date_naive();

    let mut marked = Vec::new();
    let mut not_found = Vec::new();
    let mut new_tasks = Vec::new();
    let mut rec_without_due = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            if task.is_completed {
                printer.print_warning(&format!("Task {} is already completed", id));
            } else {
                task.mark_done();
                marked.push(id);

                // Spawn the next occurrence of a recurring task.
                if task.recurrence().is_some() {
                    match task.next_occurrence(today) {
                        Some(next) => new_tasks.push(next),
                        None => rec_without_due.push(id),
                    }
                }
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

    // Add the recurring successors, remembering their rendered form to report.
    let mut created = Vec::new();
    for task in new_tasks {
        let rendered = task.to_string();
        todo.add_task(task);
        created.push(rendered);
    }

    todo.save()?;

    for id in &marked {
        printer.print_success(&format!("Marked task {} as done", id));
    }

    for rendered in &created {
        printer.print_success(&format!("Created recurring task: {}", rendered));
    }

    for id in &rec_without_due {
        printer.print_warning(&format!(
            "Task {} has a rec: tag but no due: date; no recurring task created",
            id
        ));
    }

    Ok(())
}
