use crate::diagnostic::{Diagnostic, Formatter, Severity};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime::Interpreter;
use crate::sema::check_program;
use crate::source::Source;
use std::fs;
use std::path::PathBuf;

const VERSION: &str = "0.1.0";

pub enum Command {
    Version,
    Help,
    Run { path: PathBuf },
    Check { path: PathBuf },
}

pub fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.len() < 2 {
        return Ok(Command::Help);
    }
    match args[1].as_str() {
        "--version" | "-V" => Ok(Command::Version),
        "help" | "--help" | "-h" => Ok(Command::Help),
        "run" => {
            if args.len() < 3 {
                return Err("`run` requires a source file: kernyx run <file>.knx".to_string());
            }
            Ok(Command::Run {
                path: PathBuf::from(&args[2]),
            })
        }
        "check" => {
            if args.len() < 3 {
                return Err("`check` requires a source file: kernyx check <file>.knx".to_string());
            }
            Ok(Command::Check {
                path: PathBuf::from(&args[2]),
            })
        }
        other => Err(format!(
            "unknown argument: `{}`\nRun `kernyx help` for usage.",
            other
        )),
    }
}

pub fn usage() -> String {
    String::from(
        "KERNYX 0.1.0 - a typed, interpreted programming language\n\
         \n\
         USAGE:\n\
             kernyx <command> [arguments]\n\
         \n\
         COMMANDS:\n\
             run <file>.knx    Parse, type-check, and execute a KERNYX program\n\
             check <file>.knx  Parse and type-check a program without executing\n\
             --version         Print the compiler version\n\
             help              Print this help message\n\
         \n\
         EXIT CODES:\n\
             0  success\n\
             1  compilation or runtime error\n\
             2  usage error\n",
    )
}

/// Compile (lex, parse, type-check) a source file, returning the typed program
/// and any diagnostics.
fn compile_file(
    path: &PathBuf,
) -> Result<(Option<crate::hir::TypedProgram>, Vec<Diagnostic>), String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read `{}`: {}", path.display(), e))?;
    let source = Source::new(path, text);
    let (tokens, lex_errors) = Lexer::new(&source).lex();
    let (program, parse_errors) = Parser::new(tokens).parse();
    let (typed, check_diag) = check_program(&source, program);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    diagnostics.extend(lex_errors);
    diagnostics.extend(parse_errors);
    diagnostics.extend(check_diag.items);

    Ok((typed, diagnostics))
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| d.severity == Severity::Error)
}

pub fn execute(command: Command) -> i32 {
    match command {
        Command::Version => {
            println!("kernyx {}", VERSION);
            0
        }
        Command::Help => {
            print!("{}", usage());
            0
        }
        Command::Run { path } => match compile_file(&path) {
            Ok((Some(typed), diagnostics)) => {
                if has_errors(&diagnostics) {
                    report_diagnostics(&diagnostics, &path);
                    return 1;
                }
                let mut interpreter = Interpreter::new(typed);
                if let Err(diag) = interpreter.run() {
                    let diagnostics = vec![diag];
                    report_diagnostics(&diagnostics, &path);
                    return 1;
                }
                0
            }
            Ok((None, diagnostics)) => {
                report_diagnostics(&diagnostics, &path);
                1
            }
            Err(message) => {
                eprintln!("error: {}", message);
                2
            }
        },
        Command::Check { path } => match compile_file(&path) {
            Ok((Some(_typed), diagnostics)) => {
                if has_errors(&diagnostics) {
                    report_diagnostics(&diagnostics, &path);
                    return 1;
                }
                println!("check: `{}`: no errors", path.display());
                0
            }
            Ok((None, diagnostics)) => {
                report_diagnostics(&diagnostics, &path);
                1
            }
            Err(message) => {
                eprintln!("error: {}", message);
                2
            }
        },
    }
}

fn report_diagnostics(diagnostics: &[Diagnostic], path: &PathBuf) {
    let source_text = fs::read_to_string(path).unwrap_or_default();
    let source = Source::new(path, source_text);
    let sources = [source];
    let formatter = Formatter::new(&sources);
    for diagnostic in diagnostics {
        let mut d = diagnostic.clone();
        d.source = Some(path.to_string_lossy().to_string());
        eprint!("{}", formatter.format(&d));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands() {
        assert!(matches!(
            parse_args(&["kernyx".to_string(), "--version".to_string()]),
            Ok(Command::Version)
        ));
        assert!(matches!(
            parse_args(&["kernyx".to_string(), "help".to_string()]),
            Ok(Command::Help)
        ));
        assert!(matches!(
            parse_args(&["kernyx".to_string(), "run".to_string(), "examples/hello.knx".to_string()]),
            Ok(Command::Run { path }) if path == *"examples/hello.knx"
        ));
        assert!(matches!(
            parse_args(&[
                "kernyx".to_string(),
                "check".to_string(),
                "bad.knx".to_string()
            ]),
            Ok(Command::Check { .. })
        ));
    }

    #[test]
    fn rejects_missing_file_argument() {
        assert!(parse_args(&["kernyx".to_string(), "run".to_string()]).is_err());
        assert!(parse_args(&["kernyx".to_string(), "check".to_string()]).is_err());
    }

    #[test]
    fn rejects_unknown_argument() {
        assert!(parse_args(&["kernyx".to_string(), "frobnicate".to_string()]).is_err());
    }

    #[test]
    fn no_arguments_shows_help() {
        assert!(matches!(
            parse_args(&["kernyx".to_string()]),
            Ok(Command::Help)
        ));
    }
}
