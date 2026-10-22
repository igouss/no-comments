use crate::comment_form::CommentForm;
use crate::position::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub position: Position,
    pub form: CommentForm,
}
