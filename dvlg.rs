use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

const OPTS: &[&str] = &[
    "todo", "done", "dropped", "doing", "til", "qts", "cal", "note", "idea", "fmt",
];
const TODO_OPTS: &[&str] = &["todo", "done", "dropped", "doing"];

#[derive(Debug)]
enum Note {
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
    General {
        tag: String,
        title: String,
        extra: String,
    },
}

#[derive(Debug)]
enum LineType<'a> {
    Date(&'a str),
    NoteStart(Note),
    NoteLine(&'a str),
}

#[derive(Debug)]
struct Dvlg {
    entries: BTreeMap<String, Vec<Note>>,
}

impl Dvlg {
    fn new() -> Self {
        Dvlg {
            entries: BTreeMap::new(),
        }
    }

    fn add_entry(&mut self, date: &Option<String>, entry: Note) {
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

    let title = if parts.len() == 2 {
        parts[1].trim().to_string()
    } else {
        "".to_string()
    };

    if date_header {
        return LineType::Date(line[1..].trim());
    } else if line.starts_with("- [") {
        let end = line.find(']').unwrap();
        let state = &line[3..end];
        let entry = line[end + 1..].trim().to_string();
        return LineType::NoteStart(Note::Todo {
            state: state.to_string(),
            title: entry,
            extra: String::new(),
        });
    } else if line.starts_with('[') && line.contains(']') {
        let end = line.find(']').unwrap();
        let date = &line[1..end];
        let event = line[end + 1..].trim().to_string();
        return LineType::NoteStart(Note::Calendar {
            date: date.to_string(),
            title: event,
            extra: String::new(),
        });
    } else if is_construct("/", "!") {
        return LineType::NoteStart(Note::Til {
            tag,
            title,
            extra: String::new(),
        });
    } else if is_construct("/", "$") {
        return LineType::NoteStart(Note::Idea {
            tag,
            title,
            extra: String::new(),
        });
    } else if is_construct("/", "?") {
        return LineType::NoteStart(Note::Qts {
            tag,
            question: title,
            extra: String::new(),
        });
    } else if is_construct("/", "?!") {
        return LineType::NoteStart(Note::QtsAnswer {
            tag,
            answer: title,
            extra: String::new(),
        });
    } else if note {
        return LineType::NoteStart(Note::General {
            tag,
            title: title,
            extra: String::new(),
        });
    } else {
        return LineType::NoteLine(line);
    }
}

fn parse_file<P: AsRef<Path>>(filename: P) -> io::Result<Dvlg> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);

    let mut dvlg = Dvlg::new();
    let mut current_date: Option<String> = None;
    let mut current_entry: Option<Note> = None;

    for line in reader.lines() {
        let tmp_line = line?;
        match parse_line(&tmp_line) {
            LineType::Date(date) => {
                match current_entry {
                    None => (),
                    Some(entry) => {
                        dvlg.add_entry(&current_date, entry);
                    }
                }
                current_entry = None;
                current_date = Some(date.to_string());
            }
            LineType::NoteStart(new_entry) => {
                match current_entry {
                    None => (),
                    Some(entry) => {
                        dvlg.add_entry(&current_date, entry);
                    }
                }
                current_entry = Some(new_entry);
            }
            LineType::NoteLine(note_line) => match current_entry {
                None => (),
                Some(Note::Todo { ref mut extra, .. })
                | Some(Note::Idea { ref mut extra, .. })
                | Some(Note::Til { ref mut extra, .. })
                | Some(Note::Qts { ref mut extra, .. })
                | Some(Note::QtsAnswer { ref mut extra, .. })
                | Some(Note::Calendar { ref mut extra, .. })
                | Some(Note::General { ref mut extra, .. }) => {
                    extra.push_str(note_line);
                    extra.push('\n');
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

fn note_sorter(entry: &Note) -> (i32, &str) {
    match entry {
        Note::Todo { .. } => (1, "0"),
        Note::Idea { .. } => (1, "1"),
        Note::Til { .. } => (1, "2"),
        Note::Qts { .. } => (1, "3"),
        Note::QtsAnswer { .. } => (1, "3"),
        Note::Calendar { date: date_a, .. } => (2, date_a.as_str()),
        Note::General { tag, .. } => (3, tag.as_str()),
    }
}

fn display_entries(dvlg: &Dvlg, entry_type: &str, tags: Option<&str>) {
    for (date, entries) in &dvlg.entries {
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|entry| {
                if entry_type == "fmt" {
                    return true;
                }
                match entry {
                    Note::Todo { state, .. } if TODO_OPTS.contains(&entry_type) => {
                        match (entry_type, state.as_str()) {
                            ("todo", " ") | ("done", "x") | ("doing", "/") | ("dropped", "-") => {
                                true
                            }
                            _ => false,
                        }
                    }
                    Note::Til { .. } if entry_type == "til" => true,
                    Note::Qts { .. } if entry_type == "qts" => true,
                    Note::Calendar { .. } if entry_type == "cal" => true,
                    Note::General { tag, .. } if entry_type == "note" => {
                        tags.map_or(true, |t| tag.contains(t))
                    }
                    _ => false,
                }
            })
            .collect();

        filtered.sort_by_key(|event| {
            if !["fmt", "cal"].contains(&entry_type) {
                return (1, "");
            }
            note_sorter(event)
        });

        if filtered.is_empty() {
            continue;
        }

        if entry_type != "cal" {
            println!("@{}", date);
        }

        let add_slash = |s: &String| {
            if s.is_empty() {
                s.to_string()
            } else {
                format!("/{}", s)
            }
        };

        let print_entry = |prefix: String, extra: &String| {
            println!("{}", prefix);
            if !extra.is_empty() {
                if entry_type == "fmt" {
                    println!("{}", extra.trim());
                } else {
                    print!("{}", extra);
                }
            }
        };

        let mut prev_type: Option<_> = None;

        for entry in filtered {
            let current_type = note_sorter(entry);
            match (current_type, prev_type) {
                ((2, _), Some((2, _))) | ((3, _), Some((3, _))) => (),
                (a, Some(b)) if a != b => println!(),
                ((1, _), Some((1, _))) | (_, _) => (),
            }
            match entry {
                Note::Todo {
                    state,
                    title,
                    extra,
                } => {
                    print_entry(format!("- [{}] {}", state, title), extra);
                }
                Note::Idea { title, extra, tag } => {
                    print_entry(format!("{}$ {}", add_slash(tag), title), extra);
                }
                Note::Til { title, extra, tag } => {
                    print_entry(format!("{}! {}", add_slash(tag), title), extra);
                }
                Note::Qts {
                    question,
                    extra,
                    tag,
                } => {
                    print_entry(format!("{}? {}", add_slash(tag), question), extra);
                }
                Note::QtsAnswer { answer, extra, tag } => {
                    print_entry(format!("{}?! {}", add_slash(tag), answer), extra);
                }
                Note::Calendar { date, title, extra } => {
                    print_entry(format!("[{}] {}", date, title), extra);
                }
                Note::General { tag, title, extra } => {
                    print_entry(format!("{}/ {}", add_slash(tag), title), extra);
                }
            }
            prev_type = Some(current_type);
        }
        println!()
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
