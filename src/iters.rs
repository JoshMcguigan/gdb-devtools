use super::{Line, Token};

pub(crate) fn lines(text: &str) -> Vec<Line> {
    let mut lines = vec![];

    let mut span_start = 0;

    for (index, character) in text.char_indices() {
        if character == '\n' {
            let span = span_start..index + 1;
            lines.push(Line {
                text: &text[span.clone()],
                span,
            });

            span_start = index + 1;
        }
    }

    if span_start < text.len() {
        let span = span_start..text.len();
        lines.push(Line {
            text: &text[span.clone()],
            span,
        });
    }

    lines
}

pub(crate) fn tokens<'a, 'line>(line: &'a Line<'line>) -> Vec<Token<'line>> {
    let mut tokens = vec![];

    let mut span_start = match line.text.find(|c: char| !c.is_whitespace()) {
        Some(offset) => offset,
        None => return tokens,
    };
    let mut currently_in_whitespace = false;

    for (index, character) in line
        .text
        .char_indices()
        .skip_while(|(_, c)| c.is_whitespace())
    {
        if character.is_whitespace() {
            if !currently_in_whitespace {
                let span_in_line = span_start..index;
                let span_in_file =
                    (span_in_line.start + line.span.start)..(span_in_line.end + line.span.start);
                tokens.push(Token {
                    text: &line.text[span_in_line],
                    span: span_in_file,
                });
            }

            currently_in_whitespace = true;
        } else {
            if currently_in_whitespace {
                currently_in_whitespace = false;
                span_start = index;
            }
        }
    }
    if !currently_in_whitespace {
        let span_in_line = span_start..line.text.len();
        let span_in_file =
            (span_in_line.start + line.span.start)..(span_in_line.end + line.span.start);
        tokens.push(Token {
            text: &line.text[span_in_line],
            span: span_in_file,
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
                        span: 1..21,
                    },
                ]
                [
                    Token {
                        text: "command_with_one_arg",
                        span: 22..42,
                    },
                    Token {
                        text: "foo",
                        span: 43..46,
                    },
                ]
                [
                    Token {
                        text: "command_with_two_args",
                        span: 47..68,
                    },
                    Token {
                        text: "foo",
                        span: 69..72,
                    },
                    Token {
                        text: "bar",
                        span: 73..76,
                    },
                ]
                [
                    Token {
                        text: "leading_space",
                        span: 78..91,
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
                    span: 0..20,
                },
            ]
        "#]],
        );
    }
}
