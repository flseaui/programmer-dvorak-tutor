#[macro_use]
extern crate clap;
extern crate termion;

use colored::Colorize;
use std::borrow::{Borrow, BorrowMut};
use std::fs::read_to_string;
use std::io::{stdin, stdout, Read, Stdin, Stdout, StdoutLock, Write};
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
}

impl Session {
    pub fn new(stdout: RawTerminal<Stdout>, stdin: Stdin) -> Session {
        Session { stdout, stdin }
    }
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    let stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();

    let session = Session::new(stdout, stdin);

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        let lesson = load_lesson(lesson_str);

        run_lesson(&lesson, &session);
    }
}

fn load_lesson(lesson: &str) -> Lesson {
    let path = format!("lessons/{0}.txt", lesson);
    //println!("{}", path);
    let lesson_string = read_to_string(path).unwrap();
    Lesson::new(lesson_string)
}

pub fn run_lesson(lesson: &Lesson, session: &Session) {
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
    for line in lesson.text.lines() {
        write!(
            stdout,
            "{}\n\r{}",
            line,
            termion::cursor::Goto(cursor_x - 1, cursor_y + 1)
        );
        stdout.flush().unwrap();

        let mut line_chars = line.chars();

        'character: loop {
            for c in stdin.keys() {
                match c.unwrap() {
                    Key::Char(c) => {
                        if c == '\n' {

                        } else if c == line_chars.clone().nth(cursor_x as usize - 1).unwrap() {
                            write!(stdout, "{}", c.to_string().on_bright_green());
                            stdout.flush().unwrap();
                        //break 'character;
                        } else {
                            write!(stdout, "{}", c.to_string().on_bright_red());
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
                let (temp_cursor_x, temp_cursor_y) = stdout.cursor_pos().unwrap();
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Goto(1, 4),
                    cursor_x as usize
                );
                cursor_x = temp_cursor_x;
                cursor_y = temp_cursor_y;
                write!(stdout, "{}", termion::cursor::Goto(cursor_x, cursor_y));
                stdout.flush().unwrap();
            }
        }
    }
}
