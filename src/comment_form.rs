#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommentForm {
    Line,
    Block,
    OuterLineDoc,
    InnerLineDoc,
    OuterBlockDoc,
    InnerBlockDoc,
}

impl CommentForm {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            CommentForm::Line => "line",
            CommentForm::Block => "block",
            CommentForm::OuterLineDoc => "outer-line-doc",
            CommentForm::InnerLineDoc => "inner-line-doc",
            CommentForm::OuterBlockDoc => "outer-block-doc",
            CommentForm::InnerBlockDoc => "inner-block-doc",
        }
    }
}
