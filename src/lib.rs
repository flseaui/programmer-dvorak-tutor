#[macro_use]
extern crate clap;

mod io;

use walkdir::{DirEntry, WalkDir};
use std::cmp::Ordering::{Less, Greater, Equal};
use std::fs::read_to_string;
use std::cmp::Ordering;
use serde::Deserialize;
use serde::Serialize;
use lazy_static::lazy_static;
use io::lesson;
use indexmap::map::IndexMap;

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
struct Stats {
    last_lesson_id: String,
}

lazy_static! {
    pub static ref LESSONS: IndexMap<String, Lesson> = lesson::load_lessons();
}

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    if matches.is_present("lesson") {
        let lesson_str = matches.value_of("lesson").unwrap();
        println!("{}", LESSONS["01"].id);
    }
}

fn run_lesson() {

}