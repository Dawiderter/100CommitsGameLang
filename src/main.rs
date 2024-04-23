use std::path::{Path, PathBuf};

use game_lang::{bytecode::{chunk::CodeChunk, object::ObjectHeap, vm::VM}, cli::reporter::{report_parsing_error, report_runtime_error}, compiler::parser::Parser};

#[derive(clap::Parser)]
struct Args {
    #[arg(short,long)]
    input: Option<PathBuf>
}

/// Simple REPL
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("game_lang", log::LevelFilter::Trace)
        .format_timestamp(None)
        .init();

    let args = <Args as clap::Parser>::parse();

    if let Some(input_path) = args.input {
        file(&input_path);
    } else {
        repl();
    }
}

fn file(input_path: &Path) {
    let input = std::fs::read_to_string(input_path).unwrap();
    let name = input_path.to_string_lossy();
    let mut code = CodeChunk::new();
    let mut heap = ObjectHeap::new();

    if let Err(errors) = Parser::parse_source(&input, &mut code, &mut heap) {
        for err in errors {
            report_parsing_error(&name, &input, err);
        }
        return;
    }
    let mut vm = VM::init(&code, &mut heap);
    if let Err(err) = vm.run() {
        report_runtime_error(&name, &input, err, vm.current_span())
    }
}

fn repl() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let mut heap = ObjectHeap::new();

    loop {
        let line = match rl.readline(">> "){
            Ok(line) => line,
            Err(err) => { eprintln!("{}", err); break;}
        };

        let mut code = CodeChunk::new();
        if let Err(errors) = Parser::parse_source(&line, &mut code, &mut heap) {
            for err in errors {
                report_parsing_error("REPL", &line, err);
            }
            continue;
        }

        let mut vm = VM::init(&code, &mut heap);
        if let Err(err) = vm.run() {
            report_runtime_error("REPL", &line, err, vm.current_span())
        }
    }
}