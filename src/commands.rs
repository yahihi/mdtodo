use crate::config::Config;
use crate::markdown::{Task, TodoFile};
use std::collections::HashMap;
use std::fs;

pub fn list(section_filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let todo = TodoFile::load(&todo_path)?;

    let numbered = todo.numbered_tasks();

    for section in &todo.sections {
        if let Some(ref filter) = section_filter {
            if !section.name.eq_ignore_ascii_case(filter) {
                continue;
            }
        }

        println!("## {}", section.name);

        if let Some(tasks) = numbered.get(&section.name) {
            for (num, task) in tasks {
                let status = if task.done { "[x]" } else { "[ ]" };
                let done_marker = match &task.done_date {
                    Some(date) => format!(" ✅ {}", date),
                    None => String::new(),
                };
                println!("  {}: {} {}{}", num, status, task.text, done_marker);
            }
        }

        println!();
    }

    Ok(())
}

pub fn add(section: String, text: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let section_idx = todo.get_or_create_section(&section);
    let task = Task {
        text: text.clone(),
        done: false,
        done_date: None,
    };

    todo.sections[section_idx].tasks.push(task);
    todo.save(&todo_path)?;

    println!("Added to {}: {}", section, text);

    Ok(())
}

pub fn done(task_ref: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, task_num) = parse_task_ref(&task_ref)?;
    let section_idx = todo
        .find_section(&section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let task_idx = task_num - 1;
    if task_idx >= todo.sections[section_idx].tasks.len() {
        return Err(format!("Task {} not found in section '{}'", task_num, section_name).into());
    }

    let task = &mut todo.sections[section_idx].tasks[task_idx];
    task.done = true;
    task.done_date = Some(config.today_str()?);

    let task_text = task.text.clone();
    todo.save(&todo_path)?;

    println!(
        "Marked as done: {} ({}:{})",
        task_text, section_name, task_num
    );

    Ok(())
}

pub fn undo(task_ref: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, task_num) = parse_task_ref(&task_ref)?;
    let section_idx = todo
        .find_section(&section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let task_idx = task_num - 1;
    if task_idx >= todo.sections[section_idx].tasks.len() {
        return Err(format!("Task {} not found in section '{}'", task_num, section_name).into());
    }

    let task = &mut todo.sections[section_idx].tasks[task_idx];
    task.done = false;
    task.done_date = None;

    let task_text = task.text.clone();
    todo.save(&todo_path)?;

    println!(
        "Marked as undone: {} ({}:{})",
        task_text, section_name, task_num
    );

    Ok(())
}

pub fn move_task(task_ref: String, dest: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, task_nums) = parse_task_ref_multi(&task_ref)?;
    let section_idx = todo
        .find_section(&section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let mut tasks_to_move = Vec::new();
    for task_num in task_nums.iter().rev() {
        let task_idx = task_num - 1;
        if task_idx >= todo.sections[section_idx].tasks.len() {
            return Err(
                format!("Task {} not found in section '{}'", task_num, section_name).into(),
            );
        }
        let task = todo.sections[section_idx].tasks.remove(task_idx);
        tasks_to_move.push((task_num, task));
    }

    tasks_to_move.reverse();

    let dest_idx = todo.get_or_create_section(&dest);

    for (task_num, task) in &tasks_to_move {
        println!(
            "Moved: {} ({}:{} -> {})",
            task.text, section_name, task_num, dest
        );
        todo.sections[dest_idx].tasks.push(task.clone());
    }

    todo.save(&todo_path)?;

    Ok(())
}

pub fn archive(task_ref: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let done_path = config.done_path()?;

    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, spec) = task_ref
        .split_once(':')
        .ok_or("Invalid task reference format. Use Section:number or Section:all")?;

    let section_idx = todo
        .find_section(section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let tasks_to_archive: Vec<(usize, Task)> = if spec == "all" {
        todo.sections[section_idx]
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.done)
            .map(|(i, t)| (i, t.clone()))
            .rev()
            .collect()
    } else {
        let task_nums = parse_numbers(spec)?;
        let mut tasks = Vec::new();
        for num in task_nums.iter().rev() {
            let idx = num - 1;
            if idx >= todo.sections[section_idx].tasks.len() {
                return Err(format!("Task {} not found in section '{}'", num, section_name).into());
            }
            let task = &todo.sections[section_idx].tasks[idx];
            if !task.done {
                return Err(format!(
                    "Task {} in section '{}' is not completed. Cannot archive incomplete tasks.",
                    num, section_name
                )
                .into());
            }
            tasks.push((idx, task.clone()));
        }
        tasks
    };

    if tasks_to_archive.is_empty() {
        println!("No completed tasks to archive in section '{}'", section_name);
        return Ok(());
    }

    let done_content = load_done_file(&done_path)?;
    let updated_done = append_to_done(done_content, section_name, &tasks_to_archive)?;
    save_done_file(&updated_done, &done_path)?;

    for (idx, task) in &tasks_to_archive {
        let done_date = task.done_date.as_deref().unwrap_or("unknown");
        println!(
            "Archived: {} ({}:{} -> done_list.md § {} / {})",
            task.text,
            section_name,
            idx + 1,
            done_date,
            section_name
        );
    }

    for (idx, _) in &tasks_to_archive {
        todo.sections[section_idx].tasks.remove(*idx);
    }

    todo.save(&todo_path)?;

    Ok(())
}

pub fn delete(task_ref: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, task_nums) = parse_task_ref_multi(&task_ref)?;
    let section_idx = todo
        .find_section(&section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let mut tasks_to_delete = Vec::new();
    for task_num in task_nums.iter().rev() {
        let task_idx = task_num - 1;
        if task_idx >= todo.sections[section_idx].tasks.len() {
            return Err(
                format!("Task {} not found in section '{}'", task_num, section_name).into(),
            );
        }
        let task = todo.sections[section_idx].tasks.remove(task_idx);
        tasks_to_delete.push((task_num, task));
    }

    tasks_to_delete.reverse();

    for (task_num, task) in &tasks_to_delete {
        println!(
            "Deleted: {} ({}:{})",
            task.text, section_name, task_num
        );
    }

    todo.save(&todo_path)?;

    Ok(())
}

pub fn edit(task_ref: String, new_text: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;
    let mut todo = TodoFile::load(&todo_path)?;

    let (section_name, task_num) = parse_task_ref(&task_ref)?;
    let section_idx = todo
        .find_section(&section_name)
        .ok_or(format!("Section '{}' not found", section_name))?;

    let task_idx = task_num - 1;
    if task_idx >= todo.sections[section_idx].tasks.len() {
        return Err(format!("Task {} not found in section '{}'", task_num, section_name).into());
    }

    let task = &mut todo.sections[section_idx].tasks[task_idx];
    let old_text = task.text.clone();
    task.text = new_text.clone();

    todo.save(&todo_path)?;

    println!(
        "Edited ({}:{}):\n  Before: {}\n  After:  {}",
        section_name, task_num, old_text, new_text
    );

    Ok(())
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let todo_path = config.todo_path()?;

    if todo_path.exists() {
        return Err(format!("TODO.md already exists at {}", todo_path.display()).into());
    }

    let template = r#"# TODO

## Today

## Next

## Backlogs

## Someday

## Waiting

## Inbox
"#;

    if let Some(parent) = todo_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&todo_path, template)?;

    println!("Initialized TODO.md at {}", todo_path.display());

    Ok(())
}

fn parse_task_ref(task_ref: &str) -> Result<(String, usize), Box<dyn std::error::Error>> {
    let (section, num_str) = task_ref
        .split_once(':')
        .ok_or("Invalid task reference format. Use Section:number")?;

    let num: usize = num_str
        .parse()
        .map_err(|_| "Invalid task number")?;

    Ok((section.to_string(), num))
}

fn parse_task_ref_multi(task_ref: &str) -> Result<(String, Vec<usize>), Box<dyn std::error::Error>> {
    let (section, nums_str) = task_ref
        .split_once(':')
        .ok_or("Invalid task reference format. Use Section:number or Section:number,number,...")?;

    let nums = parse_numbers(nums_str)?;

    Ok((section.to_string(), nums))
}

fn parse_numbers(nums_str: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut nums: Vec<usize> = nums_str
        .split(',')
        .map(|s| s.trim().parse::<usize>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid task number")?;

    nums.sort_unstable();
    nums.dedup();

    Ok(nums)
}

fn load_done_file(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    if path.exists() {
        Ok(fs::read_to_string(path)?)
    } else {
        Ok(String::from("# Done Log\n\n"))
    }
}

fn append_to_done(
    mut content: String,
    section_name: &str,
    tasks: &[(usize, Task)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut date_sections: HashMap<String, HashMap<String, Vec<Task>>> = HashMap::new();

    for (_, task) in tasks {
        let date = task.done_date.as_deref().unwrap_or("unknown").to_string();
        date_sections
            .entry(date)
            .or_insert_with(HashMap::new)
            .entry(section_name.to_string())
            .or_insert_with(Vec::new)
            .push(task.clone());
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    if lines.is_empty() || lines[0] != "# Done Log" {
        result.push("# Done Log".to_string());
        result.push(String::new());
    } else {
        result.push(lines[0].to_string());
        i = 1;
        if i < lines.len() && lines[i].is_empty() {
            result.push(String::new());
            i += 1;
        }
    }

    let mut existing_dates: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_date = None;
    let mut current_section = None;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("## ") {
            current_date = Some(line[3..].to_string());
            current_section = None;
        } else if line.starts_with("### ") {
            current_section = Some(line[4..].to_string());
        }

        if let (Some(ref date), Some(ref section)) = (&current_date, &current_section) {
            existing_dates
                .entry(date.clone())
                .or_insert_with(Vec::new)
                .push(section.clone());
        }

        i += 1;
    }

    content.clear();
    for line in &result {
        content.push_str(line);
        content.push('\n');
    }

    let mut dates: Vec<String> = date_sections.keys().cloned().collect();
    dates.sort();
    dates.reverse();

    for date in dates {
        content.push_str(&format!("## {}\n\n", date));

        if let Some(sections) = date_sections.get(&date) {
            for (section, tasks) in sections {
                content.push_str(&format!("### {}\n", section));
                for task in tasks {
                    content.push_str(&format!("{}\n", task.to_markdown()));
                }
                content.push('\n');
            }
        }
    }

    Ok(content)
}

fn save_done_file(content: &str, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_ref() {
        let (section, num) = parse_task_ref("Today:3").unwrap();
        assert_eq!(section, "Today");
        assert_eq!(num, 3);
    }

    #[test]
    fn test_parse_task_ref_multi() {
        let (section, nums) = parse_task_ref_multi("Today:3,1,5").unwrap();
        assert_eq!(section, "Today");
        assert_eq!(nums, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_numbers() {
        let nums = parse_numbers("3,1,5,1").unwrap();
        assert_eq!(nums, vec![1, 3, 5]);
    }
}
