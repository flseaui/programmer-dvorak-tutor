#[macro_use]
extern crate clap;

mod io;

use crossterm::style::Colorize;
use crossterm::terminal::size;
use crossterm::{
    cursor::{position, Hide, MoveDown, MoveLeft, MoveRight, MoveTo},
    execute,
    input::{input, AsyncReader, InputEvent, KeyEvent},
    screen::{AlternateScreen, RawScreen},
    style::{style, Color, PrintStyledContent},
    terminal::{Clear, ClearType, ScrollDown, ScrollUp},
    utils::Output,
};
use indexmap::map::IndexMap;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering::{Equal, Greater, Less};
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
    BackSpace,
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

        let mut next_lesson = lesson_str;

        let test: IndexMap<String, Lesson> = IndexMap::new();
        test.get_index(1);

        loop {
            if run_lesson(&LESSONS[next_lesson]) {
                let index = LESSONS.get_full(next_lesson).unwrap().0;
                next_lesson = LESSONS.get_index(index + 1).unwrap().0;
                continue;
            } else {
                println!("NOOOO");
                break;
            }
        }
    }

    if matches.is_present("continue") {
        let stats = io::stats::load_stats();
    }
}

fn run_lesson(lesson: &Lesson) -> bool {
    let alt = AlternateScreen::to_alternate(true);

    #[allow(unused)]
    let screen = RawScreen::into_raw_mode();

    let mut stdin = input().read_async();
    let mut stdout = stdout();

    execute!(&mut stdout, Hide);

    let mut lines = lesson.text.lines();
    let title = lines.next().unwrap().to_string();

    execute!(&mut stdout, MoveTo(0, 0), Output(title), MoveTo(0, 1));

    'outer: for line in lines {
        // if at bottom of terminal, scroll up
        let scroll = position().unwrap().1 >= size().unwrap().1 - 1;

        if scroll {
            execute!(&mut stdout, ScrollUp(1));
        }

        execute!(
            &mut stdout,
            Output(line.to_string()),
            MoveTo(0, position().unwrap().1 + 1)
        );

        if scroll {
            execute!(&mut stdout, ScrollUp(1));
        }

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
                    Some(Event::BackSpace) => {
                        if position().unwrap().0 > 0 {
                            execute!(&mut stdout, MoveLeft(1), Clear(ClearType::UntilNewLine));
                            char_index -= 1;
                        }
                    }
                    Some(Event::Quit) => break 'outer,
                    _ => {}
                }
            }
            char_index += 1;
        }
    }

    // Lesson finished

    // If at bottom scroll up
    if position().unwrap().1 >= size().unwrap().1 - 1 {
        execute!(&mut stdout, ScrollUp(1));
    }

    execute!(
        &mut stdout,
        MoveTo(0, position().unwrap().1 + 1),
        Output("Lesson finished, next lesson? (y/n) ")
    );

    let start_x = position().unwrap().0;

    let mut answer = false;

    loop {
        match next_event(&mut stdin) {
            Some(Event::InputCharacter(mut character)) => {
                character.make_ascii_lowercase();
                if let 'y' | 'n' = character {
                    if position().unwrap().0 > start_x {
                        execute!(&mut stdout, MoveLeft(1), Clear(ClearType::UntilNewLine));
                    }
                    execute!(&mut stdout, Output(character));

                    if character == 'y' {
                        answer = true;
                    } else if character == 'n' {
                        answer = false;
                    }
                }
            }

            Some(Event::NewLine) => {
                return answer;
            }

            Some(Event::Quit) => {
                break;
            }

            _ => {}
        }
    }

    return false;
}

fn next_event(reader: &mut AsyncReader) -> Option<Event> {
    while let Some(event) = reader.next() {
        match event {
            InputEvent::Keyboard(KeyEvent::Char(character)) => {
                return Some(Event::InputCharacter(character));
            }
            InputEvent::Keyboard(KeyEvent::Esc) => return Some(Event::Quit),
            InputEvent::Keyboard(KeyEvent::Ctrl('c')) => return Some(Event::Quit),

            InputEvent::Keyboard(KeyEvent::Backspace) => return Some(Event::BackSpace),
            InputEvent::Keyboard(KeyEvent::Enter) => return Some(Event::NewLine),
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
