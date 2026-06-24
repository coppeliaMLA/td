use crate::output::Printer;
use crate::task::Task;
use crate::time_context::{
    apply_transitions, get_transitions_for_date, set_time_context, time_context_for_due, today,
    TimeContext,
};
use crate::todo_file::TodoFile;
use anyhow::{bail, Result};
use std::path::Path;

/// Update time context for specific tasks
pub fn update(path: &Path, ids: &[usize], context: &str, printer: &Printer) -> Result<()> {
    let time_context = TimeContext::from_str(context)
        .ok_or_else(|| anyhow::anyhow!("Invalid time context: {}", context))?;

    let mut todo = TodoFile::load(path)?;

    let mut modified = Vec::new();
    let mut not_found = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            let new_desc = set_time_context(&task.description, time_context);
            *task = Task::parse(task.id, &format_task_with_desc(task, &new_desc));
            modified.push(id);
        } else {
            not_found.push(id);
        }
    }

    if !not_found.is_empty() {
        printer.print_error(&format!("Tasks not found: {:?}", not_found));
    }

    if modified.is_empty() {
        bail!("No tasks were modified");
    }

    todo.save()?;

    for &id in &modified {
        if let Some(task) = todo.get_task(id) {
            printer.print_success(&format!(
                "Set @{} for task {}: {}",
                time_context.as_str(),
                id,
                task.description
            ));
        }
    }

    Ok(())
}

/// Run automatic time context transitions based on current date
pub fn time_update(path: &Path, printer: &Printer) -> Result<()> {
    let current_date = today();
    let transitions = get_transitions_for_date(current_date);

    if transitions.is_empty() {
        printer.print_warning("No transitions to apply today");
        return Ok(());
    }

    let mut todo = TodoFile::load(path)?;
    let mut total_modified = 0;

    for task in &mut todo.tasks {
        let old_desc = task.to_string();

        // A dated, open task derives its time context straight from its due
        // date; everything else follows the relative day-based transitions.
        let new_desc = match (task.is_completed, task.due()) {
            (false, Some(due)) => match time_context_for_due(due, current_date) {
                Some(ctx) => set_time_context(&old_desc, ctx),
                None => old_desc.clone(), // too far out to bucket; leave as-is
            },
            _ => apply_transitions(&old_desc, &transitions),
        };

        if old_desc != new_desc {
            *task = Task::parse(task.id, &new_desc);
            total_modified += 1;
            printer.print_success(&format!("Updated task {}: {}", task.id, task.description));
        }
    }

    if total_modified > 0 {
        todo.save()?;
        printer.print_success(&format!(
            "Applied {} transitions, modified {} tasks",
            transitions.len(),
            total_modified
        ));
    } else {
        printer.print_warning("No tasks needed time context updates");
    }

    // Show which transitions were checked
    for (from, to) in &transitions {
        printer.print_success(&format!(
            "Checked: @{} → @{}",
            from.as_str(),
            to.as_str()
        ));
    }

    Ok(())
}

fn format_task_with_desc(task: &Task, new_desc: &str) -> String {
    let mut parts = Vec::new();

    if task.is_completed {
        parts.push("x".to_string());
        if let Some(date) = task.completion_date {
            parts.push(date.format("%Y-%m-%d").to_string());
        }
    }

    if let Some(p) = task.priority {
        if !task.is_completed {
            parts.push(format!("({})", p));
        }
    }

    if let Some(date) = task.creation_date {
        parts.push(date.format("%Y-%m-%d").to_string());
    }

    parts.push(new_desc.to_string());

    parts.join(" ")
}
