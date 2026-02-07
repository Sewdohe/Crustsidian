use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "obsidian-tasks")]
#[command(about = "Parse and filter tasks from Obsidian TaskNotes", long_about = None)]
struct Cli {
    /// Path to your Obsidian vault's TaskNotes folder
    #[arg(short, long)]
    path: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show all tasks
    All,
    /// Show today's tasks (due today)
    Today,
    /// Show overdue tasks
    Overdue,
    /// Show pending (not done) tasks
    Pending,
    /// Show tasks completed today
    CompletedToday,
    /// Show only count (for waybar)
    Count {
        #[arg(long)]
        today: bool,
        #[arg(long)]
        overdue: bool,
        #[arg(long)]
        completed_today: bool,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Task {
    #[serde(skip)]
    filename: String,
    status: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(rename = "dateCreated")]
    date_created: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    due: Option<NaiveDate>,
    #[serde(rename = "completedDate", default)]
    completed_date: Option<NaiveDate>,
    #[serde(rename = "taskSourceType", default)]
    task_source_type: Option<String>,
}

impl Task {
    fn is_done(&self) -> bool {
        self.status.to_lowercase() == "done"
    }

    fn is_due_today(&self) -> bool {
        if let Some(due) = self.due {
            due == Local::now().date_naive()
        } else {
            false
        }
    }

    fn is_overdue(&self) -> bool {
        if let Some(due) = self.due {
            !self.is_done() && due < Local::now().date_naive()
        } else {
            false
        }
    }

    fn is_completed_today(&self) -> bool {
        if let Some(completed) = self.completed_date {
            completed == Local::now().date_naive()
        } else {
            false
        }
    }
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() || lines[0] != "---" {
        return None;
    }

    // Find the closing ---
    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            return Some(lines[1..i].join("\n"));
        }
    }

    None
}

fn parse_task_file(path: &Path) -> Result<Task> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;

    let frontmatter = extract_frontmatter(&content)
        .context("No frontmatter found")?;

    let mut task: Task = serde_yaml::from_str(&frontmatter)
        .context("Failed to parse YAML frontmatter")?;

    task.filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(task)
}

fn collect_tasks(vault_path: &Path) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();

    for entry in WalkDir::new(vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
    {
        if let Ok(task) = parse_task_file(entry.path()) {
            tasks.push(task);
        }
    }

    Ok(tasks)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let tasks = collect_tasks(&cli.path)?;

    match cli.command {
        Commands::All => {
            let json = serde_json::to_string_pretty(&tasks)?;
            println!("{}", json);
        }
        Commands::Today => {
            let today_tasks: Vec<_> = tasks.iter()
                .filter(|t| t.is_due_today())
                .collect();
            let json = serde_json::to_string_pretty(&today_tasks)?;
            println!("{}", json);
        }
        Commands::Overdue => {
            let overdue_tasks: Vec<_> = tasks.iter()
                .filter(|t| t.is_overdue())
                .collect();
            let json = serde_json::to_string_pretty(&overdue_tasks)?;
            println!("{}", json);
        }
        Commands::Pending => {
            let pending_tasks: Vec<_> = tasks.iter()
                .filter(|t| !t.is_done())
                .collect();
            let json = serde_json::to_string_pretty(&pending_tasks)?;
            println!("{}", json);
        }
        Commands::CompletedToday => {
            let completed_today: Vec<_> = tasks.iter()
                .filter(|t| t.is_completed_today())
                .collect();
            let json = serde_json::to_string_pretty(&completed_today)?;
            println!("{}", json);
        }
        Commands::Count { today, overdue, completed_today } => {
            let count = if today {
                tasks.iter().filter(|t| t.is_due_today()).count()
            } else if overdue {
                tasks.iter().filter(|t| t.is_overdue()).count()
            } else if completed_today {
                tasks.iter().filter(|t| t.is_completed_today()).count()
            } else {
                tasks.iter().filter(|t| !t.is_done()).count()
            };
            println!("{}", count);
        }
    }

    Ok(())
}