#[macro_use]
extern crate clap;
extern crate termion;

use clap::ErrorKind;
use colored::Colorize;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::fs::read_to_string;
use std::io::{stdin, stdout, Error, Read, Stdin, StdinLock, Stdout, StdoutLock, Write};
use std::ops::Deref;
use std::rc::Rc;
use std::str::Chars;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::{JoinHandle, Thread};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::event::Key::Char;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Lesson {
    text: String,
}

unsafe impl Send for Lesson {}

impl Lesson {
    fn new(text: String) -> Lesson {
        Lesson { text }
    }
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    let (tx, rx) = mpsc::channel();

    let handle = run(rx);

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        tx.send(load_lesson(lesson_str))
            .expect("Lesson message sending failed!");
    }

    handle.join();
}

fn run(rx: mpsc::Receiver<Lesson>) -> JoinHandle<()> {
    let lesson_runner = thread::spawn(move || loop {
        for lesson in &rx {
            let again = run_lesson(&lesson);
            if again {

            } else {
                //return;
            }
        }
    });
    lesson_runner
}

fn load_lesson(lesson: &str) -> Lesson {
    let path = format!("lessons/{0}.txt", lesson);
    //println!("{}", path);
    let lesson_string = read_to_string(path).unwrap();
    Lesson::new(lesson_string)
}

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
