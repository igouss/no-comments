#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct LineIndex<'a> {
    text: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> LineIndex<'a> {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(
                text.match_indices('\n')
                    .map(|(offset, _): (usize, &str)| offset + 1),
            )
            .collect();
        LineIndex { text, line_starts }
    }

    #[must_use]
    pub fn position(&self, byte_offset: usize) -> Position {
        let line_index: usize = self
            .line_starts
            .partition_point(|start: &usize| *start <= byte_offset)
            - 1;
        let line_start: usize = self.line_starts[line_index];
        let column: usize = self.text[line_start..byte_offset].chars().count() + 1;
        Position {
            line: line_index + 1,
            column,
        }
    }
}
