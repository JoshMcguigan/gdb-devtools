use super::{CommandLine, Location, Token};

pub(crate) fn lines(text: &str) -> Vec<CommandLine> {
    let mut lines = vec![];

    let mut span_start = 0;
    let mut line_number = 0;
    let mut escaped = false;

    for (index, character) in text.char_indices() {
        if character == '\n' && !escaped {
            let span = span_start..index + 1;
            lines.push(CommandLine {
                text: &text[span.clone()],
                start_line_in_file: line_number,
            });

            span_start = index + 1;
            line_number += 1;
        }

        if character == '\\' {
            escaped = true;
        } else {
            escaped = false;
        }
    }

    if span_start < text.len() {
        let span = span_start..text.len();
        lines.push(CommandLine {
            text: &text[span.clone()],
            start_line_in_file: line_number,
        });
    }

    lines
}

pub(crate) fn tokens<'a, 'line>(line: &'a CommandLine<'line>) -> Vec<Token<'line>> {
    let mut tokens = vec![];

    let mut span_start = match line.text.find(|c: char| !c.is_whitespace()) {
        Some(offset) => offset,
        None => return tokens,
    };
    let mut currently_in_whitespace = false;
    let mut escaped = false;
    let mut line_start_column = 0;
    let mut line_number = 0;

    for (index, character) in line
        .text
        .char_indices()
        .skip_while(|(_, c)| c.is_whitespace())
    {
        if character == '\n' && escaped {
            escaped = false;
            currently_in_whitespace = true;
            line_start_column = index + 1;
            line_number += 1;
            continue;
        }

        if character.is_whitespace() {
            if !currently_in_whitespace {
                let span_in_line = span_start..index;
                tokens.push(Token {
                    text: &line.text[span_in_line],
                    location_in_file: Location {
                        line: line.start_line_in_file + line_number,
                        column: span_start - line_start_column,
                    },
                });
            }

            currently_in_whitespace = true;
        } else {
            if currently_in_whitespace {
                currently_in_whitespace = false;
                span_start = index;
            }
        }

        if character == '\\' {
            escaped = true;
        } else {
            escaped = false;
        }
    }
    if !currently_in_whitespace {
        let span_in_line = span_start..line.text.len();
        tokens.push(Token {
            text: &line.text[span_in_line],
            location_in_file: Location {
                line: line.start_line_in_file + line_number,
                column: span_start - line_start_column,
            },
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::{lines, tokens};

    fn check_lines_and_tokens(input: &str, expect_parse: Expect) {
        expect_parse.assert_eq(
            &lines(input)
                .into_iter()
                .map(|line| tokens(&line))
                .map(|s| format!("{:#?}\n", s))
                .collect::<Vec<String>>()
                .join(""),
        );
    }

    #[test]
    fn lines_and_tokens() {
        let script = r#"
command_with_no_args
command_with_one_arg foo
command_with_two_args foo bar
 leading_space
        "#;

        check_lines_and_tokens(
            script,
            expect![[r#"
                []
                [
                    Token {
                        text: "command_with_no_args",
                        location_in_file: Location {
                            line: 1,
                            column: 0,
                        },
                    },
                ]
                [
                    Token {
                        text: "command_with_one_arg",
                        location_in_file: Location {
                            line: 2,
                            column: 0,
                        },
                    },
                    Token {
                        text: "foo",
                        location_in_file: Location {
                            line: 2,
                            column: 21,
                        },
                    },
                ]
                [
                    Token {
                        text: "command_with_two_args",
                        location_in_file: Location {
                            line: 3,
                            column: 0,
                        },
                    },
                    Token {
                        text: "foo",
                        location_in_file: Location {
                            line: 3,
                            column: 22,
                        },
                    },
                    Token {
                        text: "bar",
                        location_in_file: Location {
                            line: 3,
                            column: 26,
                        },
                    },
                ]
                [
                    Token {
                        text: "leading_space",
                        location_in_file: Location {
                            line: 4,
                            column: 1,
                        },
                    },
                ]
                []
            "#]],
        );
    }

    #[test]
    fn lines_and_tokens_without_trailing_whitespace() {
        let script = "command_with_no_args";

        check_lines_and_tokens(
            script,
            expect![[r#"
                [
                    Token {
                        text: "command_with_no_args",
                        location_in_file: Location {
                            line: 0,
                            column: 0,
                        },
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn lines_and_tokens_escaped_newline() {
        let script = r#"
command_with_one_arg \
    foo
        "#;

        check_lines_and_tokens(
            script,
            expect![[r#"
                []
                [
                    Token {
                        text: "command_with_one_arg",
                        location_in_file: Location {
                            line: 1,
                            column: 0,
                        },
                    },
                    Token {
                        text: "foo",
                        location_in_file: Location {
                            line: 2,
                            column: 4,
                        },
                    },
                ]
                []
            "#]],
        );
    }
}
