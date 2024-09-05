use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
enum TodoStatus {
    Todo,
    Completed,
    Dropped,
}

#[derive(Debug)]
enum LineType {
    Todo {
        status: TodoStatus,
        description: String,
    },
    Date(String),
    Reminder(String),
    TilNote(String),
    Note {
        id: String,
        note: String,
    },
}

fn parse_line(line: &str) -> Option<LineType> {
    match line.chars().next() {
        Some('-') if line.starts_with("- [ ]") => Some(LineType::Todo {
            status: TodoStatus::Todo,
            description: line[5..].trim().to_string(),
        }),
        Some('-') if line.starts_with("- [x]") => Some(LineType::Todo {
            status: TodoStatus::Completed,
            description: line[5..].trim().to_string(),
        }),
        Some('-') if line.starts_with("- [-]") => Some(LineType::Todo {
            status: TodoStatus::Dropped,
            description: line[5..].trim().to_string(),
        }),
        Some('#') => Some(LineType::Date(line[1..].trim().to_string())),
        Some('!') => Some(LineType::Reminder(line[1..].trim().to_string())),
        Some('?') => Some(LineType::TilNote(line[1..].trim().to_string())),
        Some('>') => {
            let parts: Vec<&str> = line[1..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                Some(LineType::Note {
                    id: parts[0].to_string(),
                    note: parts[1].trim().to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn main() -> io::Result<()> {
    // Change this path to point to your text file
    let path = Path::new("examples/ex1.dvlg");

    // Open the file
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    // Initialize a map to store Date -> List of LineType
    let mut date_map: HashMap<String, Vec<LineType>> = HashMap::new();
    let mut current_date = String::new();

    // Process each line in the file
    for line in reader.lines() {
        let line = line?;
        if let Some(parsed_line) = parse_line(&line) {
            match parsed_line {
                LineType::Date(date) => {
                    current_date = date.clone();
                    date_map.entry(date).or_insert(Vec::new());
                }
                _ => {
                    date_map
                        .entry(current_date.clone())
                        .or_insert(Vec::new())
                        .push(parsed_line);
                }
            }
        }
    }

    // Print the map for debugging purposes
    for (date, lines) in &date_map {
        println!("Date: {}", date);
        for line in lines {
            println!("{:?}", line);
        }
        println!();
    }

    Ok(())
}
