use clap::{Arg, App};
use serde_json;

pub mod board;
pub mod piece;
pub mod serialization;

fn main() {
    let matches = App::new("Chess Engine (Rust)")
        .version("0.1")
        .author("Michael Leonard <maybeillrememberit@gmail.com")
        .about("An experimental chess engine written in Rust")
        .arg(Arg::with_name("display")
            .help("Display the given board-file")
            .takes_value(true))
        .get_matches();

    let display = matches.value_of("display").unwrap();
    println!("{}", display);
}
