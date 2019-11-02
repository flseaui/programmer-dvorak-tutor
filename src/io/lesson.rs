use indexmap::map::IndexMap;
use walkdir::{DirEntry, WalkDir};
use crate::Lesson;
use std::cmp::Ordering::{Less, Greater, Equal};
use std::fs::read_to_string;
use std::cmp::Ordering;

pub fn load_lessons() -> IndexMap<String, Lesson> {
    fn is_lesson(entry: &DirEntry) -> bool {
        entry.file_name().to_str().unwrap().starts_with("lesson_")
    }

    let mut lessons: IndexMap<String, Lesson> = IndexMap::new();

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
            .insert(id.to_string(), lesson);
    }
    lessons
}

/* 0 - less than
 * 1 - greater than
 * 2 - equal to
 */
fn compare_lesson_id(id_a: &str, id_b: &str) -> Ordering {
    let letter_a = id_a.trim_matches(char::is_numeric);
    let letter_b = id_a.trim_matches(char::is_numeric);

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