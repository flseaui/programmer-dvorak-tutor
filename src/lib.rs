#[macro_use]
extern crate clap;

mod io;

use crossterm::style::Colorize;
use crossterm::{
    cursor::{position, Hide, MoveDown, MoveLeft, MoveRight, MoveTo},
    execute,
    input::{input, AsyncReader, InputEvent, KeyEvent},
    screen::{AlternateScreen, RawScreen},
    style::{style, Color, PrintStyledContent},
    utils::Output,
};
use indexmap::map::IndexMap;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fs::read_to_string;
use std::io::{stdout, Stdout, Write};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize, Clone)]
pub struct Lesson {
    text: String,
    id: String,
}

impl Lesson {
    fn new(text: String, id: String) -> Lesson {
        Lesson { text, id }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    last_lesson_id: String,
}

/// An input (user) event.
#[derive(Debug)]
pub enum Event {
    InputCharacter(char),
    NewLine,
    Quit,
}

lazy_static! {
    pub static ref LESSONS: IndexMap<String, Lesson> = io::lesson::load_lessons();
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        println!("{}", LESSONS[lesson_str].id);

        run_lesson(&LESSONS[lesson_str])
    }

    if matches.is_present("continue") {
        let stats = io::stats::load_stats();
    }
}

fn run_lesson(lesson: &Lesson) {
    let alt = AlternateScreen::to_alternate(true);

    #[allow(unused)]
    let screen = RawScreen::into_raw_mode();

    let mut stdin = input().read_async();
    let mut stdout = stdout();

    execute!(&mut stdout, Hide);

    let mut lines = lesson.text.lines();
    let title = lines.next().unwrap().to_string();

    execute!(&mut stdout, Output(title), MoveTo(0, 1));

    'outer: for line in lines {
        execute!(
            &mut stdout,
            Output(line.to_string()),
            MoveDown(1),
            MoveTo(0, position().unwrap().1)
        );

        let chars: Vec<char> = line.chars().collect();

        let line_length = chars.len();

        let mut char_index = 0;

        'char: loop {
            'input: loop {
                match next_event(&mut stdin) {
                    Some(Event::InputCharacter(character)) => {
                        // if at end of line don't write character
                        if char_index < line_length {
                            write_character(&mut stdout, chars[char_index], character);
                            break;
                        }
                    }
                    Some(Event::NewLine) => {
                        if char_index == line_length {
                            execute!(&mut stdout, MoveDown(1), MoveTo(0, position().unwrap().1));
                            break 'char;
                        }
                    }
                    Some(Event::Quit) => break 'outer,
                    _ => {}
                }
            }
            char_index += 1;
        }
    }
}

fn next_event(reader: &mut AsyncReader) -> Option<Event> {
    while let Some(event) = reader.next() {
        match event {
            InputEvent::Keyboard(KeyEvent::Char(character)) => {
                return Some(Event::InputCharacter(character));
            }
            InputEvent::Keyboard(KeyEvent::Esc) => return Some(Event::Quit),
            InputEvent::Keyboard(KeyEvent::Ctrl('c')) => return Some(Event::Quit),
            InputEvent::Keyboard(key) => {
                if key == KeyEvent::Enter {
                    return Some(Event::NewLine);
                }
            }
            _ => {}
        };
    }
    None
}

fn write_character(stdout: &mut Stdout, current_char: char, input_char: char) {
    let content = if current_char == input_char {
        style(input_char).black().on_green()
    } else {
        style(input_char).black().on_red()
    };

    execute!(stdout, PrintStyledContent(content));
}
