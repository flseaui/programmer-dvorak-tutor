#[macro_use]
extern crate clap;

extern crate termion;

pub fn create_app() {
    let yaml = load_yaml!("../cli.yml");
    let matches = clap::App::from(yaml).get_matches();
}
