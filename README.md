# Obsidian Tasks Parser

A Rust CLI tool to parse and filter tasks from your Obsidian TaskNotes vault.

## Building

```bash
cargo build --release
```

The binary will be at `target/release/obsidian-tasks`

## Usage

```bash
# Show all tasks as JSON
obsidian-tasks --path ~/path/to/vault/TaskNotes all

# Show today's tasks
obsidian-tasks --path ~/path/to/vault/TaskNotes today

# Show overdue tasks
obsidian-tasks --path ~/path/to/vault/TaskNotes overdue

# Show pending (not done) tasks
obsidian-tasks --path ~/path/to/vault/TaskNotes pending

# Get count of pending tasks (for waybar)
obsidian-tasks --path ~/path/to/vault/TaskNotes count

# Get count of today's tasks
obsidian-tasks --path ~/path/to/vault/TaskNotes count --today

# Get count of overdue tasks
obsidian-tasks --path ~/path/to/vault/TaskNotes count --overdue
```

## Waybar Integration

Add this to your waybar config:

```json
"custom/tasks": {
    "format": " {}",
    "exec": "obsidian-tasks --path ~/Obsidian/Vault/TaskNotes count --today",
    "interval": 60,
    "on-click": "alacritty -e obsidian-tasks --path ~/Obsidian/Vault/TaskNotes today | jq",
    "tooltip": true,
    "tooltip-format": "Today's tasks"
}
```

## Task Format

This tool expects Obsidian notes with YAML frontmatter like:

```yaml
---
status: done
priority: medium
dateCreated: 2026-01-30T08:18:47.998-05:00
tags:
  - task
projects:
  - "[[Fri Jan 30th 2026]]"
due: 2026-01-30
completedDate: 2026-02-01
taskSourceType: taskNotes
---
```
