use hydra_source::{SourceMap, Span};
use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Lexer,
    Parser,
    Resolution,
    Type,
    Runtime,
    Internal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub phase: Phase,
    pub message: String,
    pub primary: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Vec<String>,
}

impl Diagnostic {
    #[must_use]
    pub fn error(
        code: &'static str,
        phase: Phase,
        message: impl Into<String>,
        primary: Span,
    ) -> Self {
        Self {
            code,
            phase,
            message: message.into(),
            primary,
            labels: Vec::new(),
            notes: Vec::new(),
            help: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    #[must_use]
    pub fn render(&self, sources: &SourceMap) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "error[{}]: {}", self.code, self.message);
        if let Some(file) = sources.get(self.primary.source) {
            if let Some((line, column)) = file.line_col(self.primary.start) {
                let _ = writeln!(out, " --> {}:{line}:{column}", file.name());
                if let Some(source_line) = file.line_text(line) {
                    let _ = writeln!(out, "  |");
                    let _ = writeln!(out, "{line:>3} | {source_line}");
                    let width = file
                        .slice(self.primary)
                        .map_or(1, |slice| slice.chars().count().max(1));
                    let _ = writeln!(
                        out,
                        "  | {}{}",
                        " ".repeat(column.saturating_sub(1)),
                        "^".repeat(width)
                    );
                }
            }
        }
        for label in &self.labels {
            if let Some(file) = sources.get(label.span.source) {
                if let Some((line, column)) = file.line_col(label.span.start) {
                    let _ = writeln!(
                        out,
                        "  = {}:{line}:{column}: {}",
                        file.name(),
                        label.message
                    );
                }
            }
        }
        for note in &self.notes {
            let _ = writeln!(out, "  = note: {note}");
        }
        for help in &self.help {
            let _ = writeln!(out, "  = help: {help}");
        }
        out
    }
}
