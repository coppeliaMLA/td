use chrono::{Duration, Months, NaiveDate};
use regex::Regex;
use lazy_static::lazy_static;
use std::fmt;

lazy_static! {
    static ref PRIORITY_RE: Regex = Regex::new(r"^\(([A-Z])\)\s+").unwrap();
    static ref DATE_RE: Regex = Regex::new(r"^(\d{4}-\d{2}-\d{2})\s+").unwrap();
    static ref COMPLETED_RE: Regex = Regex::new(r"^x\s+").unwrap();
    static ref PROJECT_RE: Regex = Regex::new(r"\+(\S+)").unwrap();
    static ref CONTEXT_RE: Regex = Regex::new(r"@(\S+)").unwrap();
    // key:value tags, e.g. `due:2026-07-01` or `rec:1w`
    static ref TAG_RE: Regex = Regex::new(r"([^\s:]+):(\S+)").unwrap();
    // the due: tag specifically, for in-place rewriting (\b avoids `overdue:`)
    static ref DUE_RE: Regex = Regex::new(r"\bdue:\S+").unwrap();
    // a recurrence spec: optional `+` (strict) then a count and a unit
    static ref REC_RE: Regex = Regex::new(r"^(\+?)(\d+)([dwmy])$").unwrap();
}

/// Extract all `key:value` tags from a piece of text, preserving order.
fn extract_tags(s: &str) -> Vec<(String, String)> {
    TAG_RE
        .captures_iter(s)
        .filter_map(|c| Some((c.get(1)?.as_str().to_string(), c.get(2)?.as_str().to_string())))
        .collect()
}

/// Advance `base` by `n` of `unit` (`d`/`w`/`m`/`y`). Returns None on overflow.
fn advance_date(base: NaiveDate, n: u32, unit: char) -> Option<NaiveDate> {
    match unit {
        'd' => base.checked_add_signed(Duration::days(n as i64)),
        'w' => base.checked_add_signed(Duration::weeks(n as i64)),
        'm' => base.checked_add_months(Months::new(n)),
        'y' => base.checked_add_months(Months::new(n * 12)),
        _ => None,
    }
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
    pub tags: Vec<(String, String)>,
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

        // Extract key:value tags
        let tags = extract_tags(&remaining);

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
            tags,
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
            tags: extract_tags(description),
            is_completed: false,
        }
    }

    pub fn mark_done(&mut self) {
        self.is_completed = true;
        self.completion_date = Some(chrono::Local::now().date_naive());
    }

    /// Look up the first value for a `key:value` tag.
    pub fn tag(&self, key: &str) -> Option<&str> {
        self.tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// The task's due date, from a `due:YYYY-MM-DD` tag, if present and valid.
    pub fn due(&self) -> Option<NaiveDate> {
        self.tag("due")
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// The raw recurrence spec, from a `rec:` tag (e.g. `1w`, `+3m`), if present.
    pub fn recurrence(&self) -> Option<&str> {
        self.tag("rec")
    }

    /// Build the next occurrence of a recurring task, to be created when this
    /// one is completed.
    ///
    /// A `rec:` spec is `<count><unit>` where unit is `d`/`w`/`m`/`y`. A leading
    /// `+` makes it *strict*: the next due date is offset from the old due date
    /// (so e.g. monthly rent never drifts). Without `+`, it is offset from the
    /// completion date (`today`). A `due:` tag is required to anchor recurrence;
    /// returns None if absent, or if the spec is malformed.
    pub fn next_occurrence(&self, today: NaiveDate) -> Option<Task> {
        let caps = REC_RE.captures(self.recurrence()?)?;
        let strict = !caps.get(1)?.as_str().is_empty();
        let n: u32 = caps.get(2)?.as_str().parse().ok()?;
        let unit = caps.get(3)?.as_str().chars().next()?;

        let due = self.due()?;
        let base = if strict { due } else { today };
        let next_due = advance_date(base, n, unit)?;

        // Rewrite the due: tag in the description to the new date.
        let new_desc = DUE_RE
            .replace(&self.description, format!("due:{}", next_due.format("%Y-%m-%d")))
            .to_string();

        let mut next = Task::new(&new_desc);
        next.priority = self.priority;
        next.creation_date = Some(today);
        Some(next)
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

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_parse_tags() {
        let task = Task::parse(1, "Pay rent due:2026-07-01 rec:1m");
        assert_eq!(task.due(), Some(ymd(2026, 7, 1)));
        assert_eq!(task.recurrence(), Some("1m"));
        assert_eq!(task.tag("rec"), Some("1m"));
        assert_eq!(task.description, "Pay rent due:2026-07-01 rec:1m");
    }

    #[test]
    fn test_next_occurrence_strict_uses_due_date() {
        // strict (+) recurs from the old due date, regardless of completion day
        let task = Task::parse(1, "(A) Pay rent due:2020-01-15 rec:+1m");
        let next = task.next_occurrence(ymd(2020, 1, 20)).unwrap();
        assert_eq!(next.due(), Some(ymd(2020, 2, 15)));
        assert_eq!(next.creation_date, Some(ymd(2020, 1, 20)));
        assert_eq!(next.priority, Some('A'));
        assert!(!next.is_completed);
    }

    #[test]
    fn test_next_occurrence_non_strict_uses_completion_date() {
        // non-strict recurs from the completion date (today)
        let task = Task::parse(1, "Water plants due:2020-01-15 rec:3d");
        let next = task.next_occurrence(ymd(2020, 1, 20)).unwrap();
        assert_eq!(next.due(), Some(ymd(2020, 1, 23)));
        assert_eq!(next.creation_date, Some(ymd(2020, 1, 20)));
    }

    #[test]
    fn test_next_occurrence_units() {
        let mk = |spec: &str| {
            Task::parse(1, &format!("T due:2020-01-15 rec:+{}", spec))
                .next_occurrence(ymd(2020, 6, 1))
                .unwrap()
                .due()
                .unwrap()
        };
        assert_eq!(mk("2d"), ymd(2020, 1, 17));
        assert_eq!(mk("2w"), ymd(2020, 1, 29));
        assert_eq!(mk("1y"), ymd(2021, 1, 15));
    }

    #[test]
    fn test_no_recurrence_without_rec_or_due() {
        // rec without due cannot be anchored
        assert!(Task::parse(1, "Task rec:1w").next_occurrence(ymd(2020, 1, 1)).is_none());
        // no rec at all
        assert!(Task::parse(1, "Task due:2020-01-01").next_occurrence(ymd(2020, 1, 1)).is_none());
        // malformed spec
        assert!(Task::parse(1, "Task due:2020-01-01 rec:soon").next_occurrence(ymd(2020, 1, 1)).is_none());
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
