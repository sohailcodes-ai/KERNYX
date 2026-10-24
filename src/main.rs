#![allow(dead_code)]

mod ast;
mod cli;
mod diagnostic;
mod hir;
mod lexer;
mod parser;
mod runtime;
mod sema;
mod source;

use crate::cli::{execute, parse_args};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = match parse_args(&args) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("error: {}", message);
            eprintln!("Run `kernyx help` for usage.");
            std::process::exit(2);
        }
    };
    let exit_code = execute(command);
    std::process::exit(exit_code);
}
