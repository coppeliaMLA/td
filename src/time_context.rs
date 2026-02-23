use chrono::{Datelike, Local, NaiveDate, Weekday};
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref TIME_CONTEXT_RE: Regex = Regex::new(
        r"@(today|tomorrow|thisweek|nextweek|thismonth|nextmonth|thisquarter|nextquarter)"
    ).unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeContext {
    Today,
    Tomorrow,
    ThisWeek,
    NextWeek,
    ThisMonth,
    NextMonth,
    ThisQuarter,
    NextQuarter,
}

impl TimeContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeContext::Today => "today",
            TimeContext::Tomorrow => "tomorrow",
            TimeContext::ThisWeek => "thisweek",
            TimeContext::NextWeek => "nextweek",
            TimeContext::ThisMonth => "thismonth",
            TimeContext::NextMonth => "nextmonth",
            TimeContext::ThisQuarter => "thisquarter",
            TimeContext::NextQuarter => "nextquarter",
        }
    }

    pub fn from_str(s: &str) -> Option<TimeContext> {
        match s.to_lowercase().as_str() {
            "today" => Some(TimeContext::Today),
            "tomorrow" => Some(TimeContext::Tomorrow),
            "thisweek" => Some(TimeContext::ThisWeek),
            "nextweek" => Some(TimeContext::NextWeek),
            "thismonth" => Some(TimeContext::ThisMonth),
            "nextmonth" => Some(TimeContext::NextMonth),
            "thisquarter" => Some(TimeContext::ThisQuarter),
            "nextquarter" => Some(TimeContext::NextQuarter),
            _ => None,
        }
    }

    pub fn all() -> &'static [TimeContext] {
        &[
            TimeContext::Today,
            TimeContext::Tomorrow,
            TimeContext::ThisWeek,
            TimeContext::NextWeek,
            TimeContext::ThisMonth,
            TimeContext::NextMonth,
            TimeContext::ThisQuarter,
            TimeContext::NextQuarter,
        ]
    }
}

/// Replace a time context in a task description
pub fn replace_time_context(description: &str, from: TimeContext, to: TimeContext) -> String {
    let from_tag = format!("@{}", from.as_str());
    let to_tag = format!("@{}", to.as_str());
    description.replace(&from_tag, &to_tag)
}

/// Remove all time contexts from a task description
pub fn remove_time_contexts(description: &str) -> String {
    let result = TIME_CONTEXT_RE.replace_all(description, "").to_string();
    // Collapse multiple spaces into one
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Set the time context for a task, replacing any existing time context
pub fn set_time_context(description: &str, context: TimeContext) -> String {
    let cleaned = remove_time_contexts(description);
    let trimmed = cleaned.trim();
    format!("{} @{}", trimmed, context.as_str())
}

/// Check if today is Friday
pub fn is_friday(date: NaiveDate) -> bool {
    date.weekday() == Weekday::Fri
}

/// Check if today is Monday
pub fn is_monday(date: NaiveDate) -> bool {
    date.weekday() == Weekday::Mon
}

/// Check if this is the last week of the month
pub fn is_last_week_of_month(date: NaiveDate) -> bool {
    let days_in_month = days_in_month(date.year(), date.month());
    let day = date.day();
    day > days_in_month - 7
}

/// Check if this is the first day of the month
pub fn is_first_of_month(date: NaiveDate) -> bool {
    date.day() == 1
}

/// Check if this is the last month of the quarter
pub fn is_last_month_of_quarter(date: NaiveDate) -> bool {
    matches!(date.month(), 3 | 6 | 9 | 12)
}

/// Check if this is the first month of the quarter
pub fn is_first_month_of_quarter(date: NaiveDate) -> bool {
    matches!(date.month(), 1 | 4 | 7 | 10)
}

/// Get the number of days in a month
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Determine which transitions should be applied based on the current date
pub fn get_transitions_for_date(date: NaiveDate) -> Vec<(TimeContext, TimeContext)> {
    let mut transitions = Vec::new();

    // Daily: @tomorrow → @today
    transitions.push((TimeContext::Tomorrow, TimeContext::Today));

    // Friday: @thisweek → @today
    if is_friday(date) {
        transitions.push((TimeContext::ThisWeek, TimeContext::Today));
    }

    // Monday: @nextweek → @thisweek
    if is_monday(date) {
        transitions.push((TimeContext::NextWeek, TimeContext::ThisWeek));
    }

    // Last week of month: @thismonth → @thisweek
    if is_last_week_of_month(date) {
        transitions.push((TimeContext::ThisMonth, TimeContext::ThisWeek));
    }

    // First of month: @nextmonth → @thismonth
    if is_first_of_month(date) {
        transitions.push((TimeContext::NextMonth, TimeContext::ThisMonth));
    }

    // Last month of quarter: @thisquarter → @thismonth
    if is_last_month_of_quarter(date) && is_first_of_month(date) {
        transitions.push((TimeContext::ThisQuarter, TimeContext::ThisMonth));
    }

    // First month of quarter: @nextquarter → @thisquarter
    if is_first_month_of_quarter(date) && is_first_of_month(date) {
        transitions.push((TimeContext::NextQuarter, TimeContext::ThisQuarter));
    }

    transitions
}

/// Apply time context transitions to a task description
pub fn apply_transitions(description: &str, transitions: &[(TimeContext, TimeContext)]) -> String {
    let mut result = description.to_string();
    for (from, to) in transitions {
        result = replace_time_context(&result, *from, *to);
    }
    result
}

/// Get current date
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_time_context() {
        let desc = "Task @tomorrow +project";
        let result = replace_time_context(desc, TimeContext::Tomorrow, TimeContext::Today);
        assert_eq!(result, "Task @today +project");
    }

    #[test]
    fn test_set_time_context() {
        let desc = "Task @tomorrow +project";
        let result = set_time_context(desc, TimeContext::NextWeek);
        assert_eq!(result, "Task +project @nextweek");
    }

    #[test]
    fn test_is_friday() {
        let friday = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        let saturday = NaiveDate::from_ymd_opt(2024, 1, 13).unwrap();
        assert!(is_friday(friday));
        assert!(!is_friday(saturday));
    }

    #[test]
    fn test_is_monday() {
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tuesday = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert!(is_monday(monday));
        assert!(!is_monday(tuesday));
    }

    #[test]
    fn test_is_last_week_of_month() {
        let last_week = NaiveDate::from_ymd_opt(2024, 1, 28).unwrap();
        let mid_month = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(is_last_week_of_month(last_week));
        assert!(!is_last_week_of_month(mid_month));
    }

    #[test]
    fn test_is_first_of_month() {
        let first = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let second = NaiveDate::from_ymd_opt(2024, 2, 2).unwrap();
        assert!(is_first_of_month(first));
        assert!(!is_first_of_month(second));
    }

    #[test]
    fn test_is_last_month_of_quarter() {
        assert!(is_last_month_of_quarter(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
        assert!(is_last_month_of_quarter(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()));
        assert!(!is_last_month_of_quarter(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()));
    }

    #[test]
    fn test_is_first_month_of_quarter() {
        assert!(is_first_month_of_quarter(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert!(is_first_month_of_quarter(NaiveDate::from_ymd_opt(2024, 4, 15).unwrap()));
        assert!(!is_first_month_of_quarter(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()));
    }

    #[test]
    fn test_transitions_on_friday() {
        let friday = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        let transitions = get_transitions_for_date(friday);
        assert!(transitions.contains(&(TimeContext::Tomorrow, TimeContext::Today)));
        assert!(transitions.contains(&(TimeContext::ThisWeek, TimeContext::Today)));
    }

    #[test]
    fn test_transitions_on_monday() {
        let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let transitions = get_transitions_for_date(monday);
        assert!(transitions.contains(&(TimeContext::Tomorrow, TimeContext::Today)));
        assert!(transitions.contains(&(TimeContext::NextWeek, TimeContext::ThisWeek)));
    }
}
