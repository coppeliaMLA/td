use crate::output::Printer;
use crate::task::Task;
use crate::todo_file::TodoFile;
use anyhow::{bail, Result};
use std::path::Path;

pub fn append(path: &Path, ids: &[usize], text: &str, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    let mut modified = Vec::new();
    let mut not_found = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            task.description = format!("{} {}", task.description.trim(), text);
            // Re-parse to update projects and contexts
            *task = Task::parse(task.id, &task.to_string());
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
            printer.print_success(&format!("Updated {}: {}", id, task.description));
        }
    }

    Ok(())
}

pub fn prepend(path: &Path, ids: &[usize], text: &str, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    let mut modified = Vec::new();
    let mut not_found = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            task.description = format!("{} {}", text, task.description.trim());
            // Re-parse to update priority, projects and contexts
            *task = Task::parse(task.id, &task.to_string());
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
            printer.print_success(&format!("Updated {}: {}", id, task.description));
        }
    }

    Ok(())
}

pub fn replace(path: &Path, ids: &[usize], from: &str, to: &str, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    let mut modified = Vec::new();
    let mut not_found = Vec::new();
    let mut no_match = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            if task.description.contains(from) || task.to_string().contains(from) {
                let new_raw = task.to_string().replace(from, to);
                *task = Task::parse(task.id, &new_raw);
                modified.push(id);
            } else {
                no_match.push(id);
            }
        } else {
            not_found.push(id);
        }
    }

    if !not_found.is_empty() {
        printer.print_error(&format!("Tasks not found: {:?}", not_found));
    }

    if !no_match.is_empty() {
        printer.print_warning(&format!("No match found in tasks: {:?}", no_match));
    }

    if modified.is_empty() {
        bail!("No tasks were modified");
    }

    todo.save()?;

    for &id in &modified {
        if let Some(task) = todo.get_task(id) {
            printer.print_success(&format!("Updated {}: {}", id, task.description));
        }
    }

    Ok(())
}

pub fn priority(path: &Path, ids: &[usize], priority: Option<char>, printer: &Printer) -> Result<()> {
    let mut todo = TodoFile::load(path)?;

    let mut modified = Vec::new();
    let mut not_found = Vec::new();

    for &id in ids {
        if let Some(task) = todo.get_task_mut(id) {
            task.priority = priority;
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

    let action = match priority {
        Some(p) => format!("Set priority to ({})", p),
        None => "Removed priority".to_string(),
    };

    for id in &modified {
        printer.print_success(&format!("{} for task {}", action, id));
    }

    Ok(())
}
