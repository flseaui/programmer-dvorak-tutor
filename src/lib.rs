#[macro_use]
extern crate clap;
extern crate termion;

use clap::ErrorKind;
use colored::Colorize;
use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;
use std::fs::read_to_string;
use std::io::{stdin, stdout, Error, Read, Stdin, Stdout, StdoutLock, Write};
use std::ops::Deref;
use std::str::Chars;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::event::Key::Char;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Lesson {
    text: String,
}

impl Lesson {
    fn new(text: String) -> Lesson {
        Lesson { text }
    }
}

pub struct Session {
    stdout: RawTerminal<Stdout>,
    stdin: Stdin,
    lesson: Cell<Option<Lesson>>,
}

impl Session {
    pub fn new(stdout: RawTerminal<Stdout>, stdin: Stdin) -> Session {
        Session {
            stdout,
            stdin,
            lesson: Cell::new(Some(Lesson::new(String::from("TEST")))),
        }
    }
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    let session = start_session();

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        session.lesson.set(Some(load_lesson(lesson_str)));
    }
}

fn start_session() -> &'static Session {
    let stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();

    let session = Session::new(stdout, stdin);

    &session;

    loop {
        match &session.lesson.take() {
            Some(lesson) => {
                let again = run_lesson(&lesson, &session);
            }
            _ => {}
        }
    }
}

fn load_lesson(lesson: &str) -> Lesson {
    let path = format!("lessons/{0}.txt", lesson);
    //println!("{}", path);
    let lesson_string = read_to_string(path).unwrap();
    Lesson::new(lesson_string)
}

pub fn run_lesson(lesson: &Lesson, session: &Session) -> bool {
    let stdin = &mut session.stdin.lock();
    let stdout = &mut session.stdout.lock();
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
    update_cursor_pos(&mut cursor_x, &mut cursor_y, stdout);

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
        update_cursor_pos(&mut cursor_x, &mut cursor_y, stdout);

        let mut line_chars = line.chars();
        let line_length = line_chars.clone().count() as u16;
        'character: loop {
            for c in stdin.keys() {
                match c.unwrap() {
                    Key::Char(c) => {
                        if cursor_x == line_length + 1 {
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

                update_cursor_pos(&mut cursor_x, &mut cursor_y, stdout);
                stdout.flush().unwrap();
            }
        }
    }

    // Lesson finished
    write!(
        stdout,
        "{}{} finished, next lesson? (y/n) ",
        termion::cursor::Goto(1, cursor_y + 1),
        title
    );

    // true - yes, false - no
    let answer: bool = match stdin.read_line() {
        Ok(line) => match line {
            None => true,
            Some(answer) => {
                if answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes") {
                    true
                } else {
                    false
                }
            }
        },
        Err(_err) => false,
    };

    return answer;
}
