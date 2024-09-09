use std::env;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Debug)]
enum DvlgEntry {
    DateHeader(String),
    Todo {
        state: String,
        description: String,
    },
    Til(String),
    Qts(String),
    Calendar {
        date: String,
        time: Option<String>,
        duration: Option<String>,
        event: String,
    },
    Note {
        tags: Vec<String>,
        content: String,
    },
}

fn parse_line(line: &str) -> Option<DvlgEntry> {
    let trimmed = line.trim();

    if trimmed.starts_with('@') && trimmed.len() == 11 {
        return Some(DvlgEntry::DateHeader(trimmed[1..].to_string()));
    }

    if trimmed.starts_with("- [") && trimmed.len() >= 6 {
        return Some(DvlgEntry::Todo {
            state: trimmed[3..4].to_string(),
            description: trimmed[6..].to_string(),
        });
    }

    if trimmed.starts_with("! ") {
        return Some(DvlgEntry::Til(trimmed[2..].to_string()));
    }

    // Match QTS
    if trimmed.starts_with("? ") {
        return Some(DvlgEntry::Qts(trimmed[2..].to_string()));
    }

    // Match Calendar Entries
    if trimmed.starts_with('[') && trimmed.contains(']') {
        if let Some(closing_bracket_pos) = trimmed.find(']') {
            let date_time_str = &trimmed[1..closing_bracket_pos];
            let event = trimmed[closing_bracket_pos + 2..].to_string(); // Skip the '] ' part
            let mut parts = date_time_str.split_whitespace();
            let date = parts.next().unwrap_or("").to_string();
            let time = parts.next().map(|s| s.to_string());
            let duration = time.as_ref().and_then(|t| {
                if t.contains('-') {
                    Some(t.split('-').nth(1).unwrap_or("").to_string())
                } else {
                    None
                }
            });
            let time = time.map(|t| t.split('-').next().unwrap_or("").to_string());
            return Some(DvlgEntry::Calendar {
                date,
                time,
                duration,
                event,
            });
        }
    }

    // Match Notes
    if let Some(pos) = trimmed.find("> ") {
        let tags_part = &trimmed[..pos].trim();
        let content = &trimmed[pos + 2..];
        let tags: Vec<String> = if tags_part.is_empty() {
            Vec::new()
        } else {
            tags_part.split('/').map(|s| s.to_string()).collect()
        };
        return Some(DvlgEntry::Note {
            tags,
            content: content.to_string(),
        });
    }

    None
}

fn parse_file(filename: &str) -> io::Result<Vec<DvlgEntry>> {
    let mut entries = Vec::new();

    // Read the file line by line
    if let Ok(file) = File::open(filename) {
        let lines = io::BufReader::new(file).lines();

        for line in lines {
            if let Ok(line_content) = line {
                if let Some(entry) = parse_line(&line_content) {
                    entries.push(entry);
                }
            }
        }
    }

    Ok(entries)
}

fn filter_entries(entries: Vec<DvlgEntry>, filter: &str) {
    let mut current_date: Option<String> = None;
    let mut has_output_for_current_date = false;

    for entry in entries {
        let line_to_print = match entry {
            DvlgEntry::DateHeader(date) => {
                current_date = Some(date);
                has_output_for_current_date = false;
                None
            }

            DvlgEntry::Todo { state, description } if filter == "todo" && state == " " => {
                Some(format!("  - [ ] {}", description))
            }

            // Filter for dropped TODOs
            DvlgEntry::Todo { state, description } if filter == "dropped" && state == "-" => {
                Some(format!("  - [-] {}", description))
            }
            DvlgEntry::Todo { state, description } if filter == "doing" && state == "/" => {
                Some(format!("  - [/] {}", description))
            }

            // Filter for TILs
            DvlgEntry::Til(til) if filter == "til" => Some(format!("  ! {}", til)),

            // Filter for QTSs
            DvlgEntry::Qts(qts) if filter == "qts" => Some(format!("  ? {}", qts)),

            _ => None,
        };

        if let Some(date) = &current_date {
            if !has_output_for_current_date && line_to_print.is_some() {
                println!("@{}", date);
                has_output_for_current_date = true;
            }
        }
        if let Some(to_print) = line_to_print {
            println!("{}", to_print);
        }
    }
}

fn print_usage() {
    println!("Usage: dvlg_parser <file> <filter>");
    println!("Filters: todo, dropped, doing, til, qts");
}

fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        return;
    }

    let filename = &args[1];
    let filter = &args[2];

    // Validate filter
    if !["todo", "dropped", "doing", "til", "qts"].contains(&filter.as_str()) {
        print_usage();
        return;
    }

    match parse_file(filename) {
        Ok(entries) => {
            filter_entries(entries, filter);
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}
