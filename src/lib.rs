#[macro_use]
extern crate clap;

use std::fs::{read_to_string, File};
use std::io::Read;
use std::path::Path;

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();

    if matches.is_present("lesson") {
        let lesson = matches.value_of("lesson").unwrap();
        load_lesson(lesson);
    }
}

pub fn load_lesson(lesson: &str) {
    let path = format!("lessons/{0}.txt", lesson);
    println!("{}", path);
    let lesson_string = read_to_string(path).unwrap();
    println!("{}", lesson_string);
}
