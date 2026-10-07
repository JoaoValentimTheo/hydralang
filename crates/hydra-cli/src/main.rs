use hydra_cli::compile;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--version" || flag == "-V" => {
            println!("hydra {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [] => {
            print_help();
            ExitCode::SUCCESS
        }
        [command] if command == "help" || command == "--help" || command == "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        [command, path] if command == "check" => command_check(path),
        [command, path] if command == "run" => command_run(path),
        _ => {
            eprintln!("error: invalid arguments\n");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn command_check(path: &str) -> ExitCode {
    let Some(text) = read_source(path) else {
        return ExitCode::from(2);
    };
    let result = compile(path, &text);
    if result.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        for diagnostic in &result.diagnostics {
            eprint!("{}", diagnostic.render(&result.sources));
        }
        ExitCode::from(1)
    }
}

fn command_run(path: &str) -> ExitCode {
    let Some(text) = read_source(path) else {
        return ExitCode::from(2);
    };
    let result = compile(path, &text);
    if !result.diagnostics.is_empty() {
        for diagnostic in &result.diagnostics {
            eprint!("{}", diagnostic.render(&result.sources));
        }
        return ExitCode::from(1);
    }
    let Some(hir) = result.hir else {
        eprintln!("error[E9005]: compiler produced no HIR without diagnostics");
        return ExitCode::from(1);
    };
    let run = hydra_runtime::execute(&hir);
    print!("{}", run.output);
    if run.diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        for diagnostic in &run.diagnostics {
            eprint!("{}", diagnostic.render(&result.sources));
        }
        ExitCode::from(1)
    }
}

fn read_source(path: &str) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            eprintln!("error: could not read `{path}`: {error}");
            None
        }
    }
}

fn print_help() {
    println!(
        "Hydra programming language\n\nUSAGE:\n    hydra check <file.hyd>\n    hydra run <file.hyd>\n    hydra --version\n    hydra help"
    );
}
