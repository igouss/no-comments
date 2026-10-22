use no_comments::position::{LineIndex, Position};

#[test]
fn the_start_of_a_file_is_one_one() {
    assert_eq!(
        LineIndex::new("").position(0),
        Position { line: 1, column: 1 }
    );
}

#[test]
fn an_offset_on_the_first_line() {
    assert_eq!(
        LineIndex::new("ab\ncd").position(1),
        Position { line: 1, column: 2 }
    );
}

#[test]
fn the_newline_belongs_to_its_line() {
    assert_eq!(
        LineIndex::new("ab\ncd").position(2),
        Position { line: 1, column: 3 }
    );
}

#[test]
fn the_first_byte_after_a_newline_starts_the_next_line() {
    assert_eq!(
        LineIndex::new("ab\ncd").position(3),
        Position { line: 2, column: 1 }
    );
}

#[test]
fn many_lines() {
    assert_eq!(
        LineIndex::new("a\nb\n\nééd").position(9),
        Position { line: 4, column: 3 }
    );
}
