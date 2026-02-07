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
        let s = self.status.to_lowercase();
        s == "done" || s == "completed" || s == "x"
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

    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            return Some(lines[1..i].join("\n"));
        }
    }

    None
}

fn parse_task_file(path: &Path) -> Result<Task> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let frontmatter = extract_frontmatter(&content)
        .context("No frontmatter found")?;

    let mut task: Task = serde_yaml::from_str(&frontmatter)
        .with_context(|| format!("Failed to parse YAML in: {}", path.display()))?;

    task.filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(task)
}

/// Helper to scan a directory for .md files and add them to the tasks vector
fn scan_dir(path: &Path, tasks: &mut Vec<Task>) {
    if !path.exists() || !path.is_dir() {
        return;
    }

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()).map(|ext| ext.to_lowercase()) == Some("md".to_string()))
    {
        if let Ok(task) = parse_task_file(entry.path()) {
            // Check if task already exists in list to avoid duplicates if Archive is a subfolder
            if !tasks.iter().any(|t| t.filename == task.filename && t.date_created == task.date_created) {
                tasks.push(task);
            }
        }
    }
}

fn collect_tasks(vault_path: &Path) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();

    // 1. Scan the main TaskNotes directory (and its subfolders like Archive/)
    scan_dir(vault_path, &mut tasks);

    // 2. Explicitly check for an 'Archive' folder that might be a sibling 
    // (In case your CLI path points to 'Tasks' but archive is at 'Archive')
    if let Some(parent) = vault_path.parent() {
        let archive_sibling = parent.join("Archive");
        if archive_sibling.exists() && archive_sibling != vault_path {
            scan_dir(&archive_sibling, &mut tasks);
        }
    }

    Ok(tasks)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let tasks = collect_tasks(&cli.path)?;

    match cli.command {
        Commands::All => {
            println!("{}", serde_json::to_string_pretty(&tasks)?);
        }
        Commands::Today => {
            let filtered: Vec<_> = tasks.iter().filter(|t| t.is_due_today()).collect();
            println!("{}", serde_json::to_string_pretty(&filtered)?);
        }
        Commands::Overdue => {
            let filtered: Vec<_> = tasks.iter().filter(|t| t.is_overdue()).collect();
            println!("{}", serde_json::to_string_pretty(&filtered)?);
        }
        Commands::Pending => {
            let filtered: Vec<_> = tasks.iter().filter(|t| !t.is_done()).collect();
            println!("{}", serde_json::to_string_pretty(&filtered)?);
        }
        Commands::CompletedToday => {
            let filtered: Vec<_> = tasks.iter().filter(|t| t.is_completed_today()).collect();
            println!("{}", serde_json::to_string_pretty(&filtered)?);
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