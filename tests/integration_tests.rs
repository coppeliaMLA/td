use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn td() -> Command {
    Command::cargo_bin("td").unwrap()
}

#[test]
fn test_help() {
    td().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A CLI tool for managing todo.txt files"));
}

#[test]
fn test_version() {
    td().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("td"));
}

#[test]
fn test_add_task() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");

    td().args(["-f", todo_path.to_str().unwrap(), "add", "Buy groceries +shopping @errands"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added:"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("Buy groceries"));
    assert!(content.contains("+shopping"));
    assert!(content.contains("@errands"));
}

#[test]
fn test_add_task_with_priority() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");

    td().args(["-f", todo_path.to_str().unwrap(), "add", "(A)", "Important task"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("(A)"));
    assert!(content.contains("Important task"));
}

#[test]
fn test_list_empty() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found"));
}

#[test]
fn test_list_tasks() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "(A) Task one\n(B) Task two\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"))
        .stdout(predicate::str::contains("Task two"));
}

#[test]
fn test_list_filter_by_priority() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "(A) Task one\n(B) Task two\n(C) Task three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list", "A"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"))
        .stdout(predicate::str::contains("Task two").not());
}

#[test]
fn test_list_filter_by_priority_range() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "(A) Task one\n(B) Task two\n(C) Task three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list", "A-B"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"))
        .stdout(predicate::str::contains("Task two"))
        .stdout(predicate::str::contains("Task three").not());
}

#[test]
fn test_list_filter_by_project() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one +work\nTask two +home\nTask three +work\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list", "+work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"))
        .stdout(predicate::str::contains("Task three"))
        .stdout(predicate::str::contains("Task two").not());
}

#[test]
fn test_list_filter_by_context() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one @phone\nTask two @email\nTask three @phone\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "list", "@phone"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"))
        .stdout(predicate::str::contains("Task three"))
        .stdout(predicate::str::contains("Task two").not());
}

#[test]
fn test_done_single_task() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "done", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Marked task 1 as done"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("x "));
}

#[test]
fn test_done_multiple_tasks() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\nTask three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "done", "1", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Marked task 1 as done"))
        .stdout(predicate::str::contains("Marked task 3 as done"));
}

#[test]
fn test_done_invalid_id() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "done", "99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_delete_task() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\nTask three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "delete", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("Task one"));
    assert!(!content.contains("Task two"));
    assert!(content.contains("Task three"));
}

#[test]
fn test_delete_multiple_tasks() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\nTask three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "rm", "1", "3"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(!content.contains("Task one"));
    assert!(content.contains("Task two"));
    assert!(!content.contains("Task three"));
}

#[test]
fn test_append_text() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "append", "1", "+project"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("Task one +project"));
}

#[test]
fn test_prepend_text() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "prepend", "1", "URGENT:"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("URGENT: Task one"));
}

#[test]
fn test_priority_set() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\nTask two\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "priority", "A", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set priority to (A)"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("(A)"));
}

#[test]
fn test_priority_remove() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "(A) Task one\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "priority", "-", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed priority"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(!content.contains("(A)"));
}

#[test]
fn test_replace_text() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one @today\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "replace", "-i", "1", "-s", "@today", "-r", "@tomorrow"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(!content.contains("@today"));
    assert!(content.contains("@tomorrow"));
}

#[test]
fn test_update_time_context() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one @today\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "update", "nextweek", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("@nextweek"));

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("@nextweek"));
    assert!(!content.contains("@today"));
}

#[test]
fn test_archive() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    let done_path = dir.path().join("done.txt");
    fs::write(&todo_path, "Task one\nx 2024-01-15 Completed task\nTask three\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "archive", "-d", done_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived 1 completed"));

    let todo_content = fs::read_to_string(&todo_path).unwrap();
    let done_content = fs::read_to_string(&done_path).unwrap();

    assert!(todo_content.contains("Task one"));
    assert!(!todo_content.contains("Completed task"));
    assert!(done_content.contains("Completed task"));
}

#[test]
fn test_time_update_command() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one @tomorrow\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "time-update"])
        .assert()
        .success();
}

#[test]
fn test_completions_bash() {
    td().args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_td()"));
}

#[test]
fn test_completions_zsh() {
    td().args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef td"));
}

#[test]
fn test_completions_fish() {
    td().args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_config_path() {
    td().args(["config", "path"])
        .assert()
        .success();
}

#[test]
fn test_config_show() {
    td().args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("todo_file"));
}

#[test]
fn test_no_color_flag() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "(A) Important task\n").unwrap();

    td().args(["--no-color", "-f", todo_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Important task"));
}

#[test]
fn test_ls_alias() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task one"));
}

#[test]
fn test_pri_alias() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    fs::write(&todo_path, "Task one\n").unwrap();

    td().args(["-f", todo_path.to_str().unwrap(), "pri", "B", "1"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("(B)"));
}

#[test]
fn test_workflow_add_list_done_archive() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");
    let done_path = dir.path().join("done.txt");

    // Add a task
    td().args(["-f", todo_path.to_str().unwrap(), "add", "(A)", "Complete project +work"])
        .assert()
        .success();

    // List it
    td().args(["-f", todo_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Complete project"));

    // Mark as done
    td().args(["-f", todo_path.to_str().unwrap(), "done", "1"])
        .assert()
        .success();

    // Archive it
    td().args(["-f", todo_path.to_str().unwrap(), "archive", "-d", done_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify it's in done.txt
    let done_content = fs::read_to_string(&done_path).unwrap();
    assert!(done_content.contains("Complete project"));

    // Verify todo.txt is empty (or has just a newline)
    let todo_content = fs::read_to_string(&todo_path).unwrap();
    assert!(!todo_content.contains("Complete project"));
}

#[test]
fn test_special_characters_in_task() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");

    td().args(["-f", todo_path.to_str().unwrap(), "add", "Task with 'quotes' and \"double quotes\""])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("quotes"));
}

#[test]
fn test_unicode_in_task() {
    let dir = tempdir().unwrap();
    let todo_path = dir.path().join("todo.txt");

    td().args(["-f", todo_path.to_str().unwrap(), "add", "Task with émojis 🎉 and üñíçödé"])
        .assert()
        .success();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("émojis"));
    assert!(content.contains("🎉"));
}
