use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_todo_path")]
    pub todo_path: String,
    #[serde(default = "default_done_path")]
    pub done_path: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_todo_path() -> String {
    "./TODO.md".to_string()
}

fn default_done_path() -> String {
    "./done_list.md".to_string()
}

fn default_timezone() -> String {
    "Local".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            todo_path: default_todo_path(),
            done_path: default_done_path(),
            timezone: default_timezone(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("mdtodo");

        Ok(config_dir.join("config.toml"))
    }

    pub fn expand_path(path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if path.starts_with("~/") {
            let home = dirs::home_dir()
                .ok_or("Could not find home directory")?;
            Ok(home.join(&path[2..]))
        } else {
            Ok(PathBuf::from(path))
        }
    }

    pub fn todo_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        Self::expand_path(&self.todo_path)
    }

    pub fn done_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        Self::expand_path(&self.done_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.todo_path, "./TODO.md");
        assert_eq!(config.done_path, "./done_list.md");
        assert_eq!(config.timezone, "Local");
    }
}
