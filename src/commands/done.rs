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
    let mut rec_warnings = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            if task.is_completed {
                printer.print_warning(&format!("Task {} is already completed", id));
            } else {
                task.mark_done();
                marked.push(id);

                // Spawn the next occurrence of a recurring task, or explain why
                // a recurrence-looking task didn't produce one.
                if task.recurrence().is_some() {
                    match task.next_occurrence(today) {
                        Some(next) => new_tasks.push(next),
                        None if task.due().is_none() => rec_warnings.push(format!(
                            "Task {} has a rec: tag but no due: date; no recurring task created",
                            id
                        )),
                        None => rec_warnings.push(format!(
                            "Task {} has an unrecognized rec: spec (use e.g. rec:1w or rec:+1m); no recurring task created",
                            id
                        )),
                    }
                } else if task.has_recurrence_marker() {
                    rec_warnings.push(format!(
                        "Task {} has a malformed rec: tag (write rec:+1m, not rec: +1m); no recurring task created",
                        id
                    ));
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

    for warning in &rec_warnings {
        printer.print_warning(warning);
    }

    Ok(())
}
