use mdtodo::markdown::{Task, TodoFile};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_env() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let todo_path = temp_dir.path().join("TODO.md");
    let done_path = temp_dir.path().join("done_list.md");
    (temp_dir, todo_path, done_path)
}

#[test]
fn test_init_creates_todo_file() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let initial_content = r#"# TODO

## Today

## Next

## Backlogs

## Someday

## Waiting

## Inbox
"#;

    fs::write(&todo_path, initial_content).unwrap();

    assert!(todo_path.exists());
    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("## Today"));
    assert!(content.contains("## Next"));
}

#[test]
fn test_task_lifecycle() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    // Start with a section
    let initial_content = "# TODO\n\n## Today\n\n## Next\n";
    fs::write(&todo_path, initial_content).unwrap();

    // Add a task
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.get_or_create_section("Today");
    todo.sections[section_idx].tasks.push(Task {
        text: "Buy milk".to_string(),
        done: false,
        done_date: None,
    });
    todo.save(&todo_path).unwrap();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [ ] Buy milk"));

    // Mark as done
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();
    todo.sections[section_idx].tasks[0].done = true;
    todo.sections[section_idx].tasks[0].done_date = Some("2026-02-13".to_string());
    todo.save(&todo_path).unwrap();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [x] Buy milk ✅ 2026-02-13"));

    // Undo
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();
    todo.sections[section_idx].tasks[0].done = false;
    todo.sections[section_idx].tasks[0].done_date = None;
    todo.save(&todo_path).unwrap();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [ ] Buy milk"));
    assert!(!content.contains("✅"));
}

#[test]
fn test_markdown_parsing_with_done_dates() {
    let content = r#"# TODO

## Today
- [ ] Task 1
- [x] Task 2 ✅ 2026-02-13

## Next
- [ ] Task 3
"#;

    let todo = TodoFile::parse(content).unwrap();
    assert_eq!(todo.sections.len(), 2);

    let today = &todo.sections[0];
    assert_eq!(today.name, "Today");
    assert_eq!(today.tasks.len(), 2);
    assert!(!today.tasks[0].done);
    assert_eq!(today.tasks[0].text, "Task 1");
    assert!(today.tasks[1].done);
    assert_eq!(today.tasks[1].text, "Task 2");
    assert_eq!(today.tasks[1].done_date, Some("2026-02-13".to_string()));

    let next = &todo.sections[1];
    assert_eq!(next.name, "Next");
    assert_eq!(next.tasks.len(), 1);
    assert_eq!(next.tasks[0].text, "Task 3");
}

#[test]
fn test_section_preservation() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task 1
- [ ] Task 2

## PROJECT1
- [ ] Task 3

## Waiting
- [ ] Task 4
"#;

    fs::write(&todo_path, content).unwrap();
    let todo = TodoFile::load(&todo_path).unwrap();

    assert_eq!(todo.sections.len(), 3);
    assert_eq!(todo.sections[0].name, "Today");
    assert_eq!(todo.sections[1].name, "PROJECT1");
    assert_eq!(todo.sections[2].name, "Waiting");

    // Save and reload - sections should be preserved
    todo.save(&todo_path).unwrap();
    let reloaded = TodoFile::load(&todo_path).unwrap();
    assert_eq!(reloaded.sections.len(), 3);
    assert_eq!(reloaded.sections[0].tasks.len(), 2);
    assert_eq!(reloaded.sections[1].tasks.len(), 1);
    assert_eq!(reloaded.sections[2].tasks.len(), 1);
}

#[test]
fn test_done_list_structure() {
    let (_temp_dir, _todo_path, done_path) = setup_test_env();

    let done_content = r#"# Done Log

## 2026-02-13

### Today
- [x] Task 1 ✅ 2026-02-13

### PROJECT1
- [x] Task 2 ✅ 2026-02-13

## 2026-02-12

### Today
- [x] Old task ✅ 2026-02-12
"#;

    fs::write(&done_path, done_content).unwrap();
    let read_content = fs::read_to_string(&done_path).unwrap();

    assert!(read_content.contains("# Done Log"));
    assert!(read_content.contains("## 2026-02-13"));
    assert!(read_content.contains("### Today"));
    assert!(read_content.contains("### PROJECT1"));
    assert!(read_content.contains("## 2026-02-12"));
}

#[test]
fn test_delete_single_task() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

## Next
- [ ] Task 4
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();

    // Delete Task 2 (index 1)
    let section_idx = todo.find_section("Today").unwrap();
    assert_eq!(todo.sections[section_idx].tasks.len(), 3);

    let removed = todo.sections[section_idx].tasks.remove(1);
    assert_eq!(removed.text, "Task 2");

    todo.save(&todo_path).unwrap();

    // Verify result
    let reloaded = TodoFile::load(&todo_path).unwrap();
    let section_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[section_idx].tasks.len(), 2);
    assert_eq!(reloaded.sections[section_idx].tasks[0].text, "Task 1");
    assert_eq!(reloaded.sections[section_idx].tasks[1].text, "Task 3");

    // Other sections unaffected
    let next_idx = reloaded.find_section("Next").unwrap();
    assert_eq!(reloaded.sections[next_idx].tasks.len(), 1);
    assert_eq!(reloaded.sections[next_idx].tasks[0].text, "Task 4");
}

#[test]
fn test_delete_multiple_tasks() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
- [ ] Task 4
- [ ] Task 5
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();

    // Delete tasks 2, 4 (indices 1, 3) - remove from back to preserve indices
    todo.sections[section_idx].tasks.remove(3); // Task 4
    todo.sections[section_idx].tasks.remove(1); // Task 2

    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    let section_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[section_idx].tasks.len(), 3);
    assert_eq!(reloaded.sections[section_idx].tasks[0].text, "Task 1");
    assert_eq!(reloaded.sections[section_idx].tasks[1].text, "Task 3");
    assert_eq!(reloaded.sections[section_idx].tasks[2].text, "Task 5");
}

#[test]
fn test_delete_completed_task() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task 1
- [x] Completed task ✅ 2026-02-13
- [ ] Task 3
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();

    // Delete the completed task
    let removed = todo.sections[section_idx].tasks.remove(1);
    assert_eq!(removed.text, "Completed task");
    assert!(removed.done);
    assert_eq!(removed.done_date, Some("2026-02-13".to_string()));

    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    let section_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[section_idx].tasks.len(), 2);
    assert_eq!(reloaded.sections[section_idx].tasks[0].text, "Task 1");
    assert_eq!(reloaded.sections[section_idx].tasks[1].text, "Task 3");
}

#[test]
fn test_edit_task_text() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Old task text
- [ ] Another task

## Next
- [ ] Unrelated task
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();

    // Edit task 1
    todo.sections[section_idx].tasks[0].text = "Updated task text".to_string();
    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    let section_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[section_idx].tasks[0].text, "Updated task text");
    assert!(!reloaded.sections[section_idx].tasks[0].done);
    // Other tasks unchanged
    assert_eq!(reloaded.sections[section_idx].tasks[1].text, "Another task");

    let next_idx = reloaded.find_section("Next").unwrap();
    assert_eq!(reloaded.sections[next_idx].tasks[0].text, "Unrelated task");
}

#[test]
fn test_edit_preserves_done_state() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [x] Completed task ✅ 2026-02-13
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();
    let section_idx = todo.find_section("Today").unwrap();

    // Edit text of a completed task
    todo.sections[section_idx].tasks[0].text = "Edited completed task".to_string();
    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    let section_idx = reloaded.find_section("Today").unwrap();
    let task = &reloaded.sections[section_idx].tasks[0];
    assert_eq!(task.text, "Edited completed task");
    assert!(task.done);
    assert_eq!(task.done_date, Some("2026-02-13".to_string()));

    // Verify markdown format
    let saved = fs::read_to_string(&todo_path).unwrap();
    assert!(saved.contains("- [x] Edited completed task ✅ 2026-02-13"));
}

#[test]
fn test_move_task_between_sections() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task A
- [ ] Task B

## Next
- [ ] Task C
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();

    // Move Task B from Today to Next
    let from_idx = todo.find_section("Today").unwrap();
    let task = todo.sections[from_idx].tasks.remove(1);
    assert_eq!(task.text, "Task B");

    let to_idx = todo.find_section("Next").unwrap();
    todo.sections[to_idx].tasks.push(task);
    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    let today_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[today_idx].tasks.len(), 1);
    assert_eq!(reloaded.sections[today_idx].tasks[0].text, "Task A");

    let next_idx = reloaded.find_section("Next").unwrap();
    assert_eq!(reloaded.sections[next_idx].tasks.len(), 2);
    assert_eq!(reloaded.sections[next_idx].tasks[0].text, "Task C");
    assert_eq!(reloaded.sections[next_idx].tasks[1].text, "Task B");
}

#[test]
fn test_move_to_new_section() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task A
- [ ] Task B
"#;

    fs::write(&todo_path, content).unwrap();
    let mut todo = TodoFile::load(&todo_path).unwrap();

    // Move Task A to a new section
    let from_idx = todo.find_section("Today").unwrap();
    let task = todo.sections[from_idx].tasks.remove(0);

    let new_idx = todo.get_or_create_section("Urgent");
    todo.sections[new_idx].tasks.push(task);
    todo.save(&todo_path).unwrap();

    let reloaded = TodoFile::load(&todo_path).unwrap();
    assert_eq!(reloaded.sections.len(), 2);

    let today_idx = reloaded.find_section("Today").unwrap();
    assert_eq!(reloaded.sections[today_idx].tasks.len(), 1);
    assert_eq!(reloaded.sections[today_idx].tasks[0].text, "Task B");

    let urgent_idx = reloaded.find_section("Urgent").unwrap();
    assert_eq!(reloaded.sections[urgent_idx].tasks.len(), 1);
    assert_eq!(reloaded.sections[urgent_idx].tasks[0].text, "Task A");
}

#[test]
fn test_find_section_case_insensitive() {
    let content = r#"# TODO

## Today
- [ ] Task 1

## PROJECT1
- [ ] Task 2
"#;

    let todo = TodoFile::parse(content).unwrap();
    assert!(todo.find_section("today").is_some());
    assert!(todo.find_section("TODAY").is_some());
    assert!(todo.find_section("Today").is_some());
    assert!(todo.find_section("project1").is_some());
    assert!(todo.find_section("nonexistent").is_none());
}

#[test]
fn test_roundtrip_preserves_content() {
    let content = r#"# TODO

## Today
- [ ] Task 1
- [x] Task 2 ✅ 2026-02-13

## Next
- [ ] Task 3

## Backlogs
- [ ] Task 4
- [ ] Task 5
"#;

    let todo = TodoFile::parse(content).unwrap();
    let output = todo.to_string();

    assert!(output.contains("# TODO"));
    assert!(output.contains("## Today"));
    assert!(output.contains("- [ ] Task 1"));
    assert!(output.contains("- [x] Task 2 ✅ 2026-02-13"));
    assert!(output.contains("## Next"));
    assert!(output.contains("- [ ] Task 3"));
    assert!(output.contains("## Backlogs"));
    assert!(output.contains("- [ ] Task 4"));
    assert!(output.contains("- [ ] Task 5"));
}

#[test]
fn test_task_to_markdown() {
    let undone = Task {
        text: "Buy groceries".to_string(),
        done: false,
        done_date: None,
    };
    assert_eq!(undone.to_markdown(), "- [ ] Buy groceries");

    let done = Task {
        text: "Clean desk".to_string(),
        done: true,
        done_date: Some("2026-02-13".to_string()),
    };
    assert_eq!(done.to_markdown(), "- [x] Clean desk ✅ 2026-02-13");

    let done_no_date = Task {
        text: "Old task".to_string(),
        done: true,
        done_date: None,
    };
    assert_eq!(done_no_date.to_markdown(), "- [x] Old task");
}

#[test]
fn test_empty_file() {
    let todo = TodoFile::parse("").unwrap();
    assert_eq!(todo.sections.len(), 0);
    assert_eq!(todo.header_lines.len(), 0);
}

#[test]
fn test_numbered_tasks() {
    let content = r#"# TODO

## Today
- [ ] Task A
- [ ] Task B
- [ ] Task C
"#;

    let todo = TodoFile::parse(content).unwrap();
    let numbered = todo.numbered_tasks();

    let today_tasks = numbered.get("Today").unwrap();
    assert_eq!(today_tasks.len(), 3);
    assert_eq!(today_tasks[0].0, 1);
    assert_eq!(today_tasks[0].1.text, "Task A");
    assert_eq!(today_tasks[1].0, 2);
    assert_eq!(today_tasks[1].1.text, "Task B");
    assert_eq!(today_tasks[2].0, 3);
    assert_eq!(today_tasks[2].1.text, "Task C");
}
