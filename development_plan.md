# Development Plan: td - Todo.txt CLI Manager

This document outlines a phased approach to developing the `td` CLI tool for managing todo.txt files.

---

## Phase 1: Core Data Structures and Parsing

**Objective**: Establish the foundation by implementing the todo.txt parser and core data structures.

### Tasks

1. **Define the `Task` struct**
   - Fields: id, raw text, priority, creation date, completion date, description, projects, contexts, is_completed
   - Implement `Display` trait for serialization back to todo.txt format

2. **Implement the todo.txt parser**
   - Parse priority `(A)` - `(Z)`
   - Parse creation date `YYYY-MM-DD`
   - Parse completion marker `x` and completion date
   - Extract projects (`+project`) and contexts (`@context`)
   - Handle key:value metadata pairs

3. **Implement the `TodoFile` struct**
   - Load tasks from file
   - Save tasks to file
   - Track line numbers as task IDs
   - Handle file not found gracefully

### Tests for Phase 1

```rust
#[test] fn test_parse_simple_task() { }
#[test] fn test_parse_task_with_priority() { }
#[test] fn test_parse_task_with_date() { }
#[test] fn test_parse_completed_task() { }
#[test] fn test_parse_task_with_projects_and_contexts() { }
#[test] fn test_task_to_string_roundtrip() { }
#[test] fn test_load_todo_file() { }
#[test] fn test_save_todo_file() { }
```

---

## Phase 2: CLI Framework and Basic Commands

**Objective**: Set up the CLI structure and implement fundamental commands.

### Tasks

1. **Set up clap CLI structure**
   - Global options: `--file`, `--verbose`
   - Subcommands: `add`, `list`, `done`, `delete`

2. **Implement `add` command**
   - Parse task text from arguments
   - Auto-add creation date if not present
   - Append to todo.txt file

3. **Implement `list` command**
   - Display all tasks with line numbers
   - Color-code by priority
   - Show completion status

4. **Implement `done` command**
   - Accept multiple task IDs
   - Mark tasks as completed with completion date
   - Move to done.txt (optional) or mark in place

5. **Implement `delete` command**
   - Accept multiple task IDs
   - Remove tasks from file
   - Reindex remaining tasks

### Tests for Phase 2

```rust
#[test] fn test_add_command() { }
#[test] fn test_add_command_auto_date() { }
#[test] fn test_list_command_empty() { }
#[test] fn test_list_command_with_tasks() { }
#[test] fn test_done_single_task() { }
#[test] fn test_done_multiple_tasks() { }
#[test] fn test_delete_single_task() { }
#[test] fn test_delete_multiple_tasks() { }
#[test] fn test_delete_preserves_other_tasks() { }
```

---

## Phase 3: Filtering and Search

**Objective**: Implement powerful filtering capabilities for the `list` command.

### Tasks

1. **Implement project filtering**
   - Filter by `+project` or just `project`
   - Support multiple project filters (OR logic)

2. **Implement context filtering**
   - Filter by `@context` or just `context`
   - Support multiple context filters (OR logic)

3. **Implement priority filtering**
   - Single priority: `A`
   - Range: `A-C` (priorities A, B, C)

4. **Implement combined filtering**
   - Multiple filters work as AND between types, OR within types
   - Example: `td list +work +personal @urgent` = (work OR personal) AND urgent

5. **Implement text search**
   - Search within task descriptions
   - Case-insensitive matching

### Tests for Phase 3

```rust
#[test] fn test_filter_by_project() { }
#[test] fn test_filter_by_multiple_projects() { }
#[test] fn test_filter_by_context() { }
#[test] fn test_filter_by_priority_single() { }
#[test] fn test_filter_by_priority_range() { }
#[test] fn test_filter_combined() { }
#[test] fn test_filter_case_insensitive() { }
#[test] fn test_filter_no_prefix_project() { }
#[test] fn test_filter_no_prefix_context() { }
```

---

## Phase 4: Modification Commands

**Objective**: Implement commands for modifying existing tasks.

### Tasks

1. **Implement `append` command**
   - Add text to end of task(s)
   - Support multiple task IDs
   - Syntax: `td append 1 2 3 "+project"`

2. **Implement `prepend` command**
   - Add text after priority/date, before description
   - Support multiple task IDs
   - Syntax: `td prepend 1 2 3 "(A)"`

3. **Implement `replace` command**
   - Find and replace text within task(s)
   - Support multiple task IDs
   - Syntax: `td replace 1 2 "@old" "@new"`

4. **Implement `priority` command**
   - Set or change task priority
   - Remove priority with empty value
   - Syntax: `td priority 1 2 A` or `td priority 1 2 -`

5. **Implement `edit` command (optional)**
   - Open task in $EDITOR
   - Syntax: `td edit 5`

### Tests for Phase 4

```rust
#[test] fn test_append_single_task() { }
#[test] fn test_append_multiple_tasks() { }
#[test] fn test_prepend_single_task() { }
#[test] fn test_prepend_preserves_priority() { }
#[test] fn test_replace_text() { }
#[test] fn test_replace_multiple_tasks() { }
#[test] fn test_replace_no_match() { }
#[test] fn test_priority_set() { }
#[test] fn test_priority_change() { }
#[test] fn test_priority_remove() { }
```

---

## Phase 5: Time Context System

**Objective**: Implement the automatic time context management system.

### Tasks

1. **Define time contexts**
   - `@today`, `@tomorrow`
   - `@thisweek`, `@nextweek`
   - `@thismonth`, `@nextmonth`
   - `@thisquarter`, `@nextquarter`

2. **Implement `update` command for manual time context changes**
   - Syntax: `td update 1 2 3 today`
   - Remove existing time context and add new one
   - Validate time context names

3. **Implement time context transition logic**
   - Daily: `@tomorrow` → `@today`
   - Friday: `@thisweek` → `@today`
   - Monday: `@nextweek` → `@thisweek`
   - Last week of month: `@thismonth` → `@thisweek`
   - First of month: `@nextmonth` → `@thismonth`
   - Last month of quarter: `@thisquarter` → `@thismonth`
   - First of quarter: `@nextquarter` → `@thisquarter`

4. **Implement `time-update` command**
   - Check current date against last run
   - Apply appropriate transitions
   - Store last run timestamp

5. **Implement date condition checking**
   - Is today Friday?
   - Is today Monday?
   - Is this the last week of the month?
   - Is this the first of the month?
   - Is this the last month of the quarter?
   - Is this the first month of the quarter?

### Tests for Phase 5

```rust
#[test] fn test_update_time_context_today() { }
#[test] fn test_update_time_context_removes_old() { }
#[test] fn test_transition_tomorrow_to_today() { }
#[test] fn test_transition_thisweek_on_friday() { }
#[test] fn test_transition_nextweek_on_monday() { }
#[test] fn test_transition_thismonth_last_week() { }
#[test] fn test_transition_nextmonth_first_day() { }
#[test] fn test_transition_thisquarter_last_month() { }
#[test] fn test_transition_nextquarter_first_month() { }
#[test] fn test_is_last_week_of_month() { }
#[test] fn test_is_last_month_of_quarter() { }
#[test] fn test_multiple_transitions_applied() { }
```

---

## Phase 6: Output Formatting and UX

**Objective**: Improve the user experience with better output formatting.

### Tasks

1. **Implement colored output**
   - Priority A: Red
   - Priority B: Yellow
   - Priority C: Blue
   - Completed tasks: Gray/strikethrough
   - Projects: Cyan
   - Contexts: Green

2. **Implement task numbering display**
   - Right-aligned line numbers
   - Clear visual separation

3. **Implement summary statistics**
   - Total tasks
   - Completed vs pending
   - Tasks by priority

4. **Implement `--no-color` flag**
   - Disable colors for piping/scripts

5. **Implement `--quiet` flag**
   - Minimal output for scripting

### Tests for Phase 6

```rust
#[test] fn test_output_contains_line_numbers() { }
#[test] fn test_no_color_flag() { }
#[test] fn test_quiet_mode() { }
#[test] fn test_summary_statistics() { }
```

---

## Phase 7: Configuration and Polish

**Objective**: Add configuration support and final polish.

### Tasks

1. **Implement configuration file**
   - Location: `~/.config/td/config.toml` or `~/.tdrc`
   - Options: default todo.txt path, colors, date format

2. **Implement archive functionality**
   - Move completed tasks to `done.txt`
   - Syntax: `td archive`

3. **Implement undo functionality (optional)**
   - Keep backup before modifications
   - Syntax: `td undo`

4. **Add shell completions**
   - Bash, Zsh, Fish completions
   - Syntax: `td completions bash`

5. **Error handling improvements**
   - Friendly error messages
   - Suggestions for common mistakes

6. **Documentation**
   - Man page
   - `--help` text refinement

### Tests for Phase 7

```rust
#[test] fn test_config_file_loading() { }
#[test] fn test_config_default_values() { }
#[test] fn test_archive_moves_completed() { }
#[test] fn test_archive_preserves_pending() { }
#[test] fn test_error_invalid_task_id() { }
#[test] fn test_error_file_not_found() { }
```

---

## Phase 8: Integration Testing and Release

**Objective**: Comprehensive testing and release preparation.

### Tasks

1. **Integration tests**
   - End-to-end CLI testing with `assert_cmd`
   - Test complete workflows
   - Test edge cases (empty file, very large file)

2. **Performance testing**
   - Test with 10,000+ tasks
   - Ensure reasonable performance

3. **Documentation review**
   - README completeness
   - Help text accuracy
   - Examples verification

4. **Release preparation**
   - Version bump
   - Changelog
   - Binary builds for major platforms

### Tests for Phase 8

```rust
#[test] fn test_workflow_add_list_done() { }
#[test] fn test_workflow_time_contexts() { }
#[test] fn test_large_file_performance() { }
#[test] fn test_concurrent_modifications() { }
#[test] fn test_special_characters_in_tasks() { }
#[test] fn test_unicode_support() { }
```

---

## Project Structure

```
td/
├── Cargo.toml
├── README.md
├── development_plan.md
├── src/
│   ├── main.rs           # Entry point, CLI setup
│   ├── lib.rs            # Library exports
│   ├── task.rs           # Task struct and parsing
│   ├── todo_file.rs      # File I/O operations
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── add.rs
│   │   ├── list.rs
│   │   ├── done.rs
│   │   ├── delete.rs
│   │   ├── modify.rs     # append, prepend, replace
│   │   ├── update.rs     # time context updates
│   │   └── archive.rs
│   ├── filter.rs         # Filtering logic
│   ├── time_context.rs   # Time context transitions
│   ├── config.rs         # Configuration handling
│   └── output.rs         # Colored output formatting
└── tests/
    ├── integration_tests.rs
    └── fixtures/
        └── sample_todo.txt
```

---

## Milestones Summary

| Phase | Milestone | Key Deliverable |
|-------|-----------|-----------------|
| 1 | Core Foundation | Working parser and Task struct |
| 2 | Basic CLI | add, list, done, delete commands |
| 3 | Filtering | Project, context, priority filtering |
| 4 | Modifications | append, prepend, replace, priority commands |
| 5 | Time Contexts | Automatic time context transitions |
| 6 | UX Polish | Colored output, statistics |
| 7 | Configuration | Config file, archive, completions |
| 8 | Release | Integration tests, documentation, binaries |

---

## Getting Started

Begin development with Phase 1:

```bash
# Run tests as you develop
cargo test

# Test the CLI manually
cargo run -- add "Test task"
cargo run -- list
```

Each phase builds upon the previous one. Complete all tests for a phase before moving to the next.
