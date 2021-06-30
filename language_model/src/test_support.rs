use crate::parse::Location;

const CURSOR_SYMBOL: &str = "<|>";

pub(crate) fn parse_cursor_position(script_containing_cursor: &str) -> (String, Location) {
    let location = script_containing_cursor
        .lines()
        .enumerate()
        .find_map(|(line, line_text)| {
            // Find returns the byte offset, which isn't what we would want if we needed to
            // support UTF-8, but for now we are only supporting ASCII.
            let column = line_text.find(CURSOR_SYMBOL)?;
            Some(Location { line, column })
        })
        .expect("script should contain cursor");

    // We specifically only replace the first cursor instance here so we can check
    // for multiple cursors below.
    let script_without_cursor = script_containing_cursor.replacen(CURSOR_SYMBOL, "", 1);

    if script_without_cursor.contains(CURSOR_SYMBOL) {
        panic!("script should only contain single cursor");
    }

    (script_without_cursor, location)
}

#[cfg(test)]
mod tests {
    use crate::parse::Location;

    use super::parse_cursor_position;

    #[test]
    #[should_panic]
    fn panics_on_missing_cursor() {
        parse_cursor_position("no cursor");
    }

    #[test]
    #[should_panic]
    fn panics_on_more_than_one_cursor() {
        parse_cursor_position("<|> <|>");
    }

    #[test]
    fn empty_script() {
        let (script, cursor_location) = parse_cursor_position("<|>");

        assert_eq!("", script);
        assert_eq!(Location { line: 0, column: 0 }, cursor_location);
    }

    #[test]
    fn end_of_first_line() {
        let (script, cursor_location) = parse_cursor_position("foo <|>");

        assert_eq!("foo ", script);
        assert_eq!(Location { line: 0, column: 4 }, cursor_location);
    }

    #[test]
    fn middle_of_first_line() {
        let (script, cursor_location) = parse_cursor_position("foo <|>bar");

        assert_eq!("foo bar", script);
        assert_eq!(Location { line: 0, column: 4 }, cursor_location);
    }

    #[test]
    fn last_line() {
        let (script, cursor_location) = parse_cursor_position("foo\n<|>bar");

        assert_eq!("foo\nbar", script);
        assert_eq!(Location { line: 1, column: 0 }, cursor_location);
    }

    #[test]
    fn middle_line() {
        let (script, cursor_location) = parse_cursor_position("foo\n<|>bar\nbaz");

        assert_eq!("foo\nbar\nbaz", script);
        assert_eq!(Location { line: 1, column: 0 }, cursor_location);
    }
}
