use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub todo_file: PathBuf,
    pub done_file: Option<PathBuf>,
    pub color: bool,
    pub verbose: bool,
    pub auto_archive: bool,
    pub date_on_add: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            todo_file: PathBuf::from("todo.txt"),
            done_file: None,
            color: true,
            verbose: false,
            auto_archive: false,
            date_on_add: true,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn with_file(mut self, path: PathBuf) -> Self {
        self.todo_file = path;
        self
    }

    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Get the default config file path
    pub fn default_path() -> Option<PathBuf> {
        // Try XDG config directory first
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("td").join("config.toml");
            if config_path.exists() {
                return Some(config_path);
            }
        }

        // Fall back to home directory
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".tdrc");
            if config_path.exists() {
                return Some(config_path);
            }
        }

        None
    }

    /// Load config from file, or return default if file doesn't exist
    pub fn load() -> Result<Self> {
        if let Some(path) = Self::default_path() {
            Self::load_from_file(&path)
        } else {
            Ok(Config::default())
        }
    }

    /// Load config from a specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save config to a file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get the effective todo file path
    pub fn get_todo_path(&self) -> PathBuf {
        if self.todo_file.is_absolute() {
            self.todo_file.clone()
        } else if let Some(home) = dirs::home_dir() {
            home.join(&self.todo_file)
        } else {
            self.todo_file.clone()
        }
    }

    /// Get the effective done file path
    pub fn get_done_path(&self) -> PathBuf {
        if let Some(ref done_file) = self.done_file {
            if done_file.is_absolute() {
                done_file.clone()
            } else if let Some(home) = dirs::home_dir() {
                home.join(done_file)
            } else {
                done_file.clone()
            }
        } else {
            // Default to done.txt in same directory as todo.txt
            let todo_path = self.get_todo_path();
            todo_path
                .parent()
                .map(|p| p.join("done.txt"))
                .unwrap_or_else(|| PathBuf::from("done.txt"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.todo_file, PathBuf::from("todo.txt"));
        assert!(config.color);
        assert!(config.date_on_add);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let config = Config {
            todo_file: PathBuf::from("/custom/todo.txt"),
            done_file: Some(PathBuf::from("/custom/done.txt")),
            color: false,
            verbose: true,
            auto_archive: true,
            date_on_add: false,
        };

        config.save_to_file(&config_path).unwrap();

        let loaded = Config::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.todo_file, PathBuf::from("/custom/todo.txt"));
        assert_eq!(loaded.done_file, Some(PathBuf::from("/custom/done.txt")));
        assert!(!loaded.color);
        assert!(loaded.verbose);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let config = Config::load_from_file(&PathBuf::from("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.todo_file, PathBuf::from("todo.txt"));
    }
}
