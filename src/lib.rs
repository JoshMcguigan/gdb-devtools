use std::ops::Range;

mod iters;

#[derive(Debug)]
struct Token<'a> {
    text: &'a str,
    span: Range<usize>,
}

struct Line<'a> {
    text: &'a str,
    span: Range<usize>,
}

#[derive(Debug)]
enum Command<'a> {
    Define {
        define: Token<'a>,
        body: Vec<Command<'a>>,
    },
    Other {
        command: Token<'a>,
        args: Vec<Token<'a>>,
    },
}

fn parse(input: &str) -> Vec<Command> {
    parse_until(&mut iters::lines(input).into_iter(), false)
}

fn parse_until<'a>(
    input: &mut impl Iterator<Item = Line<'a>>,
    until_end: bool,
) -> Vec<Command<'a>> {
    let mut commands = vec![];
    while let Some(line) = input.next() {
        let mut tokens = iters::tokens(&line);
        match tokens.first().map(|t| t.text) {
            Some("define") => {
                let body = parse_until(input, true);
                commands.push(Command::Define {
                    define: tokens.remove(0),
                    body,
                });
            }
            Some("end") => {
                if until_end {
                    return commands;
                }
            }
            // Ignore empty lines
            None => {}
            Some(command) => {
                commands.push(Command::Other {
                    command: tokens.remove(0),
                    args: tokens,
                });
            }
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::parse;

    fn check_lex_and_parse(input: &str, expect_parse: Expect) {
        expect_parse.assert_eq(
            &parse(input)
                .into_iter()
                .map(|s| format!("{:#?}\n", s))
                .collect::<Vec<String>>()
                .join(""),
        );
    }

    #[test]
    fn commands() {
        let script = r#"
command_with_no_args
command_with_one_arg foo
command_with_two_args foo bar
        "#;

        check_lex_and_parse(
            script,
            expect![[r#"
                Other {
                    command: Token {
                        text: "command_with_no_args",
                        span: 1..21,
                    },
                    args: [],
                }
                Other {
                    command: Token {
                        text: "command_with_one_arg",
                        span: 22..42,
                    },
                    args: [
                        Token {
                            text: "foo",
                            span: 43..46,
                        },
                    ],
                }
                Other {
                    command: Token {
                        text: "command_with_two_args",
                        span: 47..68,
                    },
                    args: [
                        Token {
                            text: "foo",
                            span: 69..72,
                        },
                        Token {
                            text: "bar",
                            span: 73..76,
                        },
                    ],
                }
            "#]],
        );
    }

    #[test]
    fn function_definition() {
        let script = r#"
define say_hi
    echo hi
end
        "#;

        check_lex_and_parse(
            script,
            expect![[r#"
                Define {
                    define: Token {
                        text: "define",
                        span: 1..7,
                    },
                    body: [
                        Other {
                            command: Token {
                                text: "echo",
                                span: 19..23,
                            },
                            args: [
                                Token {
                                    text: "hi",
                                    span: 24..26,
                                },
                            ],
                        },
                    ],
                }
            "#]],
        );
    }
}
