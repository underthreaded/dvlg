use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

const OPTS: &[&str] = &[
    "todo", "done", "dropped", "doing", "til", "qts", "cal", "note",
];
const TODO_OPTS: &[&str] = &["todo", "done", "dropped", "doing"];

#[derive(Debug)]
enum EntryType {
    Todo {
        state: String,
        title: String,
        extra: String,
    },
    Til {
        tag: String,
        title: String,
        extra: String,
    },
    Idea {
        tag: String,
        title: String,
        extra: String,
    },
    Qts {
        tag: String,
        question: String,
        extra: String,
    },
    QtsAnswer {
        tag: String,
        answer: String,
        extra: String,
    },
    Calendar {
        date: String,
        title: String,
        extra: String,
    },
    Note {
        tag: String,
        title: String,
        extra: String,
    },
}

#[derive(Debug)]
enum LineType<'a> {
    Date(&'a str),
    NoteStart(EntryType),
    NoteLine(&'a str),
}

#[derive(Debug)]
struct DvlgFile {
    entries: BTreeMap<String, Vec<EntryType>>,
}

impl DvlgFile {
    fn new() -> Self {
        DvlgFile {
            entries: BTreeMap::new(),
        }
    }

    fn add_entry(&mut self, date: &Option<String>, entry: EntryType) {
        let date_str = match date {
            None => "1900-01-01".to_string(),
            Some(date_str) => date_str.to_string(),
        };
        self.entries
            .entry(date_str)
            .or_insert(Vec::new())
            .push(entry);
    }
}

fn parse_line(line: &str) -> LineType {
    let date_header = line.starts_with('@');
    let todo = line.starts_with("- [");

    let note = line.starts_with('/');
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let has_part = !parts.is_empty();

    let tag = match has_part {
        false => "".to_string(),
        true => parts[0].trim_matches(['!', '/', '?', '$']).to_string(),
    };

    let is_construct = |starts_with: &str, ends_with: &str| -> bool {
        has_part
            && (parts[0].starts_with(starts_with) || parts[0].starts_with(ends_with))
            && parts[0].ends_with(ends_with)
            && !parts[0]
                .strip_prefix(starts_with)
                .unwrap_or(parts[0])
                .strip_suffix(ends_with)
                .unwrap_or(parts[0].strip_prefix(starts_with).unwrap_or(parts[0]))
                .contains(['!', '?', '$'])
    };

    let til = is_construct("/", "!");
    let qts = is_construct("/", "?");
    let qtsa = is_construct("/", "?!");
    let idea = is_construct("/", "$");

    let title = if parts.len() == 2 {
        parts[1].trim().to_string()
    } else {
        "".to_string()
    };

    if date_header {
        return LineType::Date(line[1..].trim());
    } else if todo {
        let state = &line[3..4];
        let title = line[5..].trim().to_string();
        return LineType::NoteStart(EntryType::Todo {
            state: state.to_string(),
            title,
            extra: String::new(),
        });
    } else if line.starts_with('[') && line.contains(']') {
        let end = line.find(']').unwrap();
        let date = &line[1..end];
        let title = line[end + 1..].trim().to_string();
        return LineType::NoteStart(EntryType::Calendar {
            date: date.to_string(),
            title,
            extra: String::new(),
        });
    } else if til {
        return LineType::NoteStart(EntryType::Til {
            tag,
            title,
            extra: String::new(),
        });
    } else if idea {
        return LineType::NoteStart(EntryType::Idea {
            tag,
            title,
            extra: String::new(),
        });
    } else if qts {
        return LineType::NoteStart(EntryType::Qts {
            tag,
            question: title,
            extra: String::new(),
        });
    } else if qtsa {
        return LineType::NoteStart(EntryType::QtsAnswer {
            tag,
            answer: title,
            extra: String::new(),
        });
    } else if note {
        return LineType::NoteStart(EntryType::Note {
            tag,
            title: title,
            extra: String::new(),
        });
    } else {
        return LineType::NoteLine(line);
    }
}

fn parse_file<P: AsRef<Path>>(filename: P) -> io::Result<DvlgFile> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);

    let mut dvlg = DvlgFile::new();
    let mut current_date: Option<String> = None;
    let mut current_entry: Option<EntryType> = None;

    for line in reader.lines() {
        let tmp_line = line?;
        match parse_line(&tmp_line) {
            LineType::Date(date) => current_date = Some(date.to_string()),
            LineType::NoteStart(new_entry) => {
                match current_entry {
                    None => (),
                    Some(entry) => dvlg.add_entry(&current_date, entry),
                }
                current_entry = Some(new_entry);
            }
            LineType::NoteLine(note_line) => match current_entry {
                None => (),
                Some(EntryType::Todo { ref mut extra, .. })
                | Some(EntryType::Idea { ref mut extra, .. })
                | Some(EntryType::Til { ref mut extra, .. })
                | Some(EntryType::Qts { ref mut extra, .. })
                | Some(EntryType::QtsAnswer { ref mut extra, .. })
                | Some(EntryType::Calendar { ref mut extra, .. })
                | Some(EntryType::Note { ref mut extra, .. }) => {
                    if !extra.is_empty() {
                        extra.push('\n');
                    }
                    extra.push_str(note_line);
                }
            },
        }
    }

    // If there's any leftover entry, add it
    if let Some(entry) = current_entry {
        dvlg.add_entry(&current_date, entry);
    }

    Ok(dvlg)
}

fn print_entry(prefix: &str, extra: &str) {
    println!("{}", prefix);
    if !extra.is_empty() {
        println!("{}\n", extra);
    }
}

fn display_entries(dvlg: &DvlgFile, entry_type: &str, tags: Option<&str>) {
    for (date, entries) in &dvlg.entries {
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|entry| match entry {
                EntryType::Todo { state, .. } if TODO_OPTS.contains(&entry_type) => {
                    match (entry_type, state.as_str()) {
                        ("todo", " ") | ("done", "x") | ("doing", "/") | ("dropped", "-") => true,
                        _ => false,
                    }
                }
                EntryType::Til { .. } if entry_type == "til" => true,
                EntryType::Qts { .. } if entry_type == "qts" => true,
                EntryType::Calendar { .. } if entry_type == "cal" => true,
                EntryType::Note { tag, .. } if entry_type == "note" => {
                    tags.map_or(true, |t| tag.contains(t))
                }
                _ => false,
            })
            .collect();

        if filtered.is_empty() {
            continue;
        }

        if entry_type != "cal" {
            println!("@{}", date);
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

        let add_slash = |s: &String| {
            if s.is_empty() {
                s.to_string()
            } else {
                format!("/{}", s)
            }
        };
        for entry in filtered {
            match entry {
                EntryType::Todo {
                    state,
                    title,
                    extra,
                } => {
                    print_entry(&format!("- [{}] {}", state, title), extra);
                }
                EntryType::Idea { title, extra, tag } => {
                    print_entry(&format!("{}$ {}", add_slash(tag), title), extra);
                }
                EntryType::Til { title, extra, tag } => {
                    print_entry(&format!("{}! {}", add_slash(tag), title), extra);
                }
                EntryType::Qts {
                    question,
                    extra,
                    tag,
                } => {
                    print_entry(&format!("{}? {}", add_slash(tag), question), extra);
                }
                EntryType::QtsAnswer { answer, extra, tag } => {
                    print_entry(&format!("{}?! {}", add_slash(tag), answer), extra);
                }
                EntryType::Calendar { date, title, extra } => {
                    print_entry(&format!("[{}] {}", date, title), extra);
                }
                EntryType::Note { tag, title, extra } => {
                    print_entry(&format!("{}/ {}", add_slash(tag), title), extra);
                }
            }
        }
    }
}

fn print_usage(args: &Vec<String>) {
    eprintln!("Usage: {} <filename> <{}> [tag]", args[0], OPTS.join("|"));
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

    if !OPTS.contains(&entry_type) {
        print_usage(&args);
        process::exit(1);
    }

    match parse_file(filename) {
        Ok(dvlg) => display_entries(&dvlg, entry_type, tag),
        Err(e) => eprintln!("Error parsing file: {}", e),
    }
}
