use std::ops::Range;

/// A range of byte offsets into a source file, [start, end).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn empty() -> Span {
        Span { start: 0, end: 0 }
    }

    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A position in a source file: 1-indexed line and column (byte column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

/// Source text with precomputed line starts for fast location lookups.
#[derive(Debug, Clone)]
pub struct Source {
    pub path: std::path::PathBuf,
    pub text: String,
    line_starts: Vec<usize>,
}

impl Source {
    pub fn new(path: impl Into<std::path::PathBuf>, text: String) -> Source {
        let mut line_starts = vec![0usize];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Source {
            path: path.into(),
            text,
            line_starts,
        }
    }

    pub fn location_of(&self, offset: usize) -> Location {
        let offset = offset.min(self.text.len());
        // line_starts is sorted. binary_search returns Ok(idx) if offset is an
        // exact line start (meaning offset belongs to line idx+1), otherwise
        // Err(idx) where idx is the count of starts <= offset.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx,
        };
        let start = self.line_starts[line - 1];
        let column = offset - start + 1;
        Location { line, column }
    }

    pub fn line_text(&self, line: usize) -> Option<&str> {
        if line == 0 || line > self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line - 1];
        let end = if line < self.line_starts.len() {
            self.line_starts[line] - 1
        } else {
            self.text.len()
        };
        Some(&self.text[start..end])
    }

    pub fn line_start_offset(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line - 1).copied()
    }

    pub fn line_end_offset(&self, line: usize) -> Option<usize> {
        if line < self.line_starts.len() {
            Some(self.line_starts[line] - 1)
        } else {
            Some(self.text.len())
        }
    }

    pub fn slice(&self, span: Span) -> &str {
        let end = span.end.min(self.text.len());
        &self.text[span.start..end]
    }
}

/// Range of bytes within a source file.
pub type ByteRange = Range<usize>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_of_basic() {
        let src = Source::new("test.knx", "ab\ncde\nf".to_string());
        assert_eq!(src.location_of(0), Location { line: 1, column: 1 });
        assert_eq!(src.location_of(1), Location { line: 1, column: 2 });
        assert_eq!(src.location_of(2), Location { line: 1, column: 3 });
        assert_eq!(src.location_of(3), Location { line: 2, column: 1 });
        assert_eq!(src.location_of(4), Location { line: 2, column: 2 });
        assert_eq!(src.location_of(7), Location { line: 3, column: 1 });
    }

    #[test]
    fn line_text() {
        let src = Source::new("test.knx", "ab\ncde\nf".to_string());
        assert_eq!(src.line_text(1), Some("ab"));
        assert_eq!(src.line_text(2), Some("cde"));
        assert_eq!(src.line_text(3), Some("f"));
        assert_eq!(src.line_text(4), None);
    }

    #[test]
    fn span_merge() {
        let a = Span::new(0, 3);
        let b = Span::new(5, 8);
        assert_eq!(a.merge(b), Span::new(0, 8));
    }
}
