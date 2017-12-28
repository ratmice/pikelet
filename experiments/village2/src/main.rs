extern crate failure;
extern crate village2;

use failure::Error;
use std::process;

fn on_success() -> i32 {
    println!("closing");
    0
}

fn on_error(error: Error) -> i32 {
    eprintln!("{}", error);
    1
}

fn main() {
    process::exit(match village2::run() {
        Ok(()) => on_success(),
        Err(error) => on_error(error),
    });
}
