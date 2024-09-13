use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

#[derive(Debug)]
enum EntryType {
    Todo {
        state: String,
        title: String,
        extra: String,
    },
    Til {
        title: String,
        extra: String,
    },
    Qts {
        question: String,
        extra: String,
    },
    Calendar {
        date: String,
        title: String,
        extra: String,
    },
    Note {
        tags: String,
        title: String,
        extra: String,
    },
}

#[derive(Debug)]
struct DvlgFile {
    entries: BTreeMap<String, Vec<EntryType>>, // Date -> List of Entries
}

impl DvlgFile {
    fn new() -> Self {
        DvlgFile {
            entries: BTreeMap::new(),
        }
    }

    fn add_entry(&mut self, date: &str, entry: EntryType) {
        self.entries
            .entry(date.to_string())
            .or_insert(Vec::new())
            .push(entry);
    }
}

fn parse_line(
    line: &str,
    current_date: &mut Option<String>,
    current_entry: &mut Option<EntryType>,
    dvlg: &mut DvlgFile,
) {
    fn handle_existing_entry(
        current_entry: &mut Option<EntryType>,
        current_date: &Option<String>,
        dvlg: &mut DvlgFile,
    ) {
        if let Some(entry) = current_entry.take() {
            if let Some(date) = current_date.as_ref() {
                dvlg.add_entry(date, entry);
            }
        }
    }

    if line.starts_with('@') {
        handle_existing_entry(current_entry, current_date, dvlg);
        *current_date = Some(line[1..].trim().to_string());
    } else if line.starts_with("- [") {
        handle_existing_entry(current_entry, current_date, dvlg);
        let state = &line[3..4];
        let title = line[5..].trim().to_string();
        *current_entry = Some(EntryType::Todo {
            state: state.to_string(),
            title,
            extra: String::new(),
        });
    } else if line.starts_with('!') {
        handle_existing_entry(current_entry, current_date, dvlg);
        let title = line[1..].trim().to_string();
        *current_entry = Some(EntryType::Til {
            title,
            extra: String::new(),
        });
    } else if line.starts_with('?') {
        handle_existing_entry(current_entry, current_date, dvlg);
        let question = line[1..].trim().to_string();
        *current_entry = Some(EntryType::Qts {
            question,
            extra: String::new(),
        });
    } else if line.starts_with('[') && line.contains(']') {
        handle_existing_entry(current_entry, current_date, dvlg);
        let end = line.find(']').unwrap();
        let date = &line[1..end];
        let title = line[end + 1..].trim().to_string();
        *current_entry = Some(EntryType::Calendar {
            date: date.to_string(),
            title,
            extra: String::new(),
        });
    } else if line.starts_with('>') || line.contains('>') {
        handle_existing_entry(current_entry, current_date, dvlg);
        let parts: Vec<&str> = line.splitn(2, '>').collect();
        let tags: String = parts[0].to_string();
        let title = parts[1].trim().to_string();
        *current_entry = Some(EntryType::Note {
            tags,
            title,
            extra: String::new(),
        });
    } else if let Some(entry) = current_entry.as_mut() {
        match entry {
            EntryType::Todo { extra, .. }
            | EntryType::Til { extra, .. }
            | EntryType::Qts { extra, .. }
            | EntryType::Calendar { extra, .. }
            | EntryType::Note { extra, .. } => {
                if !extra.is_empty() {
                    extra.push('\n');
                }
                extra.push_str(line);
            }
        }
    } else {
        eprintln!("Unexpected line format: {}", line);
    }
}

fn parse_file<P: AsRef<Path>>(filename: P) -> io::Result<DvlgFile> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);

    let mut dvlg = DvlgFile::new();
    let mut current_date: Option<String> = None;
    let mut current_entry: Option<EntryType> = None;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if !line.is_empty() {
            parse_line(line, &mut current_date, &mut current_entry, &mut dvlg);
        }
    }

    // If there's any leftover entry, add it
    if let Some(entry) = current_entry {
        if let Some(date) = current_date.as_ref() {
            dvlg.add_entry(date, entry);
        } else {
            eprintln!("Warning: Found an entry with no associated date.");
        }
    }

    Ok(dvlg)
}

fn display_entries(dvlg: &DvlgFile, entry_type: &str, tag: Option<&str>) {
    for (date, entries) in &dvlg.entries {
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|entry| match entry {
                EntryType::Todo { .. } if entry_type == "todo" => true,
                EntryType::Til { .. } if entry_type == "til" => true,
                EntryType::Qts { .. } if entry_type == "qts" => true,
                EntryType::Calendar { .. } if entry_type == "cal" => true,
                EntryType::Note { tags, .. } if entry_type == "note" => {
                    tag.map_or(true, |t| tags.contains(t))
                }
                _ => false,
            })
            .collect();

        if filtered.is_empty() {
            continue;
        }

        if entry_type != "cal" {
            println!("\n@{}", date);
        } else {
            filtered.sort_by(|a, b| {
                if let EntryType::Calendar { date: date_a, .. } = a {
                    if let EntryType::Calendar { date: date_b, .. } = b {
                        return date_a.cmp(date_b);
                    }
                }
                Ordering::Equal
            });
        }

        for entry in filtered {
            match entry {
                EntryType::Todo {
                    state,
                    title,
                    extra,
                } => {
                    println!("- [{}] {}", state, title);
                    if !extra.is_empty() {
                        println!("{}\n", extra);
                    }
                }
                EntryType::Til { title, extra } => {
                    println!("! {}", title);
                    if !extra.is_empty() {
                        println!("{}\n", extra);
                    }
                }
                EntryType::Qts { question, extra } => {
                    println!("? {}", question);
                    if !extra.is_empty() {
                        println!("{}\n", extra);
                    }
                }
                EntryType::Calendar { date, title, extra } => {
                    println!("[{}] {}", date, title);
                    if !extra.is_empty() {
                        println!("{}\n", extra);
                    }
                }
                EntryType::Note { tags, title, extra } => {
                    println!("{}> {}", tags, title);
                    if !extra.is_empty() {
                        println!("{}\n", extra);
                    }
                }
            }
        }
    }
    println!("");
}

fn print_usage(args: &Vec<String>) {
    eprintln!(
        "Usage: {} <filename> <todo|til|qts|cal|note> [tag]",
        args[0]
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage(&args);
        process::exit(1);
    }

    let filename: &str = &args[1];
    let entry_type: &str = &args[2];
    let tag: Option<&str> = if args.len() > 3 { Some(&args[3]) } else { None };

    if !vec!["todo", "til", "qts", "cal", "note"].contains(&entry_type) {
        print_usage(&args);
        process::exit(1);
    }

    match parse_file(filename) {
        Ok(dvlg) => display_entries(&dvlg, entry_type, tag),
        Err(e) => eprintln!("Error parsing file: {}", e),
    }
}
