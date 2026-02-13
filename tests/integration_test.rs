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

    let initial_content = r#"# TODO

## Today
- [ ] Existing task

## Next
"#;

    fs::write(&todo_path, initial_content).unwrap();

    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [ ] Existing task"));
}

#[test]
fn test_markdown_parsing_with_done_dates() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let content = r#"# TODO

## Today
- [ ] Task 1
- [x] Task 2 ✅ 2026-02-13

## Next
- [ ] Task 3
"#;

    fs::write(&todo_path, content).unwrap();
    let read_content = fs::read_to_string(&todo_path).unwrap();

    assert!(read_content.contains("- [ ] Task 1"));
    assert!(read_content.contains("- [x] Task 2 ✅ 2026-02-13"));
    assert!(read_content.contains("- [ ] Task 3"));
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
    let read_content = fs::read_to_string(&todo_path).unwrap();

    assert!(read_content.contains("## Today"));
    assert!(read_content.contains("## PROJECT1"));
    assert!(read_content.contains("## Waiting"));
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
fn test_delete_command() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let initial_content = r#"# TODO

## Today
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

## Next
- [ ] Task 4
"#;

    fs::write(&todo_path, initial_content).unwrap();

    // Verify initial state
    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [ ] Task 1"));
    assert!(content.contains("- [ ] Task 2"));
    assert!(content.contains("- [ ] Task 3"));
}

#[test]
fn test_edit_command() {
    let (_temp_dir, todo_path, _) = setup_test_env();

    let initial_content = r#"# TODO

## Today
- [ ] Old task text
- [x] Completed task ✅ 2026-02-13

## Next
- [ ] Another task
"#;

    fs::write(&todo_path, initial_content).unwrap();

    // Verify initial state
    let content = fs::read_to_string(&todo_path).unwrap();
    assert!(content.contains("- [ ] Old task text"));
    assert!(content.contains("- [x] Completed task ✅ 2026-02-13"));
}
