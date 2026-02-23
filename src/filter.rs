use crate::task::Task;

#[derive(Debug, Default)]
pub struct Filter {
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub priorities: Vec<char>,
    pub text: Option<String>,
    pub show_completed: bool,
}

impl Filter {
    pub fn new() -> Self {
        Filter::default()
    }

    pub fn with_projects(mut self, projects: Vec<String>) -> Self {
        self.projects = projects;
        self
    }

    pub fn with_contexts(mut self, contexts: Vec<String>) -> Self {
        self.contexts = contexts;
        self
    }

    pub fn with_priorities(mut self, priorities: Vec<char>) -> Self {
        self.priorities = priorities;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn include_completed(mut self) -> Self {
        self.show_completed = true;
        self
    }

    pub fn matches(&self, task: &Task) -> bool {
        // Filter out completed tasks unless explicitly requested
        if task.is_completed && !self.show_completed {
            return false;
        }

        // Check projects (OR logic within projects)
        if !self.projects.is_empty() {
            let has_project = self.projects.iter().any(|p| task.has_project(p));
            if !has_project {
                return false;
            }
        }

        // Check contexts (OR logic within contexts)
        if !self.contexts.is_empty() {
            let has_context = self.contexts.iter().any(|c| task.has_context(c));
            if !has_context {
                return false;
            }
        }

        // Check priorities
        if !self.priorities.is_empty() {
            if let Some(p) = task.priority {
                if !self.priorities.contains(&p) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check text search
        if let Some(ref text) = self.text {
            if !task.description.to_lowercase().contains(&text.to_lowercase()) {
                return false;
            }
        }

        true
    }

    /// Parse a filter term like "+project", "@context", "A", "A-C", or text
    pub fn parse_term(term: &str) -> FilterTerm {
        if term.starts_with('+') {
            FilterTerm::Project(term[1..].to_string())
        } else if term.starts_with('@') {
            FilterTerm::Context(term[1..].to_string())
        } else if term.len() == 1 && term.chars().next().unwrap().is_ascii_uppercase() {
            FilterTerm::Priority(vec![term.chars().next().unwrap()])
        } else if term.len() == 3 && term.contains('-') {
            // Priority range like A-C
            let parts: Vec<&str> = term.split('-').collect();
            if parts.len() == 2
                && parts[0].len() == 1
                && parts[1].len() == 1
            {
                let start = parts[0].chars().next().unwrap();
                let end = parts[1].chars().next().unwrap();
                if start.is_ascii_uppercase() && end.is_ascii_uppercase() && start <= end {
                    let priorities: Vec<char> = (start..=end).collect();
                    return FilterTerm::Priority(priorities);
                }
            }
            FilterTerm::Text(term.to_string())
        } else {
            // Could be a project or context without prefix
            FilterTerm::Ambiguous(term.to_string())
        }
    }
}

#[derive(Debug)]
pub enum FilterTerm {
    Project(String),
    Context(String),
    Priority(Vec<char>),
    Text(String),
    Ambiguous(String),
}

pub fn apply_filter<'a>(tasks: &'a [Task], filter: &Filter) -> Vec<&'a Task> {
    tasks.iter().filter(|t| filter.matches(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tasks() -> Vec<Task> {
        vec![
            Task::parse(1, "(A) Task one +work @office"),
            Task::parse(2, "(B) Task two +home @phone"),
            Task::parse(3, "(A) Task three +work +urgent @home"),
            Task::parse(4, "Task four +personal"),
        ]
    }

    #[test]
    fn test_filter_by_project() {
        let tasks = sample_tasks();
        let filter = Filter::new().with_projects(vec!["work".to_string()]);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_context() {
        let tasks = sample_tasks();
        let filter = Filter::new().with_contexts(vec!["office".to_string()]);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_by_priority_single() {
        let tasks = sample_tasks();
        let filter = Filter::new().with_priorities(vec!['A']);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_priority_range() {
        let tasks = sample_tasks();
        let filter = Filter::new().with_priorities(vec!['A', 'B']);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_combined() {
        let tasks = sample_tasks();
        let filter = Filter::new()
            .with_projects(vec!["work".to_string()])
            .with_priorities(vec!['A']);
        let filtered = apply_filter(&tasks, &filter);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_parse_term_project() {
        matches!(Filter::parse_term("+work"), FilterTerm::Project(p) if p == "work");
    }

    #[test]
    fn test_parse_term_priority_range() {
        if let FilterTerm::Priority(p) = Filter::parse_term("A-C") {
            assert_eq!(p, vec!['A', 'B', 'C']);
        } else {
            panic!("Expected priority range");
        }
    }
}
