use std::{error::Error, fmt};

/// A deterministic zero-based index of a source within its originating [`SourceMap`].
///
/// IDs from independently created maps may be equal. Use an ID only with its
/// originating map or a clone retaining the same sources; lookups do not track
/// cross-map provenance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(usize);

impl SourceId {
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceSpan {
    source: SourceId,
    start: usize,
    end: usize,
}

impl SourceSpan {
    pub const fn new(source: SourceId, start: usize, end: usize) -> Result<Self, InvalidSpan> {
        if start > end {
            return Err(InvalidSpan { start, end });
        }

        Ok(Self { source, start, end })
    }

    pub const fn empty(source: SourceId, offset: usize) -> Self {
        Self {
            source,
            start: offset,
            end: offset,
        }
    }

    pub const fn source(self) -> SourceId {
        self.source
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSpan {
    start: usize,
    end: usize,
}

impl InvalidSpan {
    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }
}

impl fmt::Display for InvalidSpan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid source span: start {} is greater than end {}",
            self.start, self.end
        )
    }
}

impl Error for InvalidSpan {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl LineColumn {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        let mut chars = source.char_indices().peekable();

        while let Some((index, ch)) = chars.next() {
            match ch {
                '\r' if chars.peek().is_some_and(|(_, next)| *next == '\n') => {
                    chars.next();
                    line_starts.push(index + 2);
                }
                '\n' => line_starts.push(index + 1),
                _ => {}
            }
        }

        Self { line_starts }
    }

    fn line_column(&self, source: &str, offset: usize) -> Option<LineColumn> {
        let prefix = source.get(..offset)?;
        let line_count = self
            .line_starts
            .partition_point(|line_start| *line_start <= offset);
        let line_index = line_count.checked_sub(1)?;
        let line_start = *self.line_starts.get(line_index)?;
        let column = prefix.get(line_start..)?.chars().count() + 1;

        Some(LineColumn::new(line_index + 1, column))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceFile {
    id: SourceId,
    logical_name: String,
    text: String,
    line_index: LineIndex,
}

impl SourceFile {
    fn new(id: SourceId, logical_name: String, text: String) -> Self {
        let line_index = LineIndex::new(&text);
        Self {
            id,
            logical_name,
            text,
            line_index,
        }
    }

    pub const fn id(&self) -> SourceId {
        self.id
    }

    pub fn logical_name(&self) -> &str {
        &self.logical_name
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn line_column(&self, offset: usize) -> Option<LineColumn> {
        self.line_index.line_column(&self.text, offset)
    }
}

/// An ordered collection of sources with deterministic map-local IDs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceMap {
    sources: Vec<SourceFile>,
}

impl SourceMap {
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_source(
        &mut self,
        logical_name: impl Into<String>,
        text: impl Into<String>,
    ) -> SourceId {
        let id = SourceId(self.sources.len());
        let file = SourceFile::new(id, logical_name.into(), text.into());
        self.sources.push(file);
        id
    }

    /// Returns the source at `id.index()`, or `None` if that index is absent.
    ///
    /// The caller must use an ID originating from this map or a clone retaining
    /// the same sources. A foreign ID with an in-range index cannot be detected.
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.sources.get(id.index())
    }
}
