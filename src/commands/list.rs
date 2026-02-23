use crate::filter::{apply_filter, Filter, FilterTerm};
use crate::output::Printer;
use crate::todo_file::TodoFile;
use anyhow::Result;
use std::path::Path;

pub fn list(path: &Path, terms: &[String], show_all: bool, printer: &Printer) -> Result<()> {
    let todo = TodoFile::load(path)?;

    if todo.tasks.is_empty() {
        printer.print_warning("No tasks found");
        return Ok(());
    }

    let mut filter = Filter::new();
    if show_all {
        filter = filter.include_completed();
    }

    let mut projects = Vec::new();
    let mut contexts = Vec::new();
    let mut priorities = Vec::new();

    for term in terms {
        match Filter::parse_term(term) {
            FilterTerm::Project(p) => projects.push(p),
            FilterTerm::Context(c) => contexts.push(c),
            FilterTerm::Priority(p) => priorities.extend(p),
            FilterTerm::Text(t) => {
                filter = filter.with_text(t);
            }
            FilterTerm::Ambiguous(s) => {
                // Try to match against existing projects and contexts
                let is_project = todo.tasks.iter().any(|t| t.has_project(&s));
                let is_context = todo.tasks.iter().any(|t| t.has_context(&s));

                if is_context {
                    contexts.push(s);
                } else if is_project {
                    projects.push(s);
                } else {
                    // Treat as text search
                    filter = filter.with_text(s);
                }
            }
        }
    }

    if !projects.is_empty() {
        filter = filter.with_projects(projects);
    }
    if !contexts.is_empty() {
        filter = filter.with_contexts(contexts);
    }
    if !priorities.is_empty() {
        filter = filter.with_priorities(priorities);
    }

    let mut filtered = apply_filter(&todo.tasks, &filter);

    if filtered.is_empty() {
        printer.print_warning("No matching tasks found");
        return Ok(());
    }

    // Sort by priority (A first, then B, etc., None last) then by id
    filtered.sort_by(|a, b| {
        match (a.priority, b.priority) {
            (Some(pa), Some(pb)) => pa.cmp(&pb).then(a.id.cmp(&b.id)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        }
    });

    printer.print_tasks(&filtered);

    let completed = filtered.iter().filter(|t| t.is_completed).count();
    printer.print_summary(filtered.len(), completed);

    Ok(())
}
