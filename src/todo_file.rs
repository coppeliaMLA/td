use crate::task::Task;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct TodoFile {
    pub path: PathBuf,
    pub tasks: Vec<Task>,
}

impl TodoFile {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read todo file: {}", path.display()))?;

        let tasks: Vec<Task> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(idx, line)| Task::parse(idx + 1, line))
            .collect();

        Ok(TodoFile {
            path: path.to_path_buf(),
            tasks,
        })
    }

    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(TodoFile {
                path: path.to_path_buf(),
                tasks: Vec::new(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let content: String = self
            .tasks
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let content = if content.is_empty() {
            content
        } else {
            format!("{}\n", content)
        };

        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write todo file: {}", self.path.display()))?;

        Ok(())
    }

    pub fn add_task(&mut self, task: Task) {
        let mut task = task;
        task.id = self.tasks.len() + 1;
        self.tasks.push(task);
    }

    pub fn get_task(&self, id: usize) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn get_task_mut(&mut self, id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn remove_tasks(&mut self, ids: &[usize]) {
        self.tasks.retain(|t| !ids.contains(&t.id));
        self.reindex();
    }

    fn reindex(&mut self) {
        for (idx, task) in self.tasks.iter_mut().enumerate() {
            task.id = idx + 1;
        }
    }

    pub fn default_path() -> Result<PathBuf> {
        // First check current directory
        let current_dir = std::env::current_dir()?;
        let local_path = current_dir.join("todo.txt");
        if local_path.exists() {
            return Ok(local_path);
        }

        // Fall back to home directory
        if let Some(home) = dirs::home_dir() {
            return Ok(home.join("todo.txt"));
        }

        Ok(local_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_todo_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("todo.txt");
        fs::write(&file_path, "(A) Task one\n(B) Task two\n").unwrap();

        let todo = TodoFile::load(&file_path).unwrap();
        assert_eq!(todo.tasks.len(), 2);
        assert_eq!(todo.tasks[0].priority, Some('A'));
        assert_eq!(todo.tasks[1].priority, Some('B'));
    }

    #[test]
    fn test_save_todo_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("todo.txt");

        let mut todo = TodoFile {
            path: file_path.clone(),
            tasks: Vec::new(),
        };

        todo.add_task(Task::new("Test task"));
        todo.save().unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("Test task"));
    }

    #[test]
    fn test_remove_tasks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("todo.txt");
        fs::write(&file_path, "Task 1\nTask 2\nTask 3\n").unwrap();

        let mut todo = TodoFile::load(&file_path).unwrap();
        todo.remove_tasks(&[2]);

        assert_eq!(todo.tasks.len(), 2);
        assert_eq!(todo.tasks[0].id, 1);
        assert_eq!(todo.tasks[1].id, 2);
    }
}
