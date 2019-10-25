#[macro_use]
extern crate clap;

extern crate termion;

#[macro_use]
extern crate lazy_static;

use clap::ErrorKind;
use colored::Colorize;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fs::{read_to_string, File};
use std::io::{stdin, stdout, Read, Stdin, StdinLock, Stdout, StdoutLock, Write};
use std::marker::Copy;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::str::Chars;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::sync::{mpsc, Arc};
use std::thread::{JoinHandle, Thread};
use std::{env, fs, thread};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::event::Key::Char;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize, Clone)]
pub struct Lesson {
    text: String,
    id: String,
}

unsafe impl Send for Lesson {}

impl Lesson {
    fn new(text: String, id: String) -> Lesson {
        Lesson { text, id }
    }
}

#[derive(Serialize, Deserialize)]
struct Stats {
    last_lesson_id: String,
}

lazy_static! {
    pub static ref LESSONS: Mutex<IndexMap<String, Arc<Lesson>>> = load_lessons();
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    let (tx, rx) = mpsc::channel();

    let handle = run(rx);

    let mut lessons_lock = &mut LESSONS.lock().unwrap();

    let loader = &lessons_lock["01"];

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        let lesson = lessons_lock[lesson_str].clone();
        tx.send(lesson).expect("Lesson message sending failed!");
    }

    if matches.is_present("continue") {
        let stats = load_stats();

        let index = lessons_lock.entry(stats.last_lesson_id).index() + 1;
        let lesson = lessons_lock.get_index(index).unwrap().1.clone();
        tx.send(lesson).expect("Lesson message sending failed!");
    }

    get_next_lesson();
    handle.join();
}

fn run(rx: mpsc::Receiver<Arc<Lesson>>) -> JoinHandle<()> {
    let lesson_runner = thread::spawn(move || loop {
        for lesson in &rx {
            let again = run_lesson(&lesson);
            save_stats(lesson.deref().clone());

            if again {
            } else {
                //return;
            }
        }
    });
    lesson_runner
}

fn load_stats() -> Stats {
    let stats_json = read_to_string("stats.json").unwrap();

    let stats: Stats = serde_json::from_str(stats_json.as_str()).unwrap();

    stats
}

fn save_stats(lesson: Lesson) {
    let stats = Stats {
        last_lesson_id: lesson.id,
    };
    let json = serde_json::to_string(&stats).unwrap();

    let path = Path::new("stats.json");
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(json.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn load_lessons() -> Mutex<IndexMap<String, Arc<Lesson>>> {
    fn is_lesson(entry: &DirEntry) -> bool {
        entry.file_name().to_str().unwrap().starts_with("lesson_")
    }

    let mut lessons: Mutex<IndexMap<String, Arc<Lesson>>> = Mutex::new(IndexMap::new());

    let walker = WalkDir::new("lessons")
        .sort_by(|a, b| {
            if !is_lesson(a) {
                return Less;
            }
            if !is_lesson(b) {
                return Greater;
            }

            let id_a = a
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches("lesson_")
                .trim_end_matches(".txt");

            let id_b = b
                .file_name()
                .to_str()
                .unwrap()
                .trim_start_matches("lesson_")
                .trim_end_matches(".txt");

            compare_lesson_id(id_a, id_b)
        })
        .contents_first(true)
        .into_iter();
    for entry in walker.filter_entry(|e| is_lesson(e) && e.file_type().is_file()) {
        let entry = entry.unwrap();
        let id = entry
            .file_name()
            .to_str()
            .unwrap()
            .trim_start_matches("lesson_")
            .trim_end_matches(".txt");

        let lesson_string = read_to_string(entry.path()).unwrap();
        let lesson = Lesson::new(lesson_string, id.to_string());

        lessons
            .lock()
            .unwrap()
            .insert(id.to_string(), Arc::new(lesson));
    }
    lessons
}

fn get_next_lesson() {
    fn is_lesson(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("lesson_"))
            .unwrap_or(false)
    }

    let walker = WalkDir::new("lessons").into_iter();
    for entry in walker.filter_entry(|e| !is_lesson(e) && e.file_type().is_file()) {
        let entry = entry.unwrap();
        let id = entry
            .file_name()
            .to_str()
            .unwrap()
            .trim_start_matches("lesson_")
            .trim_end_matches(".txt");
    }
}

/* 0 - less than
 * 1 - greater than
 * 2 - equal to
 */
fn compare_lesson_id(id_a: &str, id_b: &str) -> Ordering {
    let letter_a = id_a.trim_matches(char::is_numeric);
    let letter_b = id_a.trim_matches(char::is_numeric);

    println!("{}", id_a.trim_matches(char::is_alphabetic));
    let number_a = id_a
        .trim_matches(char::is_alphabetic)
        .parse::<u32>()
        .unwrap();
    let number_b = id_b
        .trim_matches(char::is_alphabetic)
        .parse::<u32>()
        .unwrap();

    if number_a > number_b {
        return Greater;
    } else if number_a == number_b {
        if letter_a.is_empty() || letter_b.is_empty() {
            return Equal;
        }
        if letter_a > letter_b {
            return Greater;
        }
        return Less;
    } else {
        return Less;
    }
}
/*

fn load_lesson(lesson: &str) -> Lesson {
    let path = format!("LESSONS/{0}.txt", lesson);
    //println!("{}", path);
    let lesson_string = read_to_string(path).unwrap();
    Lesson::new(lesson_string)
}
*/

pub fn run_lesson(lesson: &Lesson) -> bool {
    let raw_stdout = stdout().into_raw_mode().unwrap();
    let raw_stdin = stdin();

    let mut stdout = raw_stdout.lock();
    let mut stdin = raw_stdin.lock();

    let (mut cursor_x, mut cursor_y) = stdout.cursor_pos().unwrap();

    fn update_cursor_pos(cursor_x: &mut u16, cursor_y: &mut u16, stdout: &mut StdoutLock) {
        let (temp_cursor_x, temp_cursor_y) = stdout.cursor_pos().unwrap();
        *cursor_x = temp_cursor_x;
        *cursor_y = temp_cursor_y;
    }

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    );
    update_cursor_pos(&mut cursor_x, &mut cursor_y, &mut stdout);

    let top_offset = 1;
    let first_line = top_offset + 1;

    let mut lines = lesson.text.lines();
    let title = lines.next().unwrap();
    writeln!(stdout, "{}", title);

    for line in lines {
        write!(
            stdout,
            "{}{}\n\r{}",
            termion::cursor::Goto(
                1,
                if cursor_y == first_line {
                    first_line
                } else {
                    cursor_y + 1
                }
            ),
            line,
            termion::cursor::Goto(
                1,
                if cursor_y == first_line {
                    first_line + 1
                } else {
                    cursor_y + 2
                }
            )
        );
        stdout.flush().unwrap();
        update_cursor_pos(&mut cursor_x, &mut cursor_y, &mut stdout);

        let mut line_chars = line.chars();
        let line_length = line_chars.clone().count() as u16;
        'character: loop {
            let mut keys = stdin.by_ref().keys();
            for c in keys {
                match c.unwrap() {
                    Key::Char(c) => {
                        if cursor_x >= line_length + 1 {
                            if c == '\n' {
                                break 'character;
                            }
                        } else {
                            if c == '\n' {
                            } else if c == line_chars.clone().nth(cursor_x as usize - 1).unwrap() {
                                write!(stdout, "{}", c.to_string().on_bright_green());
                                stdout.flush().unwrap();
                            //break 'character;
                            } else {
                                write!(stdout, "{}", c.to_string().on_bright_red());
                            }
                        }
                    }
                    Key::Ctrl(c) => {
                        write!(stdout, "Ctrl-{}", c);
                        if c == 'c' || c == 'C' {
                            std::process::exit(1);
                        }
                    }
                    Key::Alt(c) => {
                        write!(stdout, "Alt-{}", c);
                    }
                    Key::Backspace => {
                        write!(
                            stdout,
                            "{}{}",
                            termion::cursor::Left(1),
                            termion::clear::AfterCursor
                        );
                    }
                    _ => {
                        write!(stdout, "Other");
                    }
                }

                update_cursor_pos(&mut cursor_x, &mut cursor_y, &mut stdout);
                stdout.flush().unwrap();
            }
        }

        if cursor_y > terminal_size().unwrap().1 - 1 {
            write!(stdout, "{}", termion::scroll::Up(1));
        }
    }

    // Lesson finished
    write!(
        stdout,
        "{}{} finished, next lesson? (y/n) {}",
        termion::cursor::Goto(1, cursor_y + 1),
        title,
        termion::cursor::Right(1)
    );
    stdout.flush().unwrap();
    update_cursor_pos(&mut cursor_x, &mut cursor_y, &mut stdout);
    // true - yes, false - no

    let start_x = cursor_x.clone();
    let mut answer = false;
    'outer: loop {
        for key in stdin.by_ref().keys() {
            match key.unwrap() {
                Key::Char(c) => {
                    if c == '\n' {
                        break 'outer;
                    } else {
                        let x = c.eq_ignore_ascii_case(&'n');
                        let y = c.eq_ignore_ascii_case(&'y');
                        answer = y;
                        if x || y {
                            if cursor_x != start_x + 1 {
                                write!(
                                    stdout,
                                    "{}{}",
                                    termion::cursor::Goto(cursor_x - 1, cursor_y),
                                    c
                                );
                            } else {
                                write!(stdout, "{}", c);
                            }
                        }
                    }
                }
                _ => {}
            }

            stdout.flush().unwrap();
        }
    }

    answer
}
