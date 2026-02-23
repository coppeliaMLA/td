use chrono::NaiveDate;
use regex::Regex;
use lazy_static::lazy_static;
use std::fmt;

lazy_static! {
    static ref PRIORITY_RE: Regex = Regex::new(r"^\(([A-Z])\)\s+").unwrap();
    static ref DATE_RE: Regex = Regex::new(r"^(\d{4}-\d{2}-\d{2})\s+").unwrap();
    static ref COMPLETED_RE: Regex = Regex::new(r"^x\s+").unwrap();
    static ref PROJECT_RE: Regex = Regex::new(r"\+(\S+)").unwrap();
    static ref CONTEXT_RE: Regex = Regex::new(r"@(\S+)").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: usize,
    pub raw: String,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub description: String,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub is_completed: bool,
}

impl Task {
    pub fn parse(id: usize, line: &str) -> Self {
        let mut remaining = line.trim().to_string();
        let raw = remaining.clone();

        // Check for completion
        let is_completed = COMPLETED_RE.is_match(&remaining);
        if is_completed {
            remaining = COMPLETED_RE.replace(&remaining, "").to_string();
        }

        // Parse completion date (if completed)
        let completion_date = if is_completed {
            if let Some(caps) = DATE_RE.captures(&remaining) {
                let date_str = caps.get(1).unwrap().as_str();
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
                remaining = DATE_RE.replace(&remaining, "").to_string();
                date
            } else {
                None
            }
        } else {
            None
        };

        // Parse priority
        let priority = if let Some(caps) = PRIORITY_RE.captures(&remaining) {
            let p = caps.get(1).unwrap().as_str().chars().next();
            remaining = PRIORITY_RE.replace(&remaining, "").to_string();
            p
        } else {
            None
        };

        // Parse creation date
        let creation_date = if let Some(caps) = DATE_RE.captures(&remaining) {
            let date_str = caps.get(1).unwrap().as_str();
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok();
            remaining = DATE_RE.replace(&remaining, "").to_string();
            date
        } else {
            None
        };

        // Extract projects
        let projects: Vec<String> = PROJECT_RE
            .captures_iter(&remaining)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // Extract contexts
        let contexts: Vec<String> = CONTEXT_RE
            .captures_iter(&remaining)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let description = remaining.trim().to_string();

        Task {
            id,
            raw,
            priority,
            creation_date,
            completion_date,
            description,
            projects,
            contexts,
            is_completed,
        }
    }

    pub fn new(description: &str) -> Self {
        Task {
            id: 0,
            raw: String::new(),
            priority: None,
            creation_date: None,
            completion_date: None,
            description: description.to_string(),
            projects: PROJECT_RE
                .captures_iter(description)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect(),
            contexts: CONTEXT_RE
                .captures_iter(description)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect(),
            is_completed: false,
        }
    }

    pub fn mark_done(&mut self) {
        self.is_completed = true;
        self.completion_date = Some(chrono::Local::now().date_naive());
    }

    pub fn has_project(&self, project: &str) -> bool {
        self.projects.iter().any(|p| p.eq_ignore_ascii_case(project))
    }

    pub fn has_context(&self, context: &str) -> bool {
        self.contexts.iter().any(|c| c.eq_ignore_ascii_case(context))
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.is_completed {
            parts.push("x".to_string());
            if let Some(date) = self.completion_date {
                parts.push(date.format("%Y-%m-%d").to_string());
            }
        }

        if let Some(p) = self.priority {
            if !self.is_completed {
                parts.push(format!("({})", p));
            }
        }

        if let Some(date) = self.creation_date {
            parts.push(date.format("%Y-%m-%d").to_string());
        }

        parts.push(self.description.clone());

        write!(f, "{}", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_task() {
        let task = Task::parse(1, "Buy groceries");
        assert_eq!(task.description, "Buy groceries");
        assert!(!task.is_completed);
        assert!(task.priority.is_none());
    }

    #[test]
    fn test_parse_task_with_priority() {
        let task = Task::parse(1, "(A) Important task");
        assert_eq!(task.priority, Some('A'));
        assert_eq!(task.description, "Important task");
    }

    #[test]
    fn test_parse_task_with_date() {
        let task = Task::parse(1, "2024-01-15 Task with date");
        assert_eq!(task.creation_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
    }

    #[test]
    fn test_parse_completed_task() {
        let task = Task::parse(1, "x 2024-01-15 2024-01-10 Completed task");
        assert!(task.is_completed);
        assert_eq!(task.completion_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert_eq!(task.creation_date, Some(NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()));
    }

    #[test]
    fn test_parse_task_with_projects_and_contexts() {
        let task = Task::parse(1, "Call mom +family +urgent @phone @home");
        assert!(task.has_project("family"));
        assert!(task.has_project("urgent"));
        assert!(task.has_context("phone"));
        assert!(task.has_context("home"));
    }

    #[test]
    fn test_task_to_string_roundtrip() {
        let original = "(A) 2024-01-15 Call mom +family @phone";
        let task = Task::parse(1, original);
        let output = task.to_string();
        assert!(output.contains("(A)"));
        assert!(output.contains("2024-01-15"));
        assert!(output.contains("Call mom"));
    }
}
