use std::ops::Range;

use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use owo_colors::OwoColorize;

use crate::{bytecode::vm::RuntimeError, compiler::parser::ParsingError};

pub fn report_parsing_error(name: &str, src: &str, err: ParsingError) {
    Report::build(ReportKind::Error, name, err.span.start)
        .with_config(Config::default().with_compact(true))
        .with_message(err.msg)
        .with_label(
            Label::new((name, err.span))
                .with_message("Here".red())
                .with_color(Color::Red),
        )
        .finish()
        .print((name, Source::from(src)))
        .unwrap()
}

pub fn report_runtime_error(name: &str, src: &str, err: RuntimeError, span: Range<usize>) {
    Report::build(ReportKind::Error, name, span.start)
        .with_config(Config::default().with_compact(true))
        .with_message(format!("{:?}", err))
        .with_label(
            Label::new((name, span))
                .with_message("Here".red())
                .with_color(Color::Red),
        )
        .finish()
        .print((name, Source::from(src)))
        .unwrap()
}
