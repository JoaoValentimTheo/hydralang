#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(u32);

impl SourceId {
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(source: SourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    #[must_use]
    pub const fn empty(source: SourceId, offset: usize) -> Self {
        Self::new(source, offset, offset)
    }

    #[must_use]
    pub fn join(self, other: Self) -> Option<Self> {
        if self.source != other.source {
            return None;
        }
        Some(Self::new(
            self.source,
            self.start.min(other.start),
            self.end.max(other.end),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct SourceFile {
    id: SourceId,
    name: String,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    #[must_use]
    pub fn new(id: SourceId, name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self {
            id,
            name: name.into(),
            text,
            line_starts,
        }
    }

    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn eof_span(&self) -> Span {
        Span::empty(self.id, self.text.len())
    }

    #[must_use]
    pub fn span(&self, start: usize, end: usize) -> Option<Span> {
        (start <= end
            && end <= self.text.len()
            && self.text.is_char_boundary(start)
            && self.text.is_char_boundary(end))
        .then(|| Span::new(self.id, start, end))
    }

    #[must_use]
    pub fn slice(&self, span: Span) -> Option<&str> {
        if span.source != self.id || span.start > span.end {
            return None;
        }
        self.text.get(span.start..span.end)
    }

    #[must_use]
    pub fn line_col(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = *self.line_starts.get(line_index)?;
        let column = self.text.get(line_start..offset)?.chars().count() + 1;
        Some((line_index + 1, column))
    }

    #[must_use]
    pub fn line_text(&self, one_based_line: usize) -> Option<&str> {
        let index = one_based_line.checked_sub(1)?;
        let start = *self.line_starts.get(index)?;
        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.text.len());
        self.text
            .get(start..end)
            .map(|line| line.trim_end_matches('\n'))
    }
}

#[derive(Default, Debug)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    #[must_use]
    pub const fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, name: impl Into<String>, text: impl Into<String>) -> SourceId {
        let raw = u32::try_from(self.files.len()).unwrap_or(u32::MAX);
        let id = SourceId::new(raw);
        self.files.push(SourceFile::new(id, name, text));
        id
    }

    #[must_use]
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        usize::try_from(id.raw())
            .ok()
            .and_then(|index| self.files.get(index))
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceFile, SourceId};

    #[test]
    fn unicode_columns_are_scalar_based() {
        let file = SourceFile::new(SourceId::new(0), "test.hyd", "αβ\nhello");
        let offset = "α".len();
        assert_eq!(file.line_col(offset), Some((1, 2)));
    }

    #[test]
    fn malformed_spans_do_not_slice() {
        let file = SourceFile::new(SourceId::new(0), "test.hyd", "hello");
        let other = SourceId::new(1);
        assert!(file.slice(super::Span::new(other, 0, 2)).is_none());
        assert!(file.span(3, 2).is_none());
        assert!(file.span(0, 99).is_none());
    }
}
