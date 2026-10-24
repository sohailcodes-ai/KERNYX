use crate::source::{Source, Span};
use std::fmt;

/// A stable diagnostic code, e.g. KX0001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagnosticCode(pub u32);

impl DiagnosticCode {
    pub const fn from_u32(code: u32) -> DiagnosticCode {
        DiagnosticCode(code)
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KX{:04}", self.0)
    }
}

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

/// A single labelled span within a source excerpt (e.g. the caret range).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

/// One diagnostic attached to a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub source: Option<String>, // source path for display
    pub span: Span,             // primary span
    pub labels: Vec<Label>,     // secondary spans
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            source: None,
            span,
            labels: Vec::new(),
            hint: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic {
            code,
            severity: Severity::Warning,
            message: message.into(),
            source: None,
            span,
            labels: Vec::new(),
            hint: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Diagnostic {
        self.source = Some(source.into());
        self
    }

    pub fn with_label(mut self, label: Label) -> Diagnostic {
        self.labels.push(label);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Diagnostic {
        self.hint = Some(hint.into());
        self
    }
}

/// A collection of diagnostics produced by the frontend or runtime.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Diagnostics {
        Diagnostics::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    pub fn extend(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Formatted output for diagnostics.
pub struct Formatter<'a> {
    pub sources: &'a [Source],
    pub color: bool,
}

impl<'a> Formatter<'a> {
    pub fn new(sources: &'a [Source]) -> Formatter<'a> {
        Formatter {
            sources,
            color: atty_is_stream(),
        }
    }

    pub fn format(&self, diagnostic: &Diagnostic) -> String {
        let mut out = String::new();

        let severity_style = if self.color {
            match diagnostic.severity {
                Severity::Error => "\x1b[1;31m",
                Severity::Warning => "\x1b[1;35m",
            }
        } else {
            ""
        };
        let reset = if self.color { "\x1b[0m" } else { "" };
        let label_style = if self.color { "\x1b[1;32m" } else { "" };
        out.push_str(&format!(
            "{}{}[{}]: {}{}\n",
            severity_style, diagnostic.severity, diagnostic.code, diagnostic.message, reset
        ));

        if let Some(path) = &diagnostic.source {
            if let Some(source) = self.find_source(path) {
                let loc = source.location_of(diagnostic.span.start);
                out.push_str(&format!(
                    " --> {}:{}:{}\n",
                    std::path::Path::new(path).display(),
                    loc.line,
                    loc.column
                ));
                out.push_str(&format_source_excerpt(
                    source,
                    diagnostic,
                    label_style,
                    reset,
                ));
            }
        }

        if let Some(hint) = &diagnostic.hint {
            out.push_str(&format!("  = help: {}{}{}\n", label_style, hint, reset));
        }

        out
    }

    fn find_source(&self, path: &str) -> Option<&Source> {
        self.sources
            .iter()
            .find(|s| s.path.to_string_lossy() == path)
    }
}

fn format_source_excerpt(
    source: &Source,
    diagnostic: &Diagnostic,
    label_style: &str,
    reset: &str,
) -> String {
    let mut out = String::new();
    let loc = source.location_of(diagnostic.span.start);
    let line_text = source.line_text(loc.line).unwrap_or("");

    out.push_str("  |\n");
    out.push_str(&format!("{} | {}\n", loc.line, line_text));
    out.push_str("  | ");

    // Build caret range under the primary span.
    let span_start = diagnostic.span.start;
    let span_end = diagnostic.span.end.max(span_start + 1);
    let line_start = source.line_start_offset(loc.line).unwrap_or(0);
    let line_end = source
        .line_end_offset(loc.line)
        .unwrap_or(source.text.len());

    let clip_start = span_start.max(line_start);
    let clip_end = span_end.min(line_end);
    let leading_spaces = clip_start - line_start;
    let caret_len = (clip_end - clip_start).max(1);
    out.push_str(&format!(
        "{}{} {}{}\n",
        " ".repeat(leading_spaces + 1),
        label_style,
        "^".repeat(caret_len),
        reset
    ));
    if let Some(label) = diagnostic.labels.first() {
        if let Some(msg) = &label.message {
            out.push_str(&format!("  | {}{}{}\n", label_style, msg, reset));
        }
    }
    out.push_str("  |\n");
    out
}

/// Whether stderr/stdout looks like a terminal (simple heuristic without deps).
fn atty_is_stream() -> bool {
    // Avoid a dependency; assume color is off unless explicitly requested later.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_format_includes_code_and_location() {
        let source = Source::new("bad.knx", "let x: Text = 42\n".to_string());
        let diag = Diagnostic::error(
            DiagnosticCode::from_u32(8),
            "expected Text, found Int",
            Span::new(14, 15),
        )
        .with_source("bad.knx")
        .with_label(Label {
            span: Span::new(14, 15),
            message: Some("expected Text here".into()),
        })
        .with_hint("provide a text literal or change the declared type");

        let sources = [source];
        let formatter = Formatter::new(&sources);
        let output = formatter.format(&diag);
        assert!(output.contains("error[KX0008]"));
        assert!(output.contains("expected Text, found Int"));
        assert!(output.contains("bad.knx:1:15"));
        assert!(output.contains("help:"));
    }
}
