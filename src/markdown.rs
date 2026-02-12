use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub text: String,
    pub done: bool,
    pub done_date: Option<String>,
}

impl Task {
    pub fn to_markdown(&self) -> String {
        let checkbox = if self.done { "[x]" } else { "[ ]" };
        let done_marker = match &self.done_date {
            Some(date) => format!(" ✅ {}", date),
            None => String::new(),
        };
        format!("- {} {}{}", checkbox, self.text, done_marker)
    }

    pub fn from_line(line: &str) -> Option<Self> {
        let task_regex = Regex::new(r"^- \[([ x])\] (.+)$").unwrap();

        if let Some(caps) = task_regex.captures(line.trim()) {
            let done = &caps[1] == "x";
            let text_part = caps[2].to_string();

            let done_date_regex = Regex::new(r"^(.*?) ✅ (\d{4}-\d{2}-\d{2})$").unwrap();
            let (text, done_date) = if let Some(date_caps) = done_date_regex.captures(&text_part) {
                (date_caps[1].to_string(), Some(date_caps[2].to_string()))
            } else {
                (text_part, None)
            };

            Some(Task {
                text,
                done,
                done_date,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub tasks: Vec<Task>,
    pub other_lines: Vec<(usize, String)>,
}

impl Section {
    pub fn new(name: String) -> Self {
        Section {
            name,
            tasks: Vec::new(),
            other_lines: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TodoFile {
    pub sections: Vec<Section>,
    pub header_lines: Vec<String>,
}

impl TodoFile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(TodoFile {
                sections: Vec::new(),
                header_lines: Vec::new(),
            });
        }

        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut sections = Vec::new();
        let mut header_lines = Vec::new();
        let mut current_section: Option<Section> = None;
        let mut line_idx_in_section = 0;

        let section_regex = Regex::new(r"^##\s+(.+)$").unwrap();

        for line in lines {
            if let Some(caps) = section_regex.captures(line) {
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }
                current_section = Some(Section::new(caps[1].to_string()));
                line_idx_in_section = 0;
            } else if let Some(ref mut section) = current_section {
                if let Some(task) = Task::from_line(line) {
                    section.tasks.push(task);
                } else if !line.trim().is_empty() {
                    section.other_lines.push((line_idx_in_section, line.to_string()));
                }
                line_idx_in_section += 1;
            } else {
                header_lines.push(line.to_string());
            }
        }

        if let Some(section) = current_section {
            sections.push(section);
        }

        Ok(TodoFile {
            sections,
            header_lines,
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = self.to_string();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for line in &self.header_lines {
            result.push_str(line);
            result.push('\n');
        }

        for section in &self.sections {
            result.push_str(&format!("## {}\n", section.name));

            let mut all_lines: Vec<(usize, String)> = Vec::new();

            for (idx, task) in section.tasks.iter().enumerate() {
                all_lines.push((idx, task.to_markdown()));
            }

            for (idx, line) in &section.other_lines {
                all_lines.push((*idx, line.clone()));
            }

            all_lines.sort_by_key(|(idx, _)| *idx);

            for (_, line) in all_lines {
                result.push_str(&line);
                result.push('\n');
            }

            result.push('\n');
        }

        result
    }

    pub fn find_section(&self, name: &str) -> Option<usize> {
        self.sections.iter().position(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn get_or_create_section(&mut self, name: &str) -> usize {
        if let Some(idx) = self.find_section(name) {
            idx
        } else {
            self.sections.push(Section::new(name.to_string()));
            self.sections.len() - 1
        }
    }

    pub fn numbered_tasks(&self) -> HashMap<String, Vec<(usize, &Task)>> {
        let mut result = HashMap::new();

        for section in &self.sections {
            let tasks: Vec<(usize, &Task)> = section
                .tasks
                .iter()
                .enumerate()
                .map(|(idx, task)| (idx + 1, task))
                .collect();
            result.insert(section.name.clone(), tasks);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_parsing() {
        let task = Task::from_line("- [ ] Test task").unwrap();
        assert_eq!(task.text, "Test task");
        assert!(!task.done);
        assert_eq!(task.done_date, None);

        let done_task = Task::from_line("- [x] Done task ✅ 2026-02-13").unwrap();
        assert_eq!(done_task.text, "Done task");
        assert!(done_task.done);
        assert_eq!(done_task.done_date, Some("2026-02-13".to_string()));
    }

    #[test]
    fn test_todo_file_parsing() {
        let content = r#"# TODO

## Today
- [ ] Task 1
- [x] Task 2 ✅ 2026-02-13

## Next
- [ ] Task 3
"#;

        let todo = TodoFile::parse(content).unwrap();
        assert_eq!(todo.sections.len(), 2);
        assert_eq!(todo.sections[0].name, "Today");
        assert_eq!(todo.sections[0].tasks.len(), 2);
        assert_eq!(todo.sections[1].name, "Next");
        assert_eq!(todo.sections[1].tasks.len(), 1);
    }
}
