mod config;
mod markdown;
mod commands;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "mdtodo")]
#[command(about = "A Markdown TODO CLI with section support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List tasks in a section (or all sections)
    List {
        /// Section name (optional)
        section: Option<String>,
    },
    /// Add a task to a section
    Add {
        /// Section name
        section: String,
        /// Task text
        text: String,
    },
    /// Mark a task as done
    Done {
        /// Task reference (Section:number)
        task: String,
    },
    /// Mark a task as undone
    Undo {
        /// Task reference (Section:number)
        task: String,
    },
    /// Move a task to another section
    Move {
        /// Task reference (Section:number or Section:number,number,...)
        task: String,
        /// Destination section
        dest: String,
    },
    /// Archive completed tasks to done_list.md
    Archive {
        /// Task reference (Section:number,number,... or Section:all)
        task: String,
    },
    /// Delete tasks
    Delete {
        /// Task reference (Section:number or Section:number,number,...)
        task: String,
    },
    /// Edit a task's text
    Edit {
        /// Task reference (Section:number)
        task: String,
        /// New text
        text: String,
    },
    /// Initialize TODO.md with default template
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List { section } => commands::list(section),
        Commands::Add { section, text } => commands::add(section, text),
        Commands::Done { task } => commands::done(task),
        Commands::Undo { task } => commands::undo(task),
        Commands::Move { task, dest } => commands::move_task(task, dest),
        Commands::Archive { task } => commands::archive(task),
        Commands::Delete { task } => commands::delete(task),
        Commands::Edit { task, text } => commands::edit(task, text),
        Commands::Init => commands::init(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
